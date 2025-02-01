use pyo3::prelude::*;
use pyo3::types::PyNone;

use crate::{
    communication::{append_bool, append_python, retrieve_bool, retrieve_python},
    pyany_serde::PyAnySerde,
};

use crate::pyany_serde_type::PyAnySerdeType;

#[derive(Clone)]
pub struct OptionSerde {
    value_serde_option: Option<Box<dyn PyAnySerde>>,
    serde_enum: PyAnySerdeType,
    serde_enum_bytes: Vec<u8>,
}

impl OptionSerde {
    pub fn new(value_serde_option: Option<Box<dyn PyAnySerde>>) -> Self {
        let value_serde_enum: PyAnySerdeType = match &value_serde_option {
            Some(pyany_serde) => pyany_serde.get_enum().clone(),
            None => PyAnySerdeType::OTHER,
        };
        let serde_enum = PyAnySerdeType::OPTION {
            value: Box::new(value_serde_enum),
        };
        OptionSerde {
            value_serde_option,
            serde_enum_bytes: serde_enum.serialize(),
            serde_enum,
        }
    }
}

impl PyAnySerde for OptionSerde {
    fn append<'py>(
        &mut self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        let mut offset = offset;
        if obj.is_none() {
            offset = append_bool(buf, offset, false);
        } else {
            offset = append_bool(buf, offset, true);
            offset = append_python(buf, offset, obj, &mut self.value_serde_option)?;
        }
        Ok(offset)
    }

    fn retrieve<'py>(
        &mut self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let (is_some, offset) = retrieve_bool(buf, offset)?;
        if is_some {
            retrieve_python(py, buf, offset, &mut self.value_serde_option)
        } else {
            Ok((PyNone::get(py).to_owned().into_any(), offset))
        }
    }

    fn get_enum(&self) -> &PyAnySerdeType {
        &self.serde_enum
    }

    fn get_enum_bytes(&self) -> &[u8] {
        &self.serde_enum_bytes
    }
}
