use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::PyAnySerde;

#[derive(Clone)]
pub struct TupleSerde {
    pub item_serdes: Vec<Box<dyn PyAnySerde>>,
}

impl PyAnySerde for TupleSerde {
    fn append<'py>(
        &self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        let tuple = obj.downcast::<PyTuple>()?;
        let mut offset = offset;
        for (pyany_serde, item) in self.item_serdes.iter().zip(tuple.iter()) {
            offset = pyany_serde.append(buf, offset, &item)?;
        }
        Ok(offset)
    }

    fn retrieve<'py>(
        &self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let mut tuple_vec = Vec::with_capacity(self.item_serdes.len());
        let mut offset = offset;
        for pyany_serde in self.item_serdes.iter() {
            let item;
            (item, offset) = pyany_serde.retrieve(py, buf, offset)?;
            tuple_vec.push(item);
        }
        Ok((PyTuple::new(py, tuple_vec)?.into_any(), offset))
    }

    unsafe fn retrieve_ptr(&self, buf: &[u8], mut offset: usize) -> PyResult<(*mut u8, usize)> {
        // println!("tuple retrieve_ptr");
        let mut tuple_ptr_vec = Vec::with_capacity(self.item_serdes.len());
        for pyany_serde in self.item_serdes.iter() {
            let ptr;
            (ptr, offset) = pyany_serde.retrieve_ptr(buf, offset)?;
            tuple_ptr_vec.push(ptr)
        }
        Ok((Box::into_raw(Box::new(tuple_ptr_vec)) as *mut u8, offset))
    }

    unsafe fn retrieve_from_ptr<'py>(
        &self,
        py: Python<'py>,
        ptr: *mut u8,
    ) -> PyResult<Bound<'py, PyAny>> {
        let tuple_ptr_vec = *Box::from_raw(ptr as *mut Vec<*mut u8>);
        let tuple_vec = tuple_ptr_vec
            .into_iter()
            .zip(self.item_serdes.iter())
            .map(|(item_ptr, item_serde)| item_serde.retrieve_from_ptr(py, item_ptr))
            .collect::<PyResult<Vec<_>>>()?;
        Ok(PyTuple::new(py, tuple_vec)?.into_any())
    }
}
