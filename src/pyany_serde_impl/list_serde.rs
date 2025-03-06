use pyo3::prelude::*;
use pyo3::types::PyList;

use crate::{
    communication::{append_usize, retrieve_usize},
    PyAnySerde,
};

#[derive(Clone)]
pub struct ListSerde {
    pub items_serde: Box<dyn PyAnySerde>,
}

impl PyAnySerde for ListSerde {
    fn append<'py>(
        &self,
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

    fn retrieve<'py>(
        &self,
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

    unsafe fn retrieve_ptr(&self, buf: &[u8], offset: usize) -> PyResult<(*mut u8, usize)> {
        todo!()
    }

    unsafe fn retrieve_from_ptr<'py>(
        &self,
        py: Python<'py>,
        ptr: *mut u8,
    ) -> PyResult<Bound<'py, PyAny>> {
        todo!()
    }
}
