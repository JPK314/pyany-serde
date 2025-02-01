use pyo3::types::PyBytes;
use pyo3::{intern, prelude::*};

use crate::communication::{append_bytes, retrieve_bytes};
use crate::pyany_serde::PyAnySerde;

use crate::pyany_serde_type::PyAnySerdeType;

#[derive(Clone)]
pub struct PythonSerdeSerde {
    python_serde: PyObject,
    serde_enum: PyAnySerdeType,
    serde_enum_bytes: Vec<u8>,
}

impl PythonSerdeSerde {
    pub fn new(python_serde: PyObject) -> Self {
        PythonSerdeSerde {
            python_serde,
            serde_enum_bytes: PyAnySerdeType::OTHER.serialize(),
            serde_enum: PyAnySerdeType::OTHER,
        }
    }
}

impl PyAnySerde for PythonSerdeSerde {
    fn append<'py>(
        &mut self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        append_bytes(
            buf,
            offset,
            self.python_serde
                .bind(obj.py())
                .call_method1(intern!(obj.py(), "to_bytes"), (obj,))?
                .downcast::<PyBytes>()?
                .as_bytes(),
        )
    }

    fn retrieve<'py>(
        &mut self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let (obj_bytes, offset) = retrieve_bytes(buf, offset)?;
        let obj = self
            .python_serde
            .bind(py)
            .call_method1(intern!(py, "from_bytes"), (PyBytes::new(py, obj_bytes),))?;
        Ok((obj, offset))
    }

    fn get_enum(&self) -> &PyAnySerdeType {
        &self.serde_enum
    }

    fn get_enum_bytes(&self) -> &[u8] {
        &self.serde_enum_bytes
    }
}
