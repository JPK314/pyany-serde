use pyo3::prelude::*;

use crate::{
    communication::{append_i64, retrieve_i64},
    PyAnySerde,
};

#[derive(Clone)]
pub struct IntSerde {}

impl PyAnySerde for IntSerde {
    fn append<'py>(
        &self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        Ok(append_i64(buf, offset, obj.extract::<i64>()?))
    }

    fn retrieve<'py>(
        &self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let (val, offset) = retrieve_i64(buf, offset)?;
        Ok((val.into_pyobject(py)?.into_any(), offset))
    }

    unsafe fn retrieve_ptr(&self, buf: &[u8], offset: usize) -> PyResult<(*mut u8, usize)> {
        let (val, offset) = retrieve_i64(buf, offset)?;
        Ok((Box::into_raw(Box::new(val)) as *mut u8, offset))
    }

    unsafe fn retrieve_from_ptr<'py>(
        &self,
        py: Python<'py>,
        ptr: *mut u8,
    ) -> PyResult<Bound<'py, PyAny>> {
        let val = *Box::from_raw(ptr as *mut i64);
        Ok(val.into_pyobject(py)?.into_any())
    }
}
