use std::marker::PhantomData;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PySet, PyTuple};

use crate::{
    common::{
        detect_python_type, get_python_type_byte, retrieve_python_type, NumpyDtype, PythonType,
    },
    communication::{append_usize, retrieve_usize},
    PyAnySerde,
};

use super::{
    BoolSerde, BytesSerde, ComplexSerde, FloatSerde, IntSerde, NumpyDynamicShapeSerde, PickleSerde,
    StringSerde,
};

#[derive(Clone)]
pub struct DynamicSerde {
    pickle_serde: PickleSerde,
    int_serde: IntSerde,
    float_serde: FloatSerde,
    complex_serde: ComplexSerde,
    boolean_serde: BoolSerde,
    string_serde: StringSerde,
    bytes_serde: BytesSerde,
    numpy_i8_serde: NumpyDynamicShapeSerde<i8>,
    numpy_i16_serde: NumpyDynamicShapeSerde<i16>,
    numpy_i32_serde: NumpyDynamicShapeSerde<i32>,
    numpy_i64_serde: NumpyDynamicShapeSerde<i64>,
    numpy_u8_serde: NumpyDynamicShapeSerde<u8>,
    numpy_u16_serde: NumpyDynamicShapeSerde<u16>,
    numpy_u32_serde: NumpyDynamicShapeSerde<u32>,
    numpy_u64_serde: NumpyDynamicShapeSerde<u64>,
    numpy_f32_serde: NumpyDynamicShapeSerde<f32>,
    numpy_f64_serde: NumpyDynamicShapeSerde<f64>,
}

impl DynamicSerde {
    pub fn new() -> PyResult<Self> {
        let pickle_serde = PickleSerde::new()?;
        let int_serde = IntSerde {};
        let float_serde = FloatSerde {};
        let complex_serde = ComplexSerde {};
        let boolean_serde = BoolSerde {};
        let string_serde = StringSerde {};
        let bytes_serde = BytesSerde {};
        let numpy_i8_serde = NumpyDynamicShapeSerde::<i8> { dtype: PhantomData };
        let numpy_i16_serde = NumpyDynamicShapeSerde::<i16> { dtype: PhantomData };
        let numpy_i32_serde = NumpyDynamicShapeSerde::<i32> { dtype: PhantomData };
        let numpy_i64_serde = NumpyDynamicShapeSerde::<i64> { dtype: PhantomData };
        let numpy_u8_serde = NumpyDynamicShapeSerde::<u8> { dtype: PhantomData };
        let numpy_u16_serde = NumpyDynamicShapeSerde::<u16> { dtype: PhantomData };
        let numpy_u32_serde = NumpyDynamicShapeSerde::<u32> { dtype: PhantomData };
        let numpy_u64_serde = NumpyDynamicShapeSerde::<u64> { dtype: PhantomData };
        let numpy_f32_serde = NumpyDynamicShapeSerde::<f32> { dtype: PhantomData };
        let numpy_f64_serde = NumpyDynamicShapeSerde::<f64> { dtype: PhantomData };

        Ok(DynamicSerde {
            pickle_serde,
            int_serde,
            float_serde,
            complex_serde,
            boolean_serde,
            string_serde,
            bytes_serde,
            numpy_i8_serde,
            numpy_i16_serde,
            numpy_i32_serde,
            numpy_i64_serde,
            numpy_u8_serde,
            numpy_u16_serde,
            numpy_u32_serde,
            numpy_u64_serde,
            numpy_f32_serde,
            numpy_f64_serde,
        })
    }
}

impl PyAnySerde for DynamicSerde {
    fn append<'py>(
        &self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        let python_type = detect_python_type(obj)?;
        buf[offset] = get_python_type_byte(&python_type);
        let mut offset = offset + 1;
        match python_type {
            PythonType::BOOL => {
                offset = self.boolean_serde.append(buf, offset, obj)?;
            }
            PythonType::INT => {
                offset = self.int_serde.append(buf, offset, obj)?;
            }
            PythonType::FLOAT => {
                offset = self.float_serde.append(buf, offset, obj)?;
            }
            PythonType::COMPLEX => {
                offset = self.complex_serde.append(buf, offset, obj)?;
            }
            PythonType::STRING => {
                offset = self.string_serde.append(buf, offset, obj)?;
            }
            PythonType::BYTES => {
                offset = self.bytes_serde.append(buf, offset, obj)?;
            }
            PythonType::NUMPY { dtype } => match dtype {
                NumpyDtype::INT8 => {
                    offset = self.numpy_i8_serde.append(buf, offset, obj)?;
                }
                NumpyDtype::INT16 => {
                    offset = self.numpy_i16_serde.append(buf, offset, obj)?;
                }
                NumpyDtype::INT32 => {
                    offset = self.numpy_i32_serde.append(buf, offset, obj)?;
                }
                NumpyDtype::INT64 => {
                    offset = self.numpy_i64_serde.append(buf, offset, obj)?;
                }
                NumpyDtype::UINT8 => {
                    offset = self.numpy_u8_serde.append(buf, offset, obj)?;
                }
                NumpyDtype::UINT16 => {
                    offset = self.numpy_u16_serde.append(buf, offset, obj)?;
                }
                NumpyDtype::UINT32 => {
                    offset = self.numpy_u32_serde.append(buf, offset, obj)?;
                }
                NumpyDtype::UINT64 => {
                    offset = self.numpy_u64_serde.append(buf, offset, obj)?;
                }
                NumpyDtype::FLOAT32 => {
                    offset = self.numpy_f32_serde.append(buf, offset, obj)?;
                }
                NumpyDtype::FLOAT64 => {
                    offset = self.numpy_f64_serde.append(buf, offset, obj)?;
                }
            },
            PythonType::LIST => {
                let list = obj.downcast::<PyList>()?;
                offset = append_usize(buf, offset, list.len());
                for item in list.iter() {
                    offset = self.append(buf, offset, &item)?;
                }
            }
            PythonType::SET => {
                let set = obj.downcast::<PySet>()?;
                offset = append_usize(buf, offset, set.len());
                for item in set.iter() {
                    offset = self.append(buf, offset, &item)?;
                }
            }
            PythonType::TUPLE => {
                let tuple = obj.downcast::<PyTuple>()?;
                offset = append_usize(buf, offset, tuple.len());
                for item in tuple.iter() {
                    offset = self.append(buf, offset, &item)?;
                }
            }
            PythonType::DICT => {
                let dict = obj.downcast::<PyDict>()?;
                offset = append_usize(buf, offset, dict.len());
                for (key, value) in dict.iter() {
                    offset = self.append(buf, offset, &key)?;
                    offset = self.append(buf, offset, &value)?;
                }
            }
            PythonType::OTHER => {
                offset = self.pickle_serde.append(buf, offset, obj)?;
            }
        };
        Ok(offset)
    }

    fn retrieve<'py>(
        &self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let (python_type, mut offset) = retrieve_python_type(buf, offset)?;
        let obj;
        match python_type {
            PythonType::BOOL => {
                (obj, offset) = self.boolean_serde.retrieve(py, buf, offset)?;
            }
            PythonType::INT => {
                (obj, offset) = self.int_serde.retrieve(py, buf, offset)?;
            }
            PythonType::FLOAT => {
                (obj, offset) = self.float_serde.retrieve(py, buf, offset)?;
            }
            PythonType::COMPLEX => {
                (obj, offset) = self.complex_serde.retrieve(py, buf, offset)?;
            }
            PythonType::STRING => {
                (obj, offset) = self.string_serde.retrieve(py, buf, offset)?;
            }
            PythonType::BYTES => {
                (obj, offset) = self.bytes_serde.retrieve(py, buf, offset)?;
            }
            PythonType::NUMPY { dtype } => match dtype {
                NumpyDtype::INT8 => {
                    (obj, offset) = self.numpy_i8_serde.retrieve(py, buf, offset)?;
                }
                NumpyDtype::INT16 => {
                    (obj, offset) = self.numpy_i16_serde.retrieve(py, buf, offset)?;
                }
                NumpyDtype::INT32 => {
                    (obj, offset) = self.numpy_i32_serde.retrieve(py, buf, offset)?;
                }
                NumpyDtype::INT64 => {
                    (obj, offset) = self.numpy_i64_serde.retrieve(py, buf, offset)?;
                }
                NumpyDtype::UINT8 => {
                    (obj, offset) = self.numpy_u8_serde.retrieve(py, buf, offset)?;
                }
                NumpyDtype::UINT16 => {
                    (obj, offset) = self.numpy_u16_serde.retrieve(py, buf, offset)?;
                }
                NumpyDtype::UINT32 => {
                    (obj, offset) = self.numpy_u32_serde.retrieve(py, buf, offset)?;
                }
                NumpyDtype::UINT64 => {
                    (obj, offset) = self.numpy_u64_serde.retrieve(py, buf, offset)?;
                }
                NumpyDtype::FLOAT32 => {
                    (obj, offset) = self.numpy_f32_serde.retrieve(py, buf, offset)?;
                }
                NumpyDtype::FLOAT64 => {
                    (obj, offset) = self.numpy_f64_serde.retrieve(py, buf, offset)?;
                }
            },
            PythonType::LIST => {
                let list = PyList::empty(py);
                let n_items;
                (n_items, offset) = retrieve_usize(buf, offset)?;
                for _ in 0..n_items {
                    let item;
                    (item, offset) = self.retrieve(py, buf, offset)?;
                    list.append(item)?;
                }
                obj = list.into_any();
            }
            PythonType::SET => {
                let set = PySet::empty(py)?;
                let n_items;
                (n_items, offset) = retrieve_usize(buf, offset)?;
                for _ in 0..n_items {
                    let item;
                    (item, offset) = self.retrieve(py, buf, offset)?;
                    set.add(item)?;
                }
                obj = set.into_any();
            }
            PythonType::TUPLE => {
                let n_items;
                (n_items, offset) = retrieve_usize(buf, offset)?;
                let mut tuple_vec = Vec::with_capacity(n_items);
                for _ in 0..n_items {
                    let item;
                    (item, offset) = self.retrieve(py, buf, offset)?;
                    tuple_vec.push(item);
                }
                obj = PyTuple::new(py, tuple_vec)?.into_any();
            }
            PythonType::DICT => {
                let dict = PyDict::new(py);
                let n_items;
                (n_items, offset) = retrieve_usize(buf, offset)?;
                for _ in 0..n_items {
                    let key;
                    (key, offset) = self.retrieve(py, buf, offset)?;
                    let value;
                    (value, offset) = self.retrieve(py, buf, offset)?;
                    dict.set_item(key, value)?;
                }
                obj = dict.into_any();
            }
            PythonType::OTHER => {
                (obj, offset) = self.pickle_serde.retrieve(py, buf, offset)?;
            }
        };
        Ok((obj, offset))
    }
}
