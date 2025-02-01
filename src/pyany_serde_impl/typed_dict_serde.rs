use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};

use crate::communication::{append_python, retrieve_python};
use crate::pyany_serde::PyAnySerde;

use crate::pyany_serde_type::PyAnySerdeType;

#[derive(Clone)]
pub struct TypedDictSerde {
    serde_kv_list: Vec<(Py<PyString>, Option<Box<dyn PyAnySerde>>)>,
    serde_enum: PyAnySerdeType,
    serde_enum_bytes: Vec<u8>,
}

impl TypedDictSerde {
    pub fn new(serde_kv_list: Vec<(Py<PyString>, Option<Box<dyn PyAnySerde>>)>) -> PyResult<Self> {
        let mut kv_pairs_option = Some(Vec::with_capacity(serde_kv_list.len()));
        for (key, serde_option) in serde_kv_list.iter() {
            match serde_option {
                Some(pyany_serde) => {
                    kv_pairs_option
                        .as_mut()
                        .unwrap()
                        .push((key.to_string(), pyany_serde.get_enum().clone()));
                }
                None => {
                    kv_pairs_option = None;
                    break;
                }
            }
        }
        let serde_enum = if let Some(kv_pairs) = kv_pairs_option {
            PyAnySerdeType::TYPEDDICT { kv_pairs }
        } else {
            PyAnySerdeType::OTHER
        };
        Ok(TypedDictSerde {
            serde_kv_list,
            serde_enum_bytes: serde_enum.serialize(),
            serde_enum,
        })
    }
}

impl PyAnySerde for TypedDictSerde {
    fn append<'py>(
        &mut self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        let mut offset = offset;
        for (key, serde_option) in self.serde_kv_list.iter_mut() {
            offset = append_python(
                buf,
                offset,
                &obj.get_item(key.bind(obj.py()))?,
                serde_option,
            )?;
        }
        Ok(offset)
    }

    fn retrieve<'py>(
        &mut self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let mut kv_list = Vec::with_capacity(self.serde_kv_list.len());
        let mut offset = offset;
        for (key, serde_option) in self.serde_kv_list.iter_mut() {
            let item;
            (item, offset) = retrieve_python(py, buf, offset, serde_option)?;
            kv_list.push((key.clone_ref(py), item));
        }
        Ok((
            PyDict::from_sequence(&kv_list.into_pyobject(py)?)?.into_any(),
            offset,
        ))
    }

    fn get_enum(&self) -> &PyAnySerdeType {
        &self.serde_enum
    }

    fn get_enum_bytes(&self) -> &[u8] {
        &self.serde_enum_bytes
    }
}
