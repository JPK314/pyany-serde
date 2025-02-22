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
        Ok((val.into_pyobject(py)?.to_owned().into_any(), offset))
    }
}
