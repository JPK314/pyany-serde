use core::str;
use pyo3::prelude::*;
use pyo3::types::PyString;

use crate::{
    communication::{append_bytes, retrieve_bytes},
    PyAnySerde,
};

#[derive(Clone)]
pub struct StringSerde {}

impl PyAnySerde for StringSerde {
    fn append<'py>(
        &self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        append_bytes(
            buf,
            offset,
            obj.downcast::<PyString>()?.to_str()?.as_bytes(),
        )
    }

    fn retrieve<'py>(
        &self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let (obj_bytes, offset) = retrieve_bytes(buf, offset)?;
        Ok((
            PyString::new(py, str::from_utf8(obj_bytes)?).into_any(),
            offset,
        ))
    }
}
