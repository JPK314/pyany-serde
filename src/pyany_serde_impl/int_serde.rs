use pyo3::prelude::*;

use crate::{
    communication::{append_i64, append_i64_vec, retrieve_i64},
    PyAnySerde,
};

#[derive(Clone)]
pub struct IntSerde {}

impl PyAnySerde for IntSerde {
    fn append<'py>(
        &mut self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        Ok(append_i64(buf, offset, obj.extract::<i64>()?))
    }

    fn append_vec<'py>(
        &mut self,
        v: &mut Vec<u8>,
        _start_addr: Option<usize>,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<()> {
        append_i64_vec(v, obj.extract::<i64>()?);
        Ok(())
    }

    fn retrieve<'py>(
        &mut self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let (val, offset) = retrieve_i64(buf, offset)?;
        Ok((val.into_pyobject(py)?.to_owned().into_any(), offset))
    }
}
