use pyo3::prelude::*;

use crate::{
    communication::{append_f64, retrieve_f64},
    pyany_serde::PyAnySerde,
};

use crate::pyany_serde_type::PyAnySerdeType;

#[derive(Clone)]
pub struct FloatSerde {
    serde_enum: PyAnySerdeType,
    serde_enum_bytes: Vec<u8>,
}

impl FloatSerde {
    pub fn new() -> Self {
        FloatSerde {
            serde_enum_bytes: PyAnySerdeType::FLOAT.serialize(),
            serde_enum: PyAnySerdeType::FLOAT,
        }
    }
}

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

    fn get_enum(&self) -> &PyAnySerdeType {
        &self.serde_enum
    }

    fn get_enum_bytes(&self) -> &[u8] {
        &self.serde_enum_bytes
    }
}
