use pyo3::prelude::*;

use crate::{
    communication::{append_bool, append_bool_vec, retrieve_bool},
    PyAnySerde,
};

#[derive(Clone)]
pub struct BoolSerde {}

impl PyAnySerde for BoolSerde {
    fn append<'py>(
        &mut self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        Ok(append_bool(buf, offset, obj.extract::<bool>()?))
    }

    fn append_vec<'py>(
        &mut self,
        v: &mut Vec<u8>,
        _start_addr: Option<usize>,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<()> {
        append_bool_vec(v, obj.extract::<bool>()?);
        Ok(())
    }

    fn retrieve<'py>(
        &mut self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let (val, offset) = retrieve_bool(buf, offset)?;
        Ok((val.into_pyobject(py)?.to_owned().into_any(), offset))
    }
}
