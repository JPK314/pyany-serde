use pyo3::prelude::*;

use crate::{
    communication::{append_f64, append_f64_vec, retrieve_f64},
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

    fn append_vec<'py>(
        &mut self,
        v: &mut Vec<u8>,
        _start_addr: Option<usize>,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<()> {
        append_f64_vec(v, obj.extract::<f64>()?);
        Ok(())
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
