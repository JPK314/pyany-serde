use pyo3::{prelude::*, types::PyDict};
use std::{
    collections::{BTreeMap, HashSet},
    fs,
};
use strum::IntoEnumIterator;

use crate::{
    pyany_serde_impl::{InitStrategy, InitStrategyKind, NumpySerdeConfig, NumpySerdeConfigKind},
    pyany_serde_type::PyAnySerdeTypeKind,
    PyAnySerdeType,
};

fn validate_fn_eq<'py>(
    actual_fn_option: &Option<Bound<'py, PyAny>>,
    expected_fn_option: &Option<Bound<'py, PyAny>>,
    field: String,
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

#[pyfunction(name = "validate_eq")]
pub fn validate_init_strategy_eq<'py>(
    expected: &InitStrategy,
    actual: &InitStrategy,
    field: String,
) -> PyResult<()> {
    match actual.clone() {
        InitStrategy::ALL {} => {
            let InitStrategy::ALL {} = expected.clone() else {
                panic!("Expected field {field} to be InitStrategy::ALL {{}} but was {actual}");
            };
        }
        InitStrategy::SOME {
            kwargs: actual_kwargs,
        } => {
            let InitStrategy::SOME {
                kwargs: expected_kwargs,
            } = expected.clone()
            else {
                panic!("Expected field {field} to be InitStrategy::SOME {{..}} but was {actual}");
            };
            assert_eq!(actual_kwargs, expected_kwargs);
        }
        InitStrategy::NONE {} => {
            let InitStrategy::NONE {} = expected.clone() else {
                panic!("Expected field {field} to be InitStrategy::NONE {{}} but was {actual}");
            };
        }
    };
    Ok(())
}

#[pyfunction(name = "validate_eq")]
pub fn validate_numpy_serde_config_eq<'py>(
    py: Python<'py>,
    expected: &NumpySerdeConfig,
    actual: &NumpySerdeConfig,
    field: String,
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
                panic!("Expected field {field} to be NumpySerdeConfig::DYNAMIC {{..}} but was {actual}");
            };
            validate_fn_eq(
                &actual_preprocessor_fn.map(|v| v.into_bound(py)),
                &expected_preprocessor_fn.map(|v| v.into_bound(py)),
                format!("{field}.preprocessor_fn"),
            )?;
            validate_fn_eq(
                &actual_postprocessor_fn.map(|v| v.into_bound(py)),
                &expected_postprocessor_fn.map(|v| v.into_bound(py)),
                format!("{field}.postprocessor_fn"),
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
                panic!(
                    "Expected field {field} to be NumpySerdeConfig::STATIC {{..}} but was {actual}"
                );
            };
            assert_eq!(
                expected_shape, actual_shape,
                "Expected field {field}.shape to be {:?} but was {:?}",
                expected_shape, actual_shape
            );
            validate_fn_eq(
                &expected_preprocessor_fn.map(|v| v.into_bound(py)),
                &actual_preprocessor_fn.map(|v| v.into_bound(py)),
                format!("{field}.preprocessor_fn"),
            )?;
            validate_fn_eq(
                &expected_postprocessor_fn.map(|v| v.into_bound(py)),
                &actual_postprocessor_fn.map(|v| v.into_bound(py)),
                format!("{field}.postprocessor_fn"),
            )?;
            assert_eq!(
                expected_allocation_pool_min_size,
                actual_allocation_pool_min_size,
                "Expected field {field}.allocation_pool_min_size to be {expected_allocation_pool_min_size} but was {actual_allocation_pool_min_size}",
            );
            assert_eq!(
                expected_allocation_pool_max_size, actual_allocation_pool_max_size,
                "Expected field {field}.allocation_pool_max_size to be {:?} but was {:?}",
                expected_allocation_pool_max_size, actual_allocation_pool_max_size
            );
            assert_eq!(
                expected_allocation_pool_warning_size, actual_allocation_pool_warning_size,
                "Expected field {field}.allocation_pool_warning_size to be {:?} but was {:?}",
                expected_allocation_pool_warning_size, actual_allocation_pool_warning_size
            );
        }
    }
    Ok(())
}

fn validate_key_serde_type_dict_eq<'py>(
    py: Python<'py>,
    expected: &BTreeMap<String, PyAnySerdeType>,
    actual: &BTreeMap<String, PyAnySerdeType>,
    field: String,
) -> PyResult<()> {
    let expected_keys = expected.keys().collect::<HashSet<_>>();
    let actual_keys = actual.keys().collect::<HashSet<_>>();
    assert_eq!(
        expected_keys, actual_keys,
        "Expected {field} to have keys {:?} but had keys {:?} instead",
        expected_keys, actual_keys
    );
    for (key, expected_serde_type) in expected.iter() {
        validate_pyany_serde_type_eq(
            py,
            expected_serde_type,
            actual.get(key).unwrap(),
            format!("{field}[{key}]"),
        )?;
    }
    Ok(())
}

#[pyfunction(name = "validate_eq")]
pub fn validate_pyany_serde_type_eq<'py>(
    py: Python<'py>,
    expected: &PyAnySerdeType,
    actual: &PyAnySerdeType,
    field: String,
) -> PyResult<()> {
    match actual.clone() {
        PyAnySerdeType::BOOL {} => {
            let PyAnySerdeType::BOOL {} = expected.clone() else {
                panic!("Expected field {field} to be PyAnySerdeType::BOOL {{}} but was {actual}");
            };
        }
        PyAnySerdeType::BYTES {} => {
            let PyAnySerdeType::BYTES {} = expected.clone() else {
                panic!("Expected field {field} to be PyAnySerdeType::BYTES {{}} but was {actual}");
            };
        }
        PyAnySerdeType::COMPLEX {} => {
            let PyAnySerdeType::COMPLEX {} = expected.clone() else {
                panic!(
                    "Expected field {field} to be PyAnySerdeType::COMPLEX {{}} but was {actual}"
                );
            };
        }
        PyAnySerdeType::DATACLASS {
            clazz: actual_clazz,
            init_strategy: actual_init_strategy,
            field_serde_type_dict: actual_field_serde_type_dict,
        } => {
            let PyAnySerdeType::DATACLASS {
                clazz: expected_clazz,
                init_strategy: expected_init_strategy,
                field_serde_type_dict: expected_field_serde_type_dict,
            } = expected.clone()
            else {
                panic!("Expected field {field} to be PyAnySerdeType::DATACLASS {{..}} but was {actual}");
            };
            let actual_clazz = actual_clazz.bind(py);
            let expected_clazz = expected_clazz.bind(py);
            assert!(
                actual_clazz.eq(expected_clazz)?,
                "Expected field {field}.clazz to be {} but was {}",
                expected_clazz.repr()?,
                actual_clazz.repr()?,
            );
            validate_init_strategy_eq(
                &expected_init_strategy,
                &actual_init_strategy,
                format!("{field}.init_strategy"),
            )?;
            validate_key_serde_type_dict_eq(
                py,
                &expected_field_serde_type_dict,
                &actual_field_serde_type_dict,
                format!("{field}.field_serde_type_dict"),
            )?;
        }
        PyAnySerdeType::DICT {
            keys_serde_type: actual_keys_serde_type,
            values_serde_type: actual_values_serde_type,
        } => {
            let PyAnySerdeType::DICT {
                keys_serde_type: expected_keys_serde_type,
                values_serde_type: expected_values_serde_type,
            } = expected.clone()
            else {
                panic!("Expected field {field} to be PyAnySerdeType::DICT {{..}} but was {actual}");
            };
            validate_pyany_serde_type_eq(
                py,
                &expected_keys_serde_type.extract::<PyAnySerdeType>(py)?,
                &actual_keys_serde_type.extract::<PyAnySerdeType>(py)?,
                format!("{field}.keys_serde_type"),
            )?;
            validate_pyany_serde_type_eq(
                py,
                &expected_values_serde_type.extract::<PyAnySerdeType>(py)?,
                &actual_values_serde_type.extract::<PyAnySerdeType>(py)?,
                format!("{field}.values_serde_type"),
            )?;
        }
        PyAnySerdeType::DYNAMIC {} => {
            let PyAnySerdeType::DYNAMIC {} = expected.clone() else {
                panic!(
                    "Expected field {field} to be PyAnySerdeType::DYNAMIC {{}} but was {actual}"
                );
            };
        }
        PyAnySerdeType::FLOAT {} => {
            let PyAnySerdeType::FLOAT {} = expected.clone() else {
                panic!("Expected field {field} to be PyAnySerdeType::FLOAT {{}} but was {actual}");
            };
        }
        PyAnySerdeType::INT {} => {
            let PyAnySerdeType::INT {} = expected.clone() else {
                panic!("Expected field {field} to be PyAnySerdeType::INT {{}} but was {actual}");
            };
        }
        PyAnySerdeType::LIST {
            items_serde_type: actual_items_serde_type,
        } => {
            let PyAnySerdeType::LIST {
                items_serde_type: expected_items_serde_type,
            } = expected.clone()
            else {
                panic!("Expected field {field} to be PyAnySerdeType::LIST {{..}} but was {actual}");
            };
            validate_pyany_serde_type_eq(
                py,
                &expected_items_serde_type.extract::<PyAnySerdeType>(py)?,
                &actual_items_serde_type.extract::<PyAnySerdeType>(py)?,
                format!("{field}.items_serde_type"),
            )?;
        }
        PyAnySerdeType::NUMPY {
            dtype: actual_dtype,
            config: actual_config,
        } => {
            let PyAnySerdeType::NUMPY {
                dtype: expected_dtype,
                config: expected_config,
            } = expected.clone()
            else {
                panic!(
                    "Expected field {field} to be PyAnySerdeType::NUMPY {{..}} but was {actual}"
                );
            };
            assert_eq!(
                expected_dtype, actual_dtype,
                "Expected {field}.dtype to be {:?} but was {:?}",
                expected_dtype, actual_dtype
            );
            validate_numpy_serde_config_eq(
                py,
                &expected_config,
                &actual_config,
                format!("{field}.config"),
            )?;
        }
        PyAnySerdeType::OPTION {
            value_serde_type: actual_value_serde_type,
        } => {
            let PyAnySerdeType::OPTION {
                value_serde_type: expected_value_serde_type,
            } = expected.clone()
            else {
                panic!(
                    "Expected field {field} to be PyAnySerdeType::OPTION {{..}} but was {actual}"
                );
            };
            validate_pyany_serde_type_eq(
                py,
                &expected_value_serde_type.extract::<PyAnySerdeType>(py)?,
                &actual_value_serde_type.extract::<PyAnySerdeType>(py)?,
                format!("{field}.value_serde_type"),
            )?;
        }
        PyAnySerdeType::PICKLE {} => {
            let PyAnySerdeType::PICKLE {} = expected.clone() else {
                panic!("Expected field {field} to be PyAnySerdeType::PICKLE {{}} but was {actual}");
            };
        }
        PyAnySerdeType::PYTHONSERDE {
            python_serde: actual_python_serde,
        } => {
            let PyAnySerdeType::PYTHONSERDE {
                python_serde: expected_python_serde,
            } = expected.clone()
            else {
                panic!(
                    "Expected field {field} to be PyAnySerdeType::PYTHONSERDE {{..}} but was {actual}"
                );
            };
            let actual_python_serde = actual_python_serde.bind(py);
            let expected_python_serde = expected_python_serde.bind(py);
            assert!(
                actual_python_serde.eq(expected_python_serde)?,
                "Expected clazz {} to equal {}",
                actual_python_serde.repr()?,
                expected_python_serde.repr()?
            );
        }
        PyAnySerdeType::SET {
            items_serde_type: actual_items_serde_type,
        } => {
            let PyAnySerdeType::SET {
                items_serde_type: expected_items_serde_type,
            } = expected.clone()
            else {
                panic!("Expected field {field} to be PyAnySerdeType::SET {{..}} but was {actual}");
            };
            validate_pyany_serde_type_eq(
                py,
                &expected_items_serde_type.extract::<PyAnySerdeType>(py)?,
                &actual_items_serde_type.extract::<PyAnySerdeType>(py)?,
                format!("{field}.items_serde_type"),
            )?;
        }
        PyAnySerdeType::STRING {} => {
            let PyAnySerdeType::STRING {} = expected.clone() else {
                panic!("Expected field {field} to be PyAnySerdeType::STRING {{}} but was {actual}");
            };
        }
        PyAnySerdeType::TUPLE {
            item_serde_types: actual_item_serde_types,
        } => {
            let PyAnySerdeType::TUPLE {
                item_serde_types: expected_item_serde_types,
            } = expected.clone()
            else {
                panic!(
                    "Expected field {field} to be PyAnySerdeType::TUPLE {{..}} but was {actual}"
                );
            };
            assert_eq!(
                expected_item_serde_types.len(),
                actual_item_serde_types.len(),
                "Expected field {field} to have length {} but was {}",
                expected_item_serde_types.len(),
                actual_item_serde_types.len()
            );
            for (idx, actual_serde_type) in actual_item_serde_types.iter().enumerate() {
                validate_pyany_serde_type_eq(
                    py,
                    expected_item_serde_types.get(idx).unwrap(),
                    actual_serde_type,
                    format!("{field}.item_serde_types[{idx}]"),
                )?;
            }
        }
        PyAnySerdeType::TYPEDDICT {
            key_serde_type_dict: actual_key_serde_type_dict,
        } => {
            let PyAnySerdeType::TYPEDDICT {
                key_serde_type_dict: expected_key_serde_type_dict,
            } = expected.clone()
            else {
                panic!(
                    "Expected field {field} to be PyAnySerdeType::TYPEDDICT {{..}} but was {actual}"
                );
            };
            validate_key_serde_type_dict_eq(
                py,
                &expected_key_serde_type_dict,
                &actual_key_serde_type_dict,
                format!("{field}.key_serde_type_dict"),
            )?;
        }
        PyAnySerdeType::UNION {
            option_serde_types: actual_option_serde_types,
            option_choice_fn: actual_option_choice_fn,
        } => {
            let PyAnySerdeType::UNION {
                option_serde_types: expected_option_serde_types,
                option_choice_fn: expected_option_choice_fn,
            } = expected.clone()
            else {
                panic!(
                    "Expected field {field} to be PyAnySerdeType::UNION {{..}} but was {actual}"
                );
            };
            assert_eq!(
                expected_option_serde_types.len(),
                actual_option_serde_types.len(),
                "Expected field {field} to have length {} but was {}",
                expected_option_serde_types.len(),
                actual_option_serde_types.len()
            );
            for (idx, actual_serde_type) in actual_option_serde_types.iter().enumerate() {
                validate_pyany_serde_type_eq(
                    py,
                    expected_option_serde_types.get(idx).unwrap(),
                    actual_serde_type,
                    format!("{field}.option_serde_types[{idx}]"),
                )?;
            }
            validate_fn_eq(
                &Some(expected_option_choice_fn.into_bound(py).into_any()),
                &Some(actual_option_choice_fn.into_bound(py).into_any()),
                format!("{field}.option_choice_fn"),
            )?;
        }
    };
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

    let test_module = PyModule::new(py, "python_test")?;
    let globals = test_module.dict();
    py.run(
        std::ffi::CString::new(fs::read_to_string("python/pyany_serde/python_serde.py").unwrap())?
            .as_c_str(),
        Some(&globals),
        Some(&globals),
    )?;
    let python_serde_submod = PyModule::new(py, "python_serde")?;
    python_serde_submod.setattr("PythonSerde", globals.get_item("PythonSerde").unwrap())?;
    module.add_submodule(&python_serde_submod)?;
    modules.set_item("pyany_serde.python_serde", &python_serde_submod)?;

    py.run(
        std::ffi::CString::new(fs::read_to_string(path).unwrap())?.as_c_str(),
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
