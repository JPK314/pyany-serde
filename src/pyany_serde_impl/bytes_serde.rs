use pyo3::prelude::*;
use pyo3::types::PyBytes;

use crate::{
    communication::{append_bytes, append_bytes_vec, retrieve_bytes},
    PyAnySerde,
};

#[derive(Clone)]
pub struct BytesSerde {}

impl PyAnySerde for BytesSerde {
    fn append<'py>(
        &mut self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        Ok(append_bytes(
            buf,
            offset,
            obj.downcast::<PyBytes>()?.as_bytes(),
        ))
    }

    fn append_vec<'py>(
        &mut self,
        v: &mut Vec<u8>,
        _start_addr: Option<usize>,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<()> {
        append_bytes_vec(v, obj.downcast::<PyBytes>()?.as_bytes());
        Ok(())
    }

    fn retrieve<'py>(
        &mut self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let (obj_bytes, offset) = retrieve_bytes(buf, offset)?;
        Ok((PyBytes::new(py, obj_bytes).into_any(), offset))
    }
}
