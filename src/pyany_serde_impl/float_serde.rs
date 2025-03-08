use pyo3::prelude::*;

use crate::{
    communication::{append_f64, retrieve_f64},
    PyAnySerde,
};

#[derive(Clone)]
pub struct FloatSerde {}

impl PyAnySerde for FloatSerde {
    fn append<'py>(
        &mut self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        Ok(append_f64(buf, offset, obj.extract::<f64>()?))
    }

    fn retrieve<'py>(
        &mut self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let (val, offset) = retrieve_f64(buf, offset)?;
        Ok((val.into_pyobject(py)?.into_any(), offset))
    }
}
