use pyo3::prelude::*;

use crate::{
    communication::{append_bool, retrieve_bool},
    PyAnySerde,
};

#[derive(Clone)]
pub struct BoolSerde {}

impl PyAnySerde for BoolSerde {
    fn append<'py>(
        &self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        Ok(append_bool(buf, offset, obj.extract::<bool>()?))
    }

    fn retrieve<'py>(
        &self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let (val, offset) = retrieve_bool(buf, offset)?;
        Ok((val.into_pyobject(py)?.to_owned().into_any(), offset))
    }

    unsafe fn retrieve_ptr(&self, buf: &[u8], offset: usize) -> PyResult<(*mut u8, usize)> {
        let (val, offset) = retrieve_bool(buf, offset)?;
        Ok((Box::into_raw(Box::new(val)) as *mut u8, offset))
    }

    unsafe fn retrieve_from_ptr<'py>(
        &self,
        py: Python<'py>,
        ptr: *mut u8,
    ) -> PyResult<Bound<'py, PyAny>> {
        let val = *Box::from_raw(ptr as *mut bool);
        Ok(val.into_pyobject(py)?.to_owned().into_any())
    }
}
