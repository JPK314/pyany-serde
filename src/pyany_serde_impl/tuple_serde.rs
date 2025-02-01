use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::{
    communication::{append_python, retrieve_python},
    pyany_serde::PyAnySerde,
};

use crate::pyany_serde_type::PyAnySerdeType;

#[derive(Clone)]
pub struct TupleSerde {
    item_serdes: Vec<Option<Box<dyn PyAnySerde>>>,
    serde_enum: PyAnySerdeType,
    serde_enum_bytes: Vec<u8>,
}

impl TupleSerde {
    pub fn new(item_serdes: Vec<Option<Box<dyn PyAnySerde>>>) -> PyResult<Self> {
        let mut item_serde_enums_option = Some(Vec::with_capacity(item_serdes.len()));
        for serde_option in item_serdes.iter() {
            match serde_option {
                Some(pyany_serde) => {
                    item_serde_enums_option
                        .as_mut()
                        .unwrap()
                        .push(pyany_serde.get_enum().clone());
                }
                None => {
                    item_serde_enums_option = None;
                    break;
                }
            }
        }
        let serde_enum = if let Some(items) = item_serde_enums_option {
            PyAnySerdeType::TUPLE { items }
        } else {
            PyAnySerdeType::OTHER
        };
        Ok(TupleSerde {
            item_serdes,
            serde_enum_bytes: serde_enum.serialize(),
            serde_enum,
        })
    }
}

impl PyAnySerde for TupleSerde {
    fn append<'py>(
        &mut self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        let tuple = obj.downcast::<PyTuple>()?;
        let mut offset = offset;
        for (serde_option, item) in self.item_serdes.iter_mut().zip(tuple.iter()) {
            offset = append_python(buf, offset, &item, serde_option)?;
        }
        Ok(offset)
    }

    fn retrieve<'py>(
        &mut self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let mut tuple_vec = Vec::with_capacity(self.item_serdes.len());
        let mut offset = offset;
        for serde_option in self.item_serdes.iter_mut() {
            let item;
            (item, offset) = retrieve_python(py, buf, offset, serde_option)?;
            tuple_vec.push(item);
        }
        Ok((PyTuple::new(py, tuple_vec)?.into_any(), offset))
    }

    fn get_enum(&self) -> &PyAnySerdeType {
        &self.serde_enum
    }

    fn get_enum_bytes(&self) -> &[u8] {
        &self.serde_enum_bytes
    }
}
