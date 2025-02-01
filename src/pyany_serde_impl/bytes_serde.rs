use pyo3::prelude::*;
use pyo3::types::PyBytes;

use crate::{
    communication::{append_bytes, retrieve_bytes},
    pyany_serde::PyAnySerde,
};

use crate::pyany_serde_type::PyAnySerdeType;

#[derive(Clone)]
pub struct BytesSerde {
    serde_enum: PyAnySerdeType,
    serde_enum_bytes: Vec<u8>,
}

impl BytesSerde {
    pub fn new() -> Self {
        BytesSerde {
            serde_enum: PyAnySerdeType::BYTES,
            serde_enum_bytes: PyAnySerdeType::BYTES.serialize(),
        }
    }
}

impl PyAnySerde for BytesSerde {
    fn append<'py>(
        &mut self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        append_bytes(buf, offset, obj.downcast::<PyBytes>()?.as_bytes())
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

    fn get_enum(&self) -> &PyAnySerdeType {
        &self.serde_enum
    }

    fn get_enum_bytes(&self) -> &[u8] {
        &self.serde_enum_bytes
    }
}
