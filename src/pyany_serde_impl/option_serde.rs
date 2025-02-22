use pyo3::prelude::*;
use pyo3::types::PyNone;

use crate::{
    communication::{append_bool, retrieve_bool},
    PyAnySerde,
};

#[derive(Clone)]
pub struct OptionSerde {
    pub value_serde: Box<dyn PyAnySerde>,
}

impl PyAnySerde for OptionSerde {
    fn append<'py>(
        &self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        let mut offset = offset;
        if obj.is_none() {
            offset = append_bool(buf, offset, false);
        } else {
            offset = append_bool(buf, offset, true);
            offset = self.value_serde.append(buf, offset, obj)?;
        }
        Ok(offset)
    }

    fn retrieve<'py>(
        &self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let (is_some, offset) = retrieve_bool(buf, offset)?;
        if is_some {
            self.value_serde.retrieve(py, buf, offset)
        } else {
            Ok((PyNone::get(py).to_owned().into_any(), offset))
        }
    }
}
