use pyo3::exceptions::asyncio::InvalidStateError;
use pyo3::prelude::*;
use pyo3::types::PyFunction;

use crate::{
    communication::{append_usize, append_usize_vec, retrieve_usize},
    PyAnySerde,
};

#[derive(Clone)]
pub struct UnionSerde {
    pub option_serdes: Vec<Box<dyn PyAnySerde>>,
    pub option_choice_fn: Py<PyFunction>,
}

impl PyAnySerde for UnionSerde {
    fn append<'py>(
        &mut self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        let serde_idx = self
            .option_choice_fn
            .bind(obj.py())
            .call1((obj,))?
            .extract::<usize>()?;
        let offset = append_usize(buf, offset, serde_idx);
        let pyany_serde = self.option_serdes.get_mut(serde_idx).ok_or_else(|| {
            InvalidStateError::new_err(format!(
                "Serde choice function returned {} which is not a valid choice index",
                serde_idx
            ))
        })?;
        pyany_serde.append(buf, offset, obj)
    }

    fn append_vec<'py>(
        &mut self,
        v: &mut Vec<u8>,
        start_addr: Option<usize>,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<()> {
        let serde_idx = self
            .option_choice_fn
            .bind(obj.py())
            .call1((obj,))?
            .extract::<usize>()?;
        append_usize_vec(v, serde_idx);
        let pyany_serde = self.option_serdes.get_mut(serde_idx).ok_or_else(|| {
            InvalidStateError::new_err(format!(
                "Serde choice function returned {} which is not a valid choice index",
                serde_idx
            ))
        })?;
        pyany_serde.append_vec(v, start_addr, obj)
    }

    fn retrieve<'py>(
        &mut self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let (serde_idx, offset) = retrieve_usize(buf, offset)?;
        let pyany_serde = self.option_serdes.get_mut(serde_idx).ok_or_else(|| {
            InvalidStateError::new_err(format!(
                "Deserialized serde idx {} which is not a valid choice index",
                serde_idx
            ))
        })?;
        pyany_serde.retrieve(py, buf, offset)
    }
}
