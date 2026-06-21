use pyo3::prelude::*;

use crate::{pyany_serde_impl::InitStrategy, tests::common::run_python_test_file};

#[pyfunction]
fn validate_all(v: InitStrategy) -> PyResult<()> {
    assert_eq!(v, InitStrategy::ALL {});
    Ok(())
}
#[pyfunction]
fn validate_some(v: InitStrategy) -> PyResult<()> {
    assert_eq!(
        v,
        InitStrategy::SOME {
            kwargs: vec!["a".to_string(), "b".to_string()],
        }
    );
    Ok(())
}
#[pyfunction]
fn validate_none(v: InitStrategy) -> PyResult<()> {
    assert_eq!(v, InitStrategy::NONE {});
    Ok(())
}

fn tests_submod<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyModule>> {
    let sub = PyModule::new(py, "pydantic_init_strategy_tests")?;
    sub.add_function(wrap_pyfunction!(validate_all, py)?)?;
    sub.add_function(wrap_pyfunction!(validate_some, py)?)?;
    sub.add_function(wrap_pyfunction!(validate_none, py)?)?;
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
