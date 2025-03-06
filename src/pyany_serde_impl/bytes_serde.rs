use pyo3::prelude::*;
use pyo3::types::PyBytes;

use crate::{
    communication::{append_bytes, retrieve_bytes},
    PyAnySerde,
};

#[derive(Clone)]
pub struct BytesSerde {}

impl PyAnySerde for BytesSerde {
    fn append<'py>(
        &self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        append_bytes(buf, offset, obj.downcast::<PyBytes>()?.as_bytes())
    }

    fn retrieve<'py>(
        &self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let (obj_bytes, offset) = retrieve_bytes(buf, offset)?;
        Ok((PyBytes::new(py, obj_bytes).into_any(), offset))
    }

    unsafe fn retrieve_ptr(&self, buf: &[u8], offset: usize) -> PyResult<(*mut u8, usize)> {
        let (obj_bytes, offset) = retrieve_bytes(buf, offset)?;
        Ok((Box::into_raw(Box::new(obj_bytes)) as *mut u8, offset))
    }

    unsafe fn retrieve_from_ptr<'py>(
        &self,
        py: Python<'py>,
        ptr: *mut u8,
    ) -> PyResult<Bound<'py, PyAny>> {
        let obj_bytes = *Box::from_raw(ptr as *mut &[u8]);
        Ok(PyBytes::new(py, obj_bytes).into_any())
    }
}
