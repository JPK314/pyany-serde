use pyo3::prelude::*;

pub mod common;
pub mod communication;
pub mod dyn_pyany_serde;
pub mod dyn_pyany_serde_factory;
pub mod pyany_serde;
pub mod pyany_serde_impl;
pub mod pyany_serde_type;

#[pymodule]
#[pyo3(name = "pyany_serde")]
fn define_pyany_serde_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // m.add_function(wrap_pyfunction!(env_process::env_process, m)?)?;
    m.add_class::<dyn_pyany_serde_factory::DynPyAnySerdeFactory>()?;
    m.add_class::<dyn_pyany_serde::DynPyAnySerde>()?;
    Ok(())
}
