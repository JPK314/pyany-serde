use std::mem::size_of;

use pyo3::exceptions::asyncio::InvalidStateError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

use crate::common::numpy_dtype_enum::NumpyDtype;
use crate::common::python_type::{detect_python_type, PythonType};

// This enum is used to store information about a type which is sent between processes to dynamically recover a Box<dyn PyAnySerde>
#[derive(Debug, PartialEq, Clone)]
pub enum PyAnySerdeType {
    PICKLE,
    INT,
    FLOAT,
    COMPLEX,
    BOOLEAN,
    STRING,
    BYTES,
    DYNAMIC,
    NUMPY {
        dtype: NumpyDtype,
    },
    LIST {
        items: Box<PyAnySerdeType>,
    },
    SET {
        items: Box<PyAnySerdeType>,
    },
    TUPLE {
        items: Vec<PyAnySerdeType>,
    },
    DICT {
        keys: Box<PyAnySerdeType>,
        values: Box<PyAnySerdeType>,
    },
    TYPEDDICT {
        kv_pairs: Vec<(String, PyAnySerdeType)>,
    },
    OPTION {
        value: Box<PyAnySerdeType>,
    },
    OTHER,
}

impl PyAnySerdeType {
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            PyAnySerdeType::PICKLE => vec![0],
            PyAnySerdeType::INT => vec![1],
            PyAnySerdeType::FLOAT => vec![2],
            PyAnySerdeType::COMPLEX => vec![3],
            PyAnySerdeType::BOOLEAN => vec![4],
            PyAnySerdeType::STRING => vec![5],
            PyAnySerdeType::BYTES => vec![6],
            PyAnySerdeType::DYNAMIC => vec![7],
            PyAnySerdeType::NUMPY { dtype } => match dtype {
                NumpyDtype::INT8 => vec![8, 0],
                NumpyDtype::INT16 => vec![8, 1],
                NumpyDtype::INT32 => vec![8, 2],
                NumpyDtype::INT64 => vec![8, 3],
                NumpyDtype::UINT8 => vec![8, 4],
                NumpyDtype::UINT16 => vec![8, 5],
                NumpyDtype::UINT32 => vec![8, 6],
                NumpyDtype::UINT64 => vec![8, 7],
                NumpyDtype::FLOAT32 => vec![8, 8],
                NumpyDtype::FLOAT64 => vec![8, 9],
            },
            PyAnySerdeType::LIST { items } => {
                let mut bytes = vec![9];
                bytes.append(&mut items.serialize());
                bytes
            }
            PyAnySerdeType::SET { items } => {
                let mut bytes = vec![10];
                bytes.append(&mut items.serialize());
                bytes
            }
            PyAnySerdeType::TUPLE { items } => {
                let mut bytes = vec![11];
                bytes.extend_from_slice(&items.len().to_ne_bytes());
                for item in items {
                    bytes.append(&mut item.serialize());
                }
                bytes
            }
            PyAnySerdeType::DICT { keys, values } => {
                let mut bytes = vec![12];
                bytes.append(&mut keys.serialize());
                bytes.append(&mut values.serialize());
                bytes
            }

            PyAnySerdeType::TYPEDDICT { kv_pairs } => {
                let mut bytes = vec![13];
                bytes.extend_from_slice(&kv_pairs.len().to_ne_bytes());
                for (key, serde_enum) in kv_pairs {
                    let key_bytes = key.as_bytes();
                    bytes.extend_from_slice(&key_bytes.len().to_ne_bytes());
                    bytes.extend_from_slice(key.as_bytes());
                    bytes.append(&mut serde_enum.serialize());
                }
                bytes
            }
            PyAnySerdeType::OPTION { value } => {
                let mut bytes = vec![14];
                bytes.append(&mut value.serialize());
                bytes
            }
            PyAnySerdeType::OTHER => vec![15],
        }
    }
}

impl<'py> TryFrom<Bound<'py, PyAny>> for PyAnySerdeType {
    type Error = PyErr;

    fn try_from(value: Bound<'py, PyAny>) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl<'py, 'a> TryFrom<&'a Bound<'py, PyAny>> for PyAnySerdeType {
    type Error = PyErr;

    fn try_from(value: &'a Bound<'py, PyAny>) -> Result<Self, Self::Error> {
        let python_type = detect_python_type(value)?;
        match python_type {
            PythonType::BOOL => Ok(PyAnySerdeType::BOOLEAN),
            PythonType::INT => Ok(PyAnySerdeType::INT),
            PythonType::FLOAT => Ok(PyAnySerdeType::FLOAT),
            PythonType::COMPLEX => Ok(PyAnySerdeType::COMPLEX),
            PythonType::STRING => Ok(PyAnySerdeType::STRING),
            PythonType::BYTES => Ok(PyAnySerdeType::BYTES),
            PythonType::NUMPY { dtype } => Ok(PyAnySerdeType::NUMPY { dtype }),
            PythonType::LIST => Ok(PyAnySerdeType::LIST {
                items: Box::new(value.get_item(0)?.try_into()?),
            }),
            PythonType::SET => Ok(PyAnySerdeType::SET {
                items: Box::new(
                    value
                        .py()
                        .get_type::<PyAny>()
                        .call_method1("list", (value,))?
                        .get_item(0)?
                        .try_into()?,
                ),
            }),
            PythonType::TUPLE => {
                let tuple = value.downcast::<PyTuple>()?;
                let mut items = Vec::with_capacity(tuple.len());
                for item in tuple.iter() {
                    items.push(item.try_into()?);
                }
                Ok(PyAnySerdeType::TUPLE { items })
            }
            PythonType::DICT => {
                let keys = value.downcast::<PyDict>()?.keys().get_item(0)?;
                let values = value.downcast::<PyDict>()?.values().get_item(0)?;
                Ok(PyAnySerdeType::DICT {
                    keys: Box::new(keys.try_into()?),
                    values: Box::new(values.try_into()?),
                })
            }
            PythonType::OTHER => Ok(PyAnySerdeType::OTHER),
        }
    }
}

pub fn retrieve_pyany_serde_type(buf: &[u8], offset: usize) -> PyResult<(PyAnySerdeType, usize)> {
    let mut offset = offset;
    let v = buf[offset];
    offset += 1;
    let serde = match v {
        0 => Ok(PyAnySerdeType::PICKLE),
        1 => Ok(PyAnySerdeType::INT),
        2 => Ok(PyAnySerdeType::FLOAT),
        3 => Ok(PyAnySerdeType::COMPLEX),
        4 => Ok(PyAnySerdeType::BOOLEAN),
        5 => Ok(PyAnySerdeType::STRING),
        6 => Ok(PyAnySerdeType::BYTES),
        7 => Ok(PyAnySerdeType::DYNAMIC),
        8 => {
            let dtype = match buf[offset] {
                0 => Ok(NumpyDtype::INT8),
                1 => Ok(NumpyDtype::INT16),
                2 => Ok(NumpyDtype::INT32),
                3 => Ok(NumpyDtype::INT64),
                4 => Ok(NumpyDtype::UINT8),
                5 => Ok(NumpyDtype::UINT16),
                6 => Ok(NumpyDtype::UINT32),
                7 => Ok(NumpyDtype::UINT64),
                8 => Ok(NumpyDtype::FLOAT32),
                9 => Ok(NumpyDtype::FLOAT64),
                v => Err(InvalidStateError::new_err(format!(
                    "tried to deserialize PyAnySerdeType as NUMPY but got {} for NumpyDtype",
                    v
                ))),
            }?;
            offset += 1;
            Ok(PyAnySerdeType::NUMPY { dtype })
        }
        9 => {
            let items;
            (items, offset) = retrieve_pyany_serde_type(buf, offset)?;
            Ok(PyAnySerdeType::LIST {
                items: Box::new(items),
            })
        }
        10 => {
            let items;
            (items, offset) = retrieve_pyany_serde_type(buf, offset)?;
            Ok(PyAnySerdeType::SET {
                items: Box::new(items),
            })
        }
        11 => {
            let end = offset + size_of::<usize>();
            let items_len = usize::from_ne_bytes(buf[offset..end].try_into()?);
            offset = end;
            let mut items = Vec::with_capacity(items_len);
            for _ in 0..items_len {
                let item;
                (item, offset) = retrieve_pyany_serde_type(buf, offset)?;
                items.push(item);
            }
            Ok(PyAnySerdeType::TUPLE { items })
        }
        12 => {
            let keys;
            (keys, offset) = retrieve_pyany_serde_type(buf, offset)?;
            let values;
            (values, offset) = retrieve_pyany_serde_type(buf, offset)?;
            Ok(PyAnySerdeType::DICT {
                keys: Box::new(keys),
                values: Box::new(values),
            })
        }
        13 => {
            let mut end = offset + size_of::<usize>();
            let items_len = usize::from_ne_bytes(buf[offset..end].try_into()?);
            offset = end;
            let mut kv_pairs = Vec::with_capacity(items_len);
            for _ in 0..items_len {
                end = offset + size_of::<usize>();
                let key_bytes_len = usize::from_ne_bytes(buf[offset..end].try_into()?);
                offset = end;
                end = offset + key_bytes_len;
                let key = String::from_utf8(buf[offset..end].to_vec())?;
                offset = end;
                let item;
                (item, offset) = retrieve_pyany_serde_type(buf, offset)?;
                kv_pairs.push((key, item));
            }
            Ok(PyAnySerdeType::TYPEDDICT { kv_pairs })
        }
        14 => {
            let value;
            (value, offset) = retrieve_pyany_serde_type(buf, offset)?;
            Ok(PyAnySerdeType::OPTION {
                value: Box::new(value),
            })
        }
        15 => Ok(PyAnySerdeType::OTHER),
        v => Err(InvalidStateError::new_err(format!(
            "Tried to deserialize PyAnySerdeType but got {}",
            v
        ))),
    }?;
    Ok((serde, offset))
}

#[cfg(test)]
mod tests {
    use pyo3::PyResult;

    use crate::common::numpy_dtype_enum::NumpyDtype;

    use super::*;

    #[test]
    fn test_retrieve_pyany_serde_type_1() -> PyResult<()> {
        let enum_bytes = vec![11_u8, 2, 0, 0, 0, 0, 0, 0, 0, 5, 1];
        let (serde, _) = retrieve_pyany_serde_type(&enum_bytes[..], 0)?;
        assert_eq!(
            serde,
            PyAnySerdeType::TUPLE {
                items: vec![PyAnySerdeType::STRING, PyAnySerdeType::INT]
            }
        );
        Ok(())
    }
    #[test]
    fn test_retrieve_pyany_serde_type_2() -> PyResult<()> {
        let enum_bytes = vec![9_u8, 8, 9];
        let (serde, _) = retrieve_pyany_serde_type(&enum_bytes[..], 0)?;
        assert_eq!(
            serde,
            PyAnySerdeType::LIST {
                items: Box::new(PyAnySerdeType::NUMPY {
                    dtype: NumpyDtype::FLOAT64
                })
            }
        );
        Ok(())
    }
}
