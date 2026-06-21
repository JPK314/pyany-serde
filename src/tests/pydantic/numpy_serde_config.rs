use std::env;

use pyo3::prelude::*;

use crate::{
    pyany_serde_impl::NumpySerdeConfig,
    tests::{run_python_test_file, validate_fn_eq},
};

#[pyfunction]
fn validate_dynamic<'py>(
    py: Python<'py>,
    v: NumpySerdeConfig,
    preprocessor_fn: Option<Bound<'py, PyAny>>,
    postprocessor_fn: Option<Bound<'py, PyAny>>,
) -> PyResult<()> {
    let NumpySerdeConfig::DYNAMIC {
        preprocessor_fn: v_preprocessor_fn,
        postprocessor_fn: v_postprocessor_fn,
    } = v
    else {
        panic!("Expected v to be of type NumpySerdeConfig::DYNAMIC")
    };
    validate_fn_eq(
        v_preprocessor_fn.map(|val| val.into_bound(py)).as_ref(),
        preprocessor_fn.as_ref(),
        "preprocessor_fn",
    )?;
    validate_fn_eq(
        v_postprocessor_fn.map(|val| val.into_bound(py)).as_ref(),
        postprocessor_fn.as_ref(),
        "postprocessor_fn",
    )?;
    Ok(())
}

#[pyfunction]
fn validate_static<'py>(
    py: Python<'py>,
    v: NumpySerdeConfig,
    preprocessor_fn: Option<Bound<'py, PyAny>>,
    postprocessor_fn: Option<Bound<'py, PyAny>>,
) -> PyResult<()> {
    let NumpySerdeConfig::STATIC {
        shape,
        preprocessor_fn: v_preprocessor_fn,
        postprocessor_fn: v_postprocessor_fn,
        allocation_pool_min_size,
        allocation_pool_max_size,
        allocation_pool_warning_size,
    } = v
    else {
        panic!("Expected v to be of type NumpySerdeConfig::STATIC")
    };
    validate_fn_eq(
        v_preprocessor_fn.map(|val| val.into_bound(py)).as_ref(),
        preprocessor_fn.as_ref(),
        "preprocessor_fn",
    )?;
    validate_fn_eq(
        v_postprocessor_fn.map(|val| val.into_bound(py)).as_ref(),
        postprocessor_fn.as_ref(),
        "postprocessor_fn",
    )?;
    assert_eq!(shape, [2]);
    assert_eq!(allocation_pool_min_size, 0);
    assert_eq!(allocation_pool_max_size, Some(10));
    assert_eq!(allocation_pool_warning_size, Some(1));
    Ok(())
}

fn tests_submod<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyModule>> {
    let sub = PyModule::new(py, "pydantic_numpy_serde_config_tests")?;
    sub.add_function(wrap_pyfunction!(validate_dynamic, py)?)?;
    sub.add_function(wrap_pyfunction!(validate_static, py)?)?;
    Ok(sub)
}

#[test]
fn run_pydantic_tests() -> PyResult<()> {
    env::set_var("PYANY_SERDE_UNPICKLE_WITHOUT_PROMPT", "1");
    Python::initialize();
    Python::attach(|py| {
        run_python_test_file(
            py,
            "python/tests/pydantic/numpy_serde_config.py",
            tests_submod(py)?,
        )
    })
}
