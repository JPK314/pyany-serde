use pyo3::{prelude::*, types::PyDict};
use std::fs;
use strum::IntoEnumIterator;

use crate::{
    pyany_serde_impl::{InitStrategy, InitStrategyKind, NumpySerdeConfig, NumpySerdeConfigKind},
    pyany_serde_type::PyAnySerdeTypeKind,
    PyAnySerdeType,
};

pub fn validate_fn_eq<'py>(
    actual_fn_option: Option<&Bound<'py, PyAny>>,
    expected_fn_option: Option<&Bound<'py, PyAny>>,
    field: &str,
) -> PyResult<()> {
    match expected_fn_option {
        Some(expected_fn) => {
            if let Some(input_fn) = actual_fn_option {
                assert_eq!(
                    expected_fn
                        .getattr("__code__")?
                        .getattr("co_code")?
                        .extract::<&[u8]>()?,
                    input_fn
                        .getattr("__code__")?
                        .getattr("co_code")?
                        .extract::<&[u8]>()?
                )
            } else {
                panic!("Expected {field} to be non-None")
            }
        }
        None => {
            assert!(actual_fn_option.is_none(), "Expected {field} to be None");
        }
    }
    Ok(())
}

pub fn run_python_test_file<'py>(
    py: Python<'py>,
    path: &str,
    tests_submod: Bound<'py, PyModule>,
) -> PyResult<()> {
    let module = PyModule::new(py, "pyany_serde")?;
    module.add_class::<InitStrategy>()?;
    module.add_class::<NumpySerdeConfig>()?;
    module.add_class::<PyAnySerdeType>()?;
    module
        .getattr("InitStrategy")?
        .setattr("__module__", module.name()?)?;
    for kind in InitStrategyKind::iter() {
        kind.type_object(py).setattr("__module__", module.name()?)?;
    }
    module
        .getattr("NumpySerdeConfig")?
        .setattr("__module__", module.name()?)?;
    for kind in NumpySerdeConfigKind::iter() {
        kind.type_object(py).setattr("__module__", module.name()?)?;
    }
    module
        .getattr("PyAnySerdeType")?
        .setattr("__module__", module.name()?)?;
    for kind in PyAnySerdeTypeKind::iter() {
        kind.type_object(py).setattr("__module__", module.name()?)?;
    }
    module.add_submodule(&tests_submod)?;
    let modules = py
        .import("sys")?
        .getattr("modules")?
        .cast_into::<PyDict>()?;
    modules.set_item("pyany_serde", &module)?;
    modules.set_item(
        format!("pyany_serde.{}", tests_submod.name()?.extract::<String>()?),
        &tests_submod,
    )?;

    let source = fs::read_to_string(path).unwrap();

    let test_module = PyModule::new(py, "python_test")?;
    let globals = test_module.dict();

    py.run(
        std::ffi::CString::new(source)?.as_c_str(),
        Some(&globals),
        Some(&globals),
    )?;
    modules.set_item("python_test", &test_module)?;

    for (name, value) in globals.iter() {
        let Some(name) = name.extract::<&str>().ok() else {
            continue;
        };

        if !name.starts_with("test_") {
            continue;
        }

        println!("Running {}", name);

        value.call0()?;
    }

    Ok(())
}
