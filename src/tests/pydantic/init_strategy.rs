use pyo3::prelude::*;

use crate::tests::{run_python_test_file, validate_init_strategy_eq};

fn tests_submod<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyModule>> {
    let sub = PyModule::new(py, "pydantic_init_strategy_tests")?;
    sub.add_function(wrap_pyfunction!(validate_init_strategy_eq, py)?)?;
    Ok(sub)
}

#[test]
fn run_pydantic_tests() -> PyResult<()> {
    Python::initialize();
    Python::attach(|py| {
        run_python_test_file(
            py,
            "python/tests/pydantic/init_strategy.py",
            tests_submod(py)?,
        )
    })
}
