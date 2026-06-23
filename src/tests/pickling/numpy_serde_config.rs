use pyo3::prelude::*;

use crate::tests::{run_python_test_file, validate_numpy_serde_config_eq};

fn tests_submod<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyModule>> {
    let sub = PyModule::new(py, "pickling_numpy_serde_config_tests")?;
    sub.add_function(wrap_pyfunction!(validate_numpy_serde_config_eq, py)?)?;
    Ok(sub)
}

#[test]
fn run_pickling_tests() -> PyResult<()> {
    Python::initialize();
    Python::attach(|py| {
        run_python_test_file(
            py,
            "python/tests/pickling/numpy_serde_config.py",
            tests_submod(py)?,
        )
    })
}
