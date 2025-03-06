use pyo3::prelude::*;
use pyo3::types::PyComplex;

use crate::{
    communication::{append_c_double, retrieve_c_double},
    PyAnySerde,
};

#[derive(Clone)]
pub struct ComplexSerde {}

impl PyAnySerde for ComplexSerde {
    fn append<'py>(
        &self,
        buf: &mut [u8],
        mut offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        let complex = obj.downcast::<PyComplex>()?;
        offset = append_c_double(buf, offset, complex.real());
        offset = append_c_double(buf, offset, complex.imag());
        Ok(offset)
    }

    fn retrieve<'py>(
        &self,
        py: Python<'py>,
        buf: &[u8],
        mut offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let real;
        (real, offset) = retrieve_c_double(buf, offset)?;
        let imag;
        (imag, offset) = retrieve_c_double(buf, offset)?;
        Ok((PyComplex::from_doubles(py, real, imag).into_any(), offset))
    }

    unsafe fn retrieve_ptr(&self, buf: &[u8], mut offset: usize) -> PyResult<(*mut u8, usize)> {
        let real;
        (real, offset) = retrieve_c_double(buf, offset)?;
        let imag;
        (imag, offset) = retrieve_c_double(buf, offset)?;
        Ok((Box::into_raw(Box::new((real, imag))) as *mut u8, offset))
    }

    unsafe fn retrieve_from_ptr<'py>(
        &self,
        py: Python<'py>,
        ptr: *mut u8,
    ) -> PyResult<Bound<'py, PyAny>> {
        let (real, imag) = *Box::from_raw(ptr as *mut (f64, f64));
        Ok(PyComplex::from_doubles(py, real, imag).into_any())
    }
}
