use pyo3::prelude::*;

use crate::{
    pyany_serde_impl::NumpySerdeConfig,
    tests::{run_python_test_file, validate_fn_eq},
};

#[pyfunction]
fn validate_eq<'py>(
    py: Python<'py>,
    actual: NumpySerdeConfig,
    expected: NumpySerdeConfig,
) -> PyResult<()> {
    match actual.clone() {
        NumpySerdeConfig::DYNAMIC {
            preprocessor_fn: actual_preprocessor_fn,
            postprocessor_fn: actual_postprocessor_fn,
        } => {
            let NumpySerdeConfig::DYNAMIC {
                preprocessor_fn: expected_preprocessor_fn,
                postprocessor_fn: expected_postprocessor_fn,
            } = expected.clone()
            else {
                panic!("Expected {actual} to be NumpySerdeConfig::DYNAMIC {{..}}");
            };
            validate_fn_eq(
                actual_preprocessor_fn.map(|v| v.into_bound(py)).as_ref(),
                expected_preprocessor_fn.map(|v| v.into_bound(py)).as_ref(),
                "preprocessor_fn",
            )?;
            validate_fn_eq(
                actual_postprocessor_fn.map(|v| v.into_bound(py)).as_ref(),
                expected_postprocessor_fn.map(|v| v.into_bound(py)).as_ref(),
                "postprocessor_fn",
            )?;
        }
        NumpySerdeConfig::STATIC {
            shape: actual_shape,
            preprocessor_fn: actual_preprocessor_fn,
            postprocessor_fn: actual_postprocessor_fn,
            allocation_pool_min_size: actual_allocation_pool_min_size,
            allocation_pool_max_size: actual_allocation_pool_max_size,
            allocation_pool_warning_size: actual_allocation_pool_warning_size,
        } => {
            let NumpySerdeConfig::STATIC {
                shape: expected_shape,
                preprocessor_fn: expected_preprocessor_fn,
                postprocessor_fn: expected_postprocessor_fn,
                allocation_pool_min_size: expected_allocation_pool_min_size,
                allocation_pool_max_size: expected_allocation_pool_max_size,
                allocation_pool_warning_size: expected_allocation_pool_warning_size,
            } = expected.clone()
            else {
                panic!("Expected {actual} to be NumpySerdeConfig::STATIC {{..}}");
            };
            assert_eq!(actual_shape, expected_shape);
            validate_fn_eq(
                actual_preprocessor_fn.map(|v| v.into_bound(py)).as_ref(),
                expected_preprocessor_fn.map(|v| v.into_bound(py)).as_ref(),
                "preprocessor_fn",
            )?;
            validate_fn_eq(
                actual_postprocessor_fn.map(|v| v.into_bound(py)).as_ref(),
                expected_postprocessor_fn.map(|v| v.into_bound(py)).as_ref(),
                "postprocessor_fn",
            )?;
            assert_eq!(
                actual_allocation_pool_min_size,
                expected_allocation_pool_min_size
            );
            assert_eq!(
                actual_allocation_pool_max_size,
                expected_allocation_pool_max_size
            );
            assert_eq!(
                actual_allocation_pool_warning_size,
                expected_allocation_pool_warning_size
            );
        }
    }
    Ok(())
}

fn tests_submod<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyModule>> {
    let sub = PyModule::new(py, "pickling_numpy_serde_config_tests")?;
    sub.add_function(wrap_pyfunction!(validate_eq, py)?)?;
    Ok(sub)
}

#[test]
fn run_pydantic_tests() -> PyResult<()> {
    Python::initialize();
    Python::attach(|py| {
        run_python_test_file(
            py,
            "python/tests/pickling/numpy_serde_config.py",
            tests_submod(py)?,
        )
    })
}
