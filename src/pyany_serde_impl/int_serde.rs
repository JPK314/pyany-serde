use pyo3::prelude::*;

use crate::{
    communication::{append_i64, retrieve_i64},
    pyany_serde::PyAnySerde,
};

use crate::pyany_serde_type::PyAnySerdeType;

#[derive(Clone)]
pub struct IntSerde {
    serde_enum: PyAnySerdeType,
    serde_enum_bytes: Vec<u8>,
}

impl IntSerde {
    pub fn new() -> Self {
        IntSerde {
            serde_enum_bytes: PyAnySerdeType::INT.serialize(),
            serde_enum: PyAnySerdeType::INT,
        }
    }
}

impl PyAnySerde for IntSerde {
    fn append<'py>(
        &mut self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        Ok(append_i64(buf, offset, obj.extract::<i64>()?))
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

    fn get_enum(&self) -> &PyAnySerdeType {
        &self.serde_enum
    }

    fn get_enum_bytes(&self) -> &[u8] {
        &self.serde_enum_bytes
    }
}
