#[cfg(feature = "cdylib")]
use crate::pyany_serde_type::retrieve_pyany_serde_type;
#[cfg(feature = "cdylib")]
use pyo3::prelude::*;

use crate::pyany_serde::PyAnySerde;

#[cfg_attr(feature = "cdylib", pyclass(module = "pyany_serde", unsendable))]
#[derive(Clone)]
pub struct DynPyAnySerde(pub Option<Box<dyn PyAnySerde>>);

#[cfg(feature = "cdylib")]
#[pymethods]
impl DynPyAnySerde {
    #[new]
    fn new() -> Self {
        DynPyAnySerde(None)
    }
    fn __getstate__(&self) -> Vec<u8> {
        self.0.as_ref().unwrap().get_enum_bytes().to_vec()
    }
    fn __setstate__(&mut self, state: Vec<u8>) -> PyResult<()> {
        let (serde_enum, _) = retrieve_pyany_serde_type(&state[..], 0)?;
        self.0 = Some(serde_enum.try_into()?);
        Ok(())
    }
}
