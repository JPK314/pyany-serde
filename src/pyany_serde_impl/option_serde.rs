use pyo3::prelude::*;
use pyo3::types::PyNone;

use crate::{
    communication::{append_bool, append_bool_vec, retrieve_bool},
    PyAnySerde,
};

#[derive(Clone)]
pub struct OptionSerde {
    pub value_serde: Box<dyn PyAnySerde>,
}

impl PyAnySerde for OptionSerde {
    fn append<'py>(
        &mut self,
        buf: &mut [u8],
        mut offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        if obj.is_none() {
            offset = append_bool(buf, offset, false);
        } else {
            offset = append_bool(buf, offset, true);
            offset = self.value_serde.append(buf, offset, obj)?;
        }
        Ok(offset)
    }

    fn append_vec<'py>(
        &mut self,
        v: &mut Vec<u8>,
        start_addr: Option<usize>,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<()> {
        if obj.is_none() {
            append_bool_vec(v, false);
        } else {
            append_bool_vec(v, true);
            self.value_serde.append_vec(v, start_addr, obj)?;
        }
        Ok(())
    }

    fn retrieve<'py>(
        &mut self,
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
