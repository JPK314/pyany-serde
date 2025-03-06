use pyo3::prelude::*;

use crate::{
    communication::{append_f64, retrieve_f64},
    PyAnySerde,
};

#[derive(Clone)]
pub struct FloatSerde {}

impl PyAnySerde for FloatSerde {
    fn append<'py>(
        &self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        Ok(append_f64(buf, offset, obj.extract::<f64>()?))
    }

    fn retrieve<'py>(
        &self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let (val, offset) = retrieve_f64(buf, offset)?;
        Ok((val.into_pyobject(py)?.into_any(), offset))
    }

    unsafe fn retrieve_ptr(&self, buf: &[u8], offset: usize) -> PyResult<(*mut u8, usize)> {
        // println!("float retrieve_ptr");
        let (val, offset) = retrieve_f64(buf, offset)?;
        Ok((Box::into_raw(Box::new(val)) as *mut u8, offset))
    }

    unsafe fn retrieve_from_ptr<'py>(
        &self,
        py: Python<'py>,
        ptr: *mut u8,
    ) -> PyResult<Bound<'py, PyAny>> {
        let val = *Box::from_raw(ptr as *mut f64);
        Ok(val.into_pyobject(py)?.into_any())
    }
}
