use pyo3::prelude::*;
use pyo3::types::PySet;

use crate::{
    communication::{append_usize, append_usize_vec, retrieve_usize},
    PyAnySerde,
};

#[derive(Clone)]
pub struct SetSerde {
    pub items_serde: Box<dyn PyAnySerde>,
}

impl PyAnySerde for SetSerde {
    fn append<'py>(
        &mut self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        let set = obj.downcast::<PySet>()?;
        let mut offset = append_usize(buf, offset, set.len());
        for item in set.iter() {
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
        let set = obj.downcast::<PySet>()?;
        append_usize_vec(v, set.len());
        for item in set.iter() {
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
        let set = PySet::empty(py)?;
        let (n_items, mut offset) = retrieve_usize(buf, offset)?;
        for _ in 0..n_items {
            let item;
            (item, offset) = self.items_serde.retrieve(py, buf, offset)?;
            set.add(item)?;
        }
        Ok((set.into_any(), offset))
    }
}
