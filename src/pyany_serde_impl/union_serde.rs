use pyo3::exceptions::asyncio::InvalidStateError;
use pyo3::prelude::*;
use pyo3::types::PyFunction;

use crate::{
    communication::{append_python, append_usize, retrieve_python, retrieve_usize},
    pyany_serde::PyAnySerde,
};

use crate::pyany_serde_type::PyAnySerdeType;

#[derive(Clone)]
pub struct UnionSerde {
    serde_options: Vec<Option<Box<dyn PyAnySerde>>>,
    serde_choice_fn: Py<PyFunction>,
    serde_enum: PyAnySerdeType,
    serde_enum_bytes: Vec<u8>,
}

impl UnionSerde {
    pub fn new(
        serde_options: Vec<Option<Box<dyn PyAnySerde>>>,
        serde_choice_fn: Py<PyFunction>,
    ) -> Self {
        UnionSerde {
            serde_options,
            serde_choice_fn,
            serde_enum: PyAnySerdeType::OTHER,
            serde_enum_bytes: PyAnySerdeType::OTHER.serialize(),
        }
    }
}

impl PyAnySerde for UnionSerde {
    fn append<'py>(
        &mut self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        let serde_idx = self
            .serde_choice_fn
            .bind(obj.py())
            .call1((obj,))?
            .extract::<usize>()?;
        let offset = append_usize(buf, offset, serde_idx);
        let serde_option = self.serde_options.get_mut(serde_idx).ok_or_else(|| {
            InvalidStateError::new_err(format!(
                "Serde choice function returned {} which is not a valid choice index",
                serde_idx
            ))
        })?;
        append_python(buf, offset, obj, serde_option)
    }

    fn retrieve<'py>(
        &mut self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let (serde_idx, offset) = retrieve_usize(buf, offset)?;
        let serde_option = self.serde_options.get_mut(serde_idx).ok_or_else(|| {
            InvalidStateError::new_err(format!(
                "Deserialized serde idx {} which is not a valid choice index",
                serde_idx
            ))
        })?;
        retrieve_python(py, buf, offset, serde_option)
    }

    fn get_enum(&self) -> &PyAnySerdeType {
        &self.serde_enum
    }

    fn get_enum_bytes(&self) -> &[u8] {
        &self.serde_enum_bytes
    }
}
