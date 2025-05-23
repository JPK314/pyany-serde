use pyo3::prelude::*;
use pyo3::types::PyList;

use crate::{
    communication::{append_usize, append_usize_vec, retrieve_usize},
    PyAnySerde,
};

#[derive(Clone)]
pub struct ListSerde {
    pub items_serde: Box<dyn PyAnySerde>,
}

impl PyAnySerde for ListSerde {
    fn append<'py>(
        &mut self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        let list = obj.downcast::<PyList>()?;
        let mut offset = append_usize(buf, offset, list.len());
        for item in list.iter() {
            offset = self.items_serde.append(buf, offset, &item)?;
        }
        Ok(offset)
    }

    fn append_vec<'py>(
        &mut self,
        v: &mut Vec<u8>,
        start_addr: Option<usize>,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<()> {
        let list = obj.downcast::<PyList>()?;
        append_usize_vec(v, list.len());
        for item in list.iter() {
            self.items_serde.append_vec(v, start_addr, &item)?;
        }
        Ok(())
    }

    fn retrieve<'py>(
        &mut self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let list = PyList::empty(py);
        let (n_items, mut offset) = retrieve_usize(buf, offset)?;
        for _ in 0..n_items {
            let item;
            (item, offset) = self.items_serde.retrieve(py, buf, offset)?;
            list.append(item)?;
        }
        Ok((list.into_any(), offset))
    }
}
