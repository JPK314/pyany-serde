use pyo3::types::PyBytes;
use pyo3::{intern, prelude::*};

use crate::{
    communication::{append_bytes, retrieve_bytes},
    PyAnySerde,
};

#[derive(Clone)]
pub struct PythonSerdeSerde {
    pub python_serde: PyObject,
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
}
