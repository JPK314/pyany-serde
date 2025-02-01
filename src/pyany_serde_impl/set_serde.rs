use pyo3::prelude::*;
use pyo3::types::PySet;

use crate::{
    communication::{append_python, append_usize, retrieve_python, retrieve_usize},
    pyany_serde::PyAnySerde,
};

use crate::pyany_serde_type::PyAnySerdeType;

#[derive(Clone)]
pub struct SetSerde {
    item_serde_option: Option<Box<dyn PyAnySerde>>,
    serde_enum: PyAnySerdeType,
    serde_enum_bytes: Vec<u8>,
}

impl SetSerde {
    pub fn new(item_serde_option: Option<Box<dyn PyAnySerde>>) -> Self {
        let item_serde_enum: PyAnySerdeType = match &item_serde_option {
            Some(pyany_serde) => pyany_serde.get_enum().clone(),
            None => PyAnySerdeType::OTHER,
        };
        let serde_enum = PyAnySerdeType::SET {
            items: Box::new(item_serde_enum),
        };
        SetSerde {
            item_serde_option,
            serde_enum_bytes: serde_enum.serialize(),
            serde_enum,
        }
    }
}

impl PyAnySerde for SetSerde {
    fn append<'py>(
        &mut self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        let set = obj.downcast::<PySet>()?;
        let mut offset = append_usize(buf, offset, set.len());
        for item in set.iter() {
            offset = append_python(buf, offset, &item, &mut self.item_serde_option)?;
        }
        Ok(offset)
    }

    fn retrieve<'py>(
        &mut self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let set = PySet::empty(py)?;
        let (n_items, mut offset) = retrieve_usize(buf, offset)?;
        for _ in 0..n_items {
            let item;
            (item, offset) = retrieve_python(py, buf, offset, &mut self.item_serde_option)?;
            set.add(item)?;
        }
        Ok((set.into_any(), offset))
    }

    fn get_enum(&self) -> &PyAnySerdeType {
        &self.serde_enum
    }

    fn get_enum_bytes(&self) -> &[u8] {
        &self.serde_enum_bytes
    }
}
