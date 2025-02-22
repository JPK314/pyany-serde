use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::{
    communication::{append_usize, retrieve_usize},
    PyAnySerde,
};

#[derive(Clone)]
pub struct DictSerde {
    pub keys_serde: Box<dyn PyAnySerde>,
    pub values_serde: Box<dyn PyAnySerde>,
}

impl PyAnySerde for DictSerde {
    fn append<'py>(
        &self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        let dict = obj.downcast::<PyDict>()?;
        let mut offset = append_usize(buf, offset, dict.len());
        for (key, value) in dict.iter() {
            offset = self.keys_serde.append(buf, offset, &key)?;
            offset = self.values_serde.append(buf, offset, &value)?;
        }
        Ok(offset)
    }

    fn retrieve<'py>(
        &self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let dict = PyDict::new(py);
        let (n_items, mut offset) = retrieve_usize(buf, offset)?;
        for _ in 0..n_items {
            let key;
            (key, offset) = self.keys_serde.retrieve(py, buf, offset)?;
            let value;
            (value, offset) = self.values_serde.retrieve(py, buf, offset)?;
            dict.set_item(key, value)?;
        }
        Ok((dict.into_any(), offset))
    }
}
