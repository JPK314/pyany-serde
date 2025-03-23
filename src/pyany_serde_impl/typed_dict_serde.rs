use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};

use crate::PyAnySerde;

#[derive(Clone)]
pub struct TypedDictSerde {
    pub serde_kv_list: Vec<(Py<PyString>, Box<dyn PyAnySerde>)>,
}

impl PyAnySerde for TypedDictSerde {
    fn append<'py>(
        &mut self,
        buf: &mut [u8],
        mut offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        for (key, pyany_serde) in self.serde_kv_list.iter_mut() {
            offset = pyany_serde.append(buf, offset, &obj.get_item(key.bind(obj.py()))?)?;
        }
        Ok(offset)
    }

    fn append_vec<'py>(
        &mut self,
        v: &mut Vec<u8>,
        start_addr: Option<usize>,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<()> {
        for (key, pyany_serde) in self.serde_kv_list.iter_mut() {
            pyany_serde.append_vec(v, start_addr, &obj.get_item(key.bind(obj.py()))?)?;
        }
        Ok(())
    }

    fn retrieve<'py>(
        &mut self,
        py: Python<'py>,
        buf: &[u8],
        mut offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let mut kv_list = Vec::with_capacity(self.serde_kv_list.len());
        for (key, pyany_serde) in self.serde_kv_list.iter_mut() {
            let item;
            (item, offset) = pyany_serde.retrieve(py, buf, offset)?;
            kv_list.push((key.clone_ref(py), item));
        }
        Ok((
            PyDict::from_sequence(&kv_list.into_pyobject(py)?)?.into_any(),
            offset,
        ))
    }
}
