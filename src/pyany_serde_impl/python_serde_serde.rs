use pyo3::ffi::{PyBUF_READ, PyBUF_WRITE, PyMemoryView_FromMemory};
use pyo3::types::PyBytes;
use pyo3::{intern, prelude::*};
use std::os::raw::c_char;

use crate::PyAnySerde;

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
        let py = obj.py();
        let memory_view = unsafe {
            Bound::<'py, PyAny>::from_owned_ptr(
                py,
                PyMemoryView_FromMemory(
                    buf.as_mut_ptr() as *mut c_char,
                    buf.len().try_into().unwrap(),
                    PyBUF_WRITE,
                ),
            )
        };

        self.python_serde
            .bind(obj.py())
            .call_method1(intern!(py, "append"), (memory_view, offset, obj))?
            .extract()
    }

    fn append_vec<'py>(
        &mut self,
        v: &mut Vec<u8>,
        start_addr: Option<usize>,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<()> {
        v.extend_from_slice(
            self.python_serde
                .bind(obj.py())
                .call_method1(intern!(obj.py(), "get_bytes"), (start_addr, obj))?
                .downcast::<PyBytes>()?
                .as_bytes(),
        );
        Ok(())
    }

    fn retrieve<'py>(
        &mut self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let memory_view = unsafe {
            Bound::<'py, PyAny>::from_owned_ptr(
                py,
                PyMemoryView_FromMemory(
                    buf.as_ptr() as *mut c_char,
                    buf.len().try_into().unwrap(),
                    PyBUF_READ,
                ),
            )
        };
        self.python_serde
            .bind(py)
            .call_method1(intern!(py, "retrieve"), (memory_view, offset))?
            .extract()
    }
}
