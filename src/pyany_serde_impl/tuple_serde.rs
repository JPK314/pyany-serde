use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::PyAnySerde;

#[derive(Clone)]
pub struct TupleSerde {
    pub item_serdes: Vec<Box<dyn PyAnySerde>>,
}

impl PyAnySerde for TupleSerde {
    fn append<'py>(
        &mut self,
        buf: &mut [u8],
        mut offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        let tuple = obj.downcast::<PyTuple>()?;
        for (pyany_serde, item) in self.item_serdes.iter_mut().zip(tuple.iter()) {
            offset = pyany_serde.append(buf, offset, &item)?;
        }
        Ok(offset)
    }

    fn append_vec<'py>(
        &mut self,
        v: &mut Vec<u8>,
        start_addr: Option<usize>,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<()> {
        let tuple = obj.downcast::<PyTuple>()?;
        for (pyany_serde, item) in self.item_serdes.iter_mut().zip(tuple.iter()) {
            pyany_serde.append_vec(v, start_addr, &item)?;
        }
        Ok(())
    }
    fn retrieve<'py>(
        &mut self,
        py: Python<'py>,
        buf: &[u8],
        mut offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let mut tuple_vec = Vec::with_capacity(self.item_serdes.len());
        for pyany_serde in self.item_serdes.iter_mut() {
            let item;
            (item, offset) = pyany_serde.retrieve(py, buf, offset)?;
            tuple_vec.push(item);
        }
        Ok((PyTuple::new(py, tuple_vec)?.into_any(), offset))
    }
}
