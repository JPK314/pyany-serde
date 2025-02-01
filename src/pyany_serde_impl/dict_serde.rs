use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::{
    communication::{append_python, append_usize, retrieve_python, retrieve_usize},
    pyany_serde::PyAnySerde,
};

use crate::pyany_serde_type::PyAnySerdeType;

#[derive(Clone)]
pub struct DictSerde {
    key_serde_option: Option<Box<dyn PyAnySerde>>,
    value_serde_option: Option<Box<dyn PyAnySerde>>,
    serde_enum: PyAnySerdeType,
    serde_enum_bytes: Vec<u8>,
}

impl DictSerde {
    pub fn new(
        key_serde_option: Option<Box<dyn PyAnySerde>>,
        value_serde_option: Option<Box<dyn PyAnySerde>>,
    ) -> Self {
        let key_serde_enum: PyAnySerdeType = match &key_serde_option {
            Some(pyany_serde) => pyany_serde.get_enum().clone(),
            None => PyAnySerdeType::OTHER,
        };
        let value_serde_enum: PyAnySerdeType = match &value_serde_option {
            Some(pyany_serde) => pyany_serde.get_enum().clone(),
            None => PyAnySerdeType::OTHER,
        };
        let serde_enum = PyAnySerdeType::DICT {
            keys: Box::new(key_serde_enum),
            values: Box::new(value_serde_enum),
        };
        DictSerde {
            key_serde_option,
            value_serde_option,
            serde_enum_bytes: serde_enum.serialize(),
            serde_enum,
        }
    }
}

impl PyAnySerde for DictSerde {
    fn append<'py>(
        &mut self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        let dict = obj.downcast::<PyDict>()?;
        let mut offset = append_usize(buf, offset, dict.len());
        for (key, value) in dict.iter() {
            offset = append_python(buf, offset, &key, &mut self.key_serde_option)?;
            offset = append_python(buf, offset, &value, &mut self.value_serde_option)?;
        }
        Ok(offset)
    }

    fn retrieve<'py>(
        &mut self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let dict = PyDict::new(py);
        let (n_items, mut offset) = retrieve_usize(buf, offset)?;
        for _ in 0..n_items {
            let key;
            (key, offset) = retrieve_python(py, buf, offset, &mut self.key_serde_option)?;
            let value;
            (value, offset) = retrieve_python(py, buf, offset, &mut self.value_serde_option)?;
            dict.set_item(key, value)?;
        }
        Ok((dict.into_any(), offset))
    }

    fn get_enum(&self) -> &PyAnySerdeType {
        &self.serde_enum
    }

    fn get_enum_bytes(&self) -> &[u8] {
        &self.serde_enum_bytes
    }
}
