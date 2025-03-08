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
        &mut self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        let complex = obj.downcast::<PyComplex>()?;
        let mut offset = append_c_double(buf, offset, complex.real());
        offset = append_c_double(buf, offset, complex.imag());
        Ok(offset)
    }

    fn retrieve<'py>(
        &mut self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let (real, mut offset) = retrieve_c_double(buf, offset)?;
        let imag;
        (imag, offset) = retrieve_c_double(buf, offset)?;
        Ok((PyComplex::from_doubles(py, real, imag).into_any(), offset))
    }
}
