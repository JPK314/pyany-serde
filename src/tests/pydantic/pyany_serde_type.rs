use std::env;

use pyo3::prelude::*;

use crate::tests::{run_python_test_file, validate_pyany_serde_type_eq};

fn tests_submod<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyModule>> {
    let sub = PyModule::new(py, "pydantic_pyany_serde_type_tests")?;
    sub.add_function(wrap_pyfunction!(validate_pyany_serde_type_eq, py)?)?;
    Ok(sub)
}

#[test]
fn run_pydantic_tests() -> PyResult<()> {
    unsafe {
        env::set_var("PYANY_SERDE_UNPICKLE_WITHOUT_PROMPT", "1");
    }
    Python::initialize();
    Python::attach(|py| {
        run_python_test_file(
            py,
            "python/tests/pydantic/pyany_serde_type.py",
            tests_submod(py)?,
        )
    })
}
