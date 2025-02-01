use core::str;
use pyo3::prelude::*;
use pyo3::types::PyString;

use crate::{
    communication::{append_bytes, retrieve_bytes},
    pyany_serde::PyAnySerde,
};

use crate::pyany_serde_type::PyAnySerdeType;

#[derive(Clone)]
pub struct StringSerde {
    serde_enum: PyAnySerdeType,
    serde_enum_bytes: Vec<u8>,
}

impl StringSerde {
    pub fn new() -> Self {
        StringSerde {
            serde_enum: PyAnySerdeType::STRING,
            serde_enum_bytes: PyAnySerdeType::STRING.serialize(),
        }
    }
}

impl PyAnySerde for StringSerde {
    fn append<'py>(
        &mut self,
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
        &mut self,
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

    fn get_enum(&self) -> &PyAnySerdeType {
        &self.serde_enum
    }

    fn get_enum_bytes(&self) -> &[u8] {
        &self.serde_enum_bytes
    }
}
