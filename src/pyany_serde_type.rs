use std::collections::BTreeMap;
use std::env;
use std::io;
use std::io::Write;
use std::str::FromStr;

use num_traits::{FromPrimitive, ToPrimitive};
use pyo3::exceptions::asyncio::InvalidStateError;
use pyo3::exceptions::PyValueError;
use pyo3::types::{PyBytes, PyCFunction, PyDict, PyFunction, PyTuple, PyType};
use pyo3::{prelude::*, PyTypeInfo};
use strum::{IntoEnumIterator, VariantNames};
use strum_macros::Display;

use crate::common::NumpyDtype;
use crate::communication::{
    append_bytes_vec, append_string_vec, append_usize_vec, retrieve_bytes, retrieve_string,
    retrieve_usize,
};
use crate::pyany_serde_impl::{
    numpy_check_for_unpickling, InitStrategy, NumpySerdeConfig, PickleableInitStrategy,
    PickleableNumpySerdeConfig,
};

// This enum is used to store information about a type which is sent between processes to dynamically recover a Box<dyn PyAnySerde>
#[pyclass]
#[derive(Clone)]
pub struct PickleablePyAnySerdeType(pub Option<Option<PyAnySerdeType>>);

#[pymethods]
impl PickleablePyAnySerdeType {
    // We need a zero-args constructor for compatibility with unpickling
    #[new]
    #[pyo3(signature = (*args))]
    fn new<'py>(args: Bound<'py, PyTuple>) -> PyResult<Self> {
        let vec_args = args.iter().collect::<Vec<_>>();
        if vec_args.len() > 1 {
            return Err(PyValueError::new_err(format!(
                "PickleablePyAnySerde constructor takes 0 or 1 parameters, received {}",
                args.as_any().repr()?.to_str()?
            )));
        }
        if vec_args.len() == 1 {
            Ok(PickleablePyAnySerdeType(Some(
                vec_args[0].extract::<Option<PyAnySerdeType>>()?,
            )))
        } else {
            Ok(PickleablePyAnySerdeType(None))
        }
    }

    // pickle methods
    pub fn __getstate__(&self) -> PyResult<Vec<u8>> {
        let pyany_serde_type_option = self.0.as_ref().unwrap();
        Ok(match pyany_serde_type_option {
            Some(pyany_serde_type) => {
                let mut option_bytes = vec![1];
                let mut pyany_serde_type_bytes = match pyany_serde_type {
                    PyAnySerdeType::BOOL {} => vec![0],
                    PyAnySerdeType::BYTES {} => vec![1],
                    PyAnySerdeType::COMPLEX {} => vec![2],
                    PyAnySerdeType::DATACLASS {
                        clazz,
                        init_strategy,
                        field_serde_type_dict,
                    } => {
                        let mut bytes = vec![3];
                        append_bytes_vec(
                            &mut bytes,
                            &PickleableInitStrategy(Some(init_strategy.clone())).__getstate__()[..],
                        );
                        append_usize_vec(&mut bytes, field_serde_type_dict.len());
                        for (field, serde_type) in field_serde_type_dict.iter() {
                            append_string_vec(&mut bytes, field);
                            append_bytes_vec(
                                &mut bytes,
                                &PickleablePyAnySerdeType(Some(Some(serde_type.clone())))
                                    .__getstate__()?[..],
                            );
                        }
                        Python::with_gil::<_, PyResult<_>>(|py| {
                            let clazz_py_bytes = py
                                .import("pickle")?
                                .getattr("dumps")?
                                .call1((clazz,))?
                                .downcast_into::<PyBytes>()?;
                            append_bytes_vec(&mut bytes, clazz_py_bytes.as_bytes());
                            Ok(bytes)
                        })?
                    }
                    PyAnySerdeType::DICT {
                        keys_serde_type,
                        values_serde_type,
                    } => {
                        let mut bytes = vec![4];
                        Python::with_gil::<_, PyResult<_>>(|py| {
                            for py_serde_type in
                                vec![keys_serde_type, values_serde_type].into_iter()
                            {
                                let serde_type = py_serde_type.extract::<PyAnySerdeType>(py)?;
                                append_bytes_vec(
                                    &mut bytes,
                                    &PickleablePyAnySerdeType(Some(Some(serde_type.clone())))
                                        .__getstate__()?[..],
                                );
                            }
                            Ok(bytes)
                        })?
                    }
                    PyAnySerdeType::DYNAMIC {} => vec![5],
                    PyAnySerdeType::FLOAT {} => vec![6],
                    PyAnySerdeType::INT {} => vec![7],
                    PyAnySerdeType::LIST { items_serde_type } => {
                        let mut bytes = vec![8];
                        Python::with_gil::<_, PyResult<_>>(|py| {
                            let serde_type = items_serde_type.extract::<PyAnySerdeType>(py)?;
                            append_bytes_vec(
                                &mut bytes,
                                &PickleablePyAnySerdeType(Some(Some(serde_type))).__getstate__()?[..],
                            );
                            Ok(bytes)
                        })?
                    }
                    PyAnySerdeType::NUMPY { dtype, config } => {
                        let mut bytes = vec![9, dtype.to_u8().unwrap()];
                        append_bytes_vec(
                            &mut bytes,
                            &PickleableNumpySerdeConfig(Some(config.clone())).__getstate__()?[..],
                        );
                        bytes
                    }
                    PyAnySerdeType::OPTION { value_serde_type } => {
                        let mut bytes = vec![10];
                        Python::with_gil::<_, PyResult<_>>(|py| {
                            let serde_type = value_serde_type.extract::<PyAnySerdeType>(py)?;
                            append_bytes_vec(
                                &mut bytes,
                                &PickleablePyAnySerdeType(Some(Some(serde_type.clone())))
                                    .__getstate__()?[..],
                            );
                            Ok(bytes)
                        })?
                    }
                    PyAnySerdeType::PICKLE {} => vec![11],
                    PyAnySerdeType::PYTHONSERDE { python_serde } => {
                        let mut bytes = vec![12];
                        Python::with_gil::<_, PyResult<_>>(|py| {
                            let python_serde_py_bytes = py
                                .import("pickle")?
                                .getattr("dumps")?
                                .call1((python_serde,))?
                                .downcast_into::<PyBytes>()?;
                            append_bytes_vec(&mut bytes, python_serde_py_bytes.as_bytes());
                            Ok(bytes)
                        })?
                    }
                    PyAnySerdeType::SET { items_serde_type } => {
                        let mut bytes = vec![13];
                        Python::with_gil::<_, PyResult<_>>(|py| {
                            let serde_type = items_serde_type.extract::<PyAnySerdeType>(py)?;
                            append_bytes_vec(
                                &mut bytes,
                                &PickleablePyAnySerdeType(Some(Some(serde_type.clone())))
                                    .__getstate__()?[..],
                            );
                            Ok(bytes)
                        })?
                    }
                    PyAnySerdeType::STRING {} => vec![14],
                    PyAnySerdeType::TUPLE { item_serde_types } => {
                        let mut bytes = vec![15];
                        bytes.extend_from_slice(&item_serde_types.len().to_ne_bytes());
                        for serde_type in item_serde_types.iter() {
                            append_bytes_vec(
                                &mut bytes,
                                &PickleablePyAnySerdeType(Some(Some(serde_type.clone())))
                                    .__getstate__()?[..],
                            );
                        }
                        bytes
                    }
                    PyAnySerdeType::TYPEDDICT {
                        key_serde_type_dict,
                    } => {
                        let mut bytes = vec![16];
                        bytes.extend_from_slice(&key_serde_type_dict.len().to_ne_bytes());
                        for (key, serde_type) in key_serde_type_dict.iter() {
                            append_string_vec(&mut bytes, key);
                            append_bytes_vec(
                                &mut bytes,
                                &PickleablePyAnySerdeType(Some(Some(serde_type.clone())))
                                    .__getstate__()?[..],
                            );
                        }
                        bytes
                    }
                    PyAnySerdeType::UNION {
                        option_serde_types,
                        option_choice_fn,
                    } => {
                        let mut bytes = vec![17];
                        bytes.extend_from_slice(&option_serde_types.len().to_ne_bytes());
                        for serde_type in option_serde_types.iter() {
                            append_bytes_vec(
                                &mut bytes,
                                &PickleablePyAnySerdeType(Some(Some(serde_type.clone())))
                                    .__getstate__()?[..],
                            );
                        }
                        Python::with_gil::<_, PyResult<_>>(|py| {
                            let option_choice_fn_py_bytes = py
                                .import("pickle")?
                                .getattr("dumps")?
                                .call1((option_choice_fn,))?
                                .downcast_into::<PyBytes>()?;
                            append_bytes_vec(&mut bytes, option_choice_fn_py_bytes.as_bytes());
                            Ok(bytes)
                        })?
                    }
                };
                option_bytes.append(&mut pyany_serde_type_bytes);
                option_bytes
            }
            None => vec![0],
        })
    }

    pub fn __setstate__(&mut self, state: Vec<u8>) -> PyResult<()> {
        let buf = &state[..];
        let option_byte = state[0];
        self.0 = Some(match option_byte {
            0 => None,
            1 => {
                let type_byte = state[1];
                let mut offset = 2;
                Some(match type_byte {
                    0 => PyAnySerdeType::BOOL {},
                    1 => PyAnySerdeType::BYTES {},
                    2 => PyAnySerdeType::COMPLEX {},
                    3 => {
                        let init_strategy_bytes;
                        (init_strategy_bytes, offset) = retrieve_bytes(buf, offset)?;
                        let mut pickleable_init_strategy = PickleableInitStrategy(None);
                        pickleable_init_strategy.__setstate__(init_strategy_bytes.to_vec())?;
                        let n_fields;
                        (n_fields, offset) = retrieve_usize(buf, offset)?;
                        let mut field_serde_type_dict = BTreeMap::new();
                        for _ in 0..n_fields {
                            let field;
                            (field, offset) = retrieve_string(buf, offset)?;
                            let serde_type_bytes;
                            (serde_type_bytes, offset) = retrieve_bytes(buf, offset)?;
                            let mut pickleable_serde_type = PickleablePyAnySerdeType(None);
                            pickleable_serde_type.__setstate__(serde_type_bytes.to_vec())?;
                            field_serde_type_dict
                                .insert(field, pickleable_serde_type.0.unwrap().unwrap());
                        }
                        Python::with_gil::<_, PyResult<_>>(|py| {
                            let clazz_bytes;
                            (clazz_bytes, offset) = retrieve_bytes(buf, offset)?;
                            let clazz = py
                                .import("pickle")?
                                .getattr("loads")?
                                .call1((PyBytes::new(py, clazz_bytes).into_pyobject(py)?,))?
                                .unbind();
                            Ok(PyAnySerdeType::DATACLASS {
                                clazz,
                                init_strategy: pickleable_init_strategy.0.unwrap(),
                                field_serde_type_dict,
                            })
                        })?
                    }
                    4 => Python::with_gil::<_, PyResult<_>>(|py| {
                        let keys_serde_type_bytes;
                        (keys_serde_type_bytes, offset) = retrieve_bytes(buf, offset)?;
                        let mut pickleable_keys_serde_type = PickleablePyAnySerdeType(None);
                        pickleable_keys_serde_type.__setstate__(keys_serde_type_bytes.to_vec())?;
                        let values_serde_type_bytes;
                        (values_serde_type_bytes, offset) = retrieve_bytes(buf, offset)?;
                        let mut pickleable_values_serde_type = PickleablePyAnySerdeType(None);
                        pickleable_values_serde_type
                            .__setstate__(values_serde_type_bytes.to_vec())?;
                        Ok(PyAnySerdeType::DICT {
                            keys_serde_type: Py::new(
                                py,
                                pickleable_keys_serde_type.0.unwrap().unwrap(),
                            )?,
                            values_serde_type: Py::new(
                                py,
                                pickleable_values_serde_type.0.unwrap().unwrap(),
                            )?,
                        })
                    })?,
                    5 => PyAnySerdeType::DYNAMIC {},
                    6 => PyAnySerdeType::FLOAT {},
                    7 => PyAnySerdeType::INT {},
                    8 => Python::with_gil::<_, PyResult<_>>(|py| {
                        let serde_type_bytes;
                        (serde_type_bytes, offset) = retrieve_bytes(buf, offset)?;
                        let mut pickleable_serde_type = PickleablePyAnySerdeType(None);
                        pickleable_serde_type.__setstate__(serde_type_bytes.to_vec())?;
                        Ok(PyAnySerdeType::LIST {
                            items_serde_type: Py::new(
                                py,
                                pickleable_serde_type.0.unwrap().unwrap(),
                            )?,
                        })
                    })?,
                    9 => {
                        let dtype = NumpyDtype::from_u8(buf[offset]).unwrap();
                        offset += 1;
                        let numpy_serde_config_bytes;
                        (numpy_serde_config_bytes, _) = retrieve_bytes(buf, offset)?;
                        let mut pickleable_numpy_serde_config = PickleableNumpySerdeConfig(None);
                        pickleable_numpy_serde_config
                            .__setstate__(numpy_serde_config_bytes.to_vec())?;
                        PyAnySerdeType::NUMPY {
                            dtype,
                            config: pickleable_numpy_serde_config.0.unwrap(),
                        }
                    }
                    10 => Python::with_gil::<_, PyResult<_>>(|py| {
                        let serde_type_bytes;
                        (serde_type_bytes, offset) = retrieve_bytes(buf, offset)?;
                        let mut pickleable_serde_type = PickleablePyAnySerdeType(None);
                        pickleable_serde_type.__setstate__(serde_type_bytes.to_vec())?;
                        Ok(PyAnySerdeType::OPTION {
                            value_serde_type: Py::new(
                                py,
                                pickleable_serde_type.0.unwrap().unwrap(),
                            )?,
                        })
                    })?,
                    11 => PyAnySerdeType::PICKLE {},
                    12 => Python::with_gil::<_, PyResult<_>>(|py| {
                        let python_serde_bytes;
                        (python_serde_bytes, offset) = retrieve_bytes(buf, offset)?;
                        let python_serde = py
                            .import("pickle")?
                            .getattr("loads")?
                            .call1((PyBytes::new(py, python_serde_bytes).into_pyobject(py)?,))?
                            .unbind();
                        Ok(PyAnySerdeType::PYTHONSERDE { python_serde })
                    })?,
                    13 => Python::with_gil::<_, PyResult<_>>(|py| {
                        let serde_type_bytes;
                        (serde_type_bytes, offset) = retrieve_bytes(buf, offset)?;
                        let mut pickleable_serde_type = PickleablePyAnySerdeType(None);
                        pickleable_serde_type.__setstate__(serde_type_bytes.to_vec())?;
                        Ok(PyAnySerdeType::SET {
                            items_serde_type: Py::new(
                                py,
                                pickleable_serde_type.0.unwrap().unwrap(),
                            )?,
                        })
                    })?,
                    14 => PyAnySerdeType::STRING {},
                    15 => {
                        let n_items;
                        (n_items, offset) = retrieve_usize(buf, offset)?;
                        let mut item_serde_types = Vec::with_capacity(n_items);
                        for _ in 0..n_items {
                            let serde_type_bytes;
                            (serde_type_bytes, offset) = retrieve_bytes(buf, offset)?;
                            let mut pickleable_serde_type = PickleablePyAnySerdeType(None);
                            pickleable_serde_type.__setstate__(serde_type_bytes.to_vec())?;
                            item_serde_types.push(pickleable_serde_type.0.unwrap().unwrap())
                        }
                        PyAnySerdeType::TUPLE { item_serde_types }
                    }
                    16 => {
                        let n_keys;
                        (n_keys, offset) = retrieve_usize(buf, offset)?;
                        let mut key_serde_type_dict = BTreeMap::new();
                        for _ in 0..n_keys {
                            let key;
                            (key, offset) = retrieve_string(buf, offset)?;
                            let serde_type_bytes;
                            (serde_type_bytes, offset) = retrieve_bytes(buf, offset)?;
                            let mut pickleable_serde_type = PickleablePyAnySerdeType(None);
                            pickleable_serde_type.__setstate__(serde_type_bytes.to_vec())?;
                            key_serde_type_dict
                                .insert(key, pickleable_serde_type.0.unwrap().unwrap());
                        }
                        PyAnySerdeType::TYPEDDICT {
                            key_serde_type_dict,
                        }
                    }
                    17 => {
                        let n_options;
                        (n_options, offset) = retrieve_usize(buf, offset)?;
                        let mut option_serde_types = Vec::with_capacity(n_options);
                        for _ in 0..n_options {
                            let serde_type_bytes;
                            (serde_type_bytes, offset) = retrieve_bytes(buf, offset)?;
                            let mut pickleable_serde_type = PickleablePyAnySerdeType(None);
                            pickleable_serde_type.__setstate__(serde_type_bytes.to_vec())?;
                            option_serde_types.push(pickleable_serde_type.0.unwrap().unwrap())
                        }
                        Python::with_gil::<_, PyResult<_>>(|py| {
                            let option_choice_fn_bytes;
                            (option_choice_fn_bytes, offset) = retrieve_bytes(buf, offset)?;
                            let option_choice_fn = py.import("pickle")?.getattr("loads")?.call1(
                                (PyBytes::new(py, option_choice_fn_bytes).into_pyobject(py)?,),
                            )?;
                            Ok(PyAnySerdeType::UNION {
                                option_serde_types,
                                option_choice_fn: option_choice_fn
                                    .downcast_into::<PyFunction>()?
                                    .unbind(),
                            })
                        })?
                    }
                    v => Err(InvalidStateError::new_err(format!(
                        "Got invalid type byte for PyAnySerde: {v}"
                    )))?,
                })
            }
            v => Err(InvalidStateError::new_err(format!(
                "Got invalid option byte for PyAnySerdeType: {v}"
            )))?,
        });

        Ok(())
    }
}

#[pyclass]
#[derive(Debug, Clone, Display, strum_macros::VariantNames)]
pub enum PyAnySerdeType {
    BOOL {},
    BYTES {},
    COMPLEX {},
    DATACLASS {
        clazz: PyObject,
        init_strategy: InitStrategy,
        field_serde_type_dict: BTreeMap<String, PyAnySerdeType>,
    },
    DICT {
        keys_serde_type: Py<PyAnySerdeType>,
        values_serde_type: Py<PyAnySerdeType>,
    },
    DYNAMIC {},
    FLOAT {},
    INT {},
    LIST {
        items_serde_type: Py<PyAnySerdeType>,
    },
    #[pyo3(constructor = (dtype, config = NumpySerdeConfig::DYNAMIC { preprocessor_fn: None, postprocessor_fn: None }))]
    NUMPY {
        dtype: NumpyDtype,
        config: NumpySerdeConfig,
    },
    OPTION {
        value_serde_type: Py<PyAnySerdeType>,
    },
    PICKLE {},
    PYTHONSERDE {
        python_serde: PyObject,
    },
    SET {
        items_serde_type: Py<PyAnySerdeType>,
    },
    STRING {},
    TUPLE {
        item_serde_types: Vec<PyAnySerdeType>,
    },
    TYPEDDICT {
        key_serde_type_dict: BTreeMap<String, PyAnySerdeType>,
    },
    UNION {
        option_serde_types: Vec<PyAnySerdeType>,
        option_choice_fn: Py<PyFunction>,
    },
}

fn check_for_unpickling_aux<'py>(data: &Bound<'py, PyAny>) -> PyResult<bool> {
    let pyany_serde_type_field = data
        .get_item("type")?
        .extract::<String>()?
        .to_ascii_lowercase();
    Ok(match pyany_serde_type_field.as_str() {
        "dataclass" => true,
        "dict" => {
            check_for_unpickling_aux(&data.get_item("keys_serde_type")?)?
                || check_for_unpickling_aux(&data.get_item("values_serde_type")?)?
        }
        "list" => check_for_unpickling_aux(&data.get_item("items_serde_type")?)?,
        "numpy" => numpy_check_for_unpickling(&data.get_item("config")?)?,
        "option" => check_for_unpickling_aux(&data.get_item("value_serde_type")?)?,
        "pythonserde" => true,
        "set" => check_for_unpickling_aux(&data.get_item("items_serde_type")?)?,
        "tuple" => {
            let mut has_unpickling = false;
            for item_serde_type_data in data
                .get_item("item_serde_types")?
                .extract::<Vec<Bound<'_, PyAny>>>()?
                .iter()
            {
                has_unpickling |= check_for_unpickling_aux(&item_serde_type_data)?;
            }
            has_unpickling
        }
        "typeddict" => {
            let mut has_unpickling = false;
            for (_, serde_type_data) in data
                .get_item("key_serde_type_dict")?
                .downcast_into::<PyDict>()?
                .iter()
            {
                has_unpickling |= check_for_unpickling_aux(&serde_type_data)?;
            }
            has_unpickling
        }
        "union" => true,
        _ => false,
    })
}

#[pyfunction]
fn check_for_unpickling<'py, 'a>(data: &'a Bound<'py, PyAny>) -> PyResult<&'a Bound<'py, PyAny>> {
    let silent_mode = env::var("PYANY_SERDE_UNPICKLE_WITHOUT_PROMPT")
        .map(|v| v.eq("1"))
        .unwrap_or(false);
    if !silent_mode && check_for_unpickling_aux(&data)? {
        println!("WARNING: About to call unpickle on the hexadecimal-encoded binary contents of some config fields. If you do not trust the origins of this json, or you cannot otherwise verify the safety of this field's contents, you should not proceed.");
        print!("Proceed? (y/N)\t");
        io::stdout().flush()?;
        let mut response = String::new();
        io::stdin().read_line(&mut response).unwrap();
        if !response.trim().eq_ignore_ascii_case("y") {
            Err(PyValueError::new_err("Operation cancelled by user due to unpickling required to build config model from json"))?
        } else {
            println!("Continuing with execution. If you would like to ignore this warning in the future, set the environment variable PYANY_SERDE_UNPICKLE_WITHOUT_PROMPT to \"1\".")
        }
    }
    Ok(data)
}

fn get_before_validator_fn<'py>(
    _handler: &Bound<'py, PyAny>,
    _schema_validator: &Bound<'py, PyAny>,
) -> PyResult<Bound<'py, PyCFunction>> {
    let _py = _handler.py();
    let py_handler = _handler.clone().unbind();
    let py_schema_validator = _schema_validator.clone().unbind();
    let func = move |args: &Bound<'_, PyTuple>,
                     _kwargs: Option<&Bound<'_, PyDict>>|
          -> PyResult<PyObject> {
        // initial setup
        let py = args.py();
        let data = args.get_item(0)?;
        let handler = py_handler.bind(py);
        let schema_validator = py_schema_validator.bind(py);

        // processing of data
        let pyany_serde_type_field = data
            .get_item("type")?
            .extract::<String>()?
            .to_ascii_lowercase();
        let pyany_serde_type = match pyany_serde_type_field.as_str() {
            "bool" => PyAnySerdeType::BOOL {},
            "bytes" => PyAnySerdeType::BYTES {},
            "complex" => PyAnySerdeType::COMPLEX {},
            "dataclass" => {
                let clazz_bytes_hex = data.get_item("dataclass_pkl")?.extract::<String>()?;
                let clazz = py
                    .import("pickle")?
                    .getattr("loads")?
                    .call1((PyBytes::new(
                        py,
                        &hex::decode(clazz_bytes_hex.as_str()).map_err(|err| {
                            PyValueError::new_err(format!(
                                "dataclass_pkl could not be decoded from hex into bytes: {}",
                                err.to_string()
                            ))
                        })?,
                    ),))?
                    .unbind();
                let init_strategy = schema_validator
                    .call1((handler
                        .call_method1("generate_schema", (InitStrategy::type_object(py),))?,))?
                    .call_method1("validate_python", (data.get_item("init_strategy")?,))?
                    .extract::<InitStrategy>()?;
                let mut field_serde_type_dict = BTreeMap::new();
                for (key, serde_type_data) in data
                    .get_item("field_serde_type_dict")?
                    .downcast_into::<PyDict>()?
                    .into_iter()
                {
                    let key = key.extract::<String>()?;
                    let value = get_before_validator_fn(handler, schema_validator)?
                        .call1((serde_type_data,))?
                        .extract::<PyAnySerdeType>()?;
                    field_serde_type_dict.insert(key, value);
                }
                PyAnySerdeType::DATACLASS {
                    clazz,
                    init_strategy,
                    field_serde_type_dict,
                }
            }
            "dict" => {
                let keys_serde_type_data = data.get_item("keys_serde_type")?;
                let keys_serde_type = get_before_validator_fn(handler, schema_validator)?
                    .call1((keys_serde_type_data,))?
                    .extract::<PyAnySerdeType>()?;
                let values_serde_type_data = data.get_item("values_serde_type")?;
                let values_serde_type = get_before_validator_fn(handler, schema_validator)?
                    .call1((values_serde_type_data,))?
                    .extract::<PyAnySerdeType>()?;
                PyAnySerdeType::DICT {
                    keys_serde_type: Py::new(py, keys_serde_type)?,
                    values_serde_type: Py::new(py, values_serde_type)?,
                }
            }
            "dynamic" => PyAnySerdeType::DYNAMIC {},
            "float" => PyAnySerdeType::FLOAT {},
            "int" => PyAnySerdeType::INT {},
            "list" => {
                let items_serde_type_data = data.get_item("items_serde_type")?;
                let items_serde_type = get_before_validator_fn(handler, schema_validator)?
                    .call1((items_serde_type_data,))?
                    .extract::<PyAnySerdeType>()?;
                PyAnySerdeType::LIST {
                    items_serde_type: Py::new(py, items_serde_type)?,
                }
            }
            "numpy" => {
                let dtype_string = data.get_item("dtype")?.extract::<String>()?;
                let dtype = NumpyDtype::from_str(dtype_string.as_str()).map_err(|_| {
                    PyValueError::new_err(format!(
                        "dtype was provided as {dtype_string} which is not a valid dtype"
                    ))
                })?;
                let numpy_serde_config = schema_validator
                    .call1((handler
                        .call_method1("generate_schema", (NumpySerdeConfig::type_object(py),))?,))?
                    .call_method1("validate_python", (data.get_item("config")?,))?
                    .extract::<NumpySerdeConfig>()?;
                PyAnySerdeType::NUMPY {
                    dtype,
                    config: numpy_serde_config,
                }
            }
            "option" => {
                let value_serde_type_data = data.get_item("value_serde_type")?;
                let value_serde_type = get_before_validator_fn(handler, schema_validator)?
                    .call1((value_serde_type_data,))?
                    .extract::<PyAnySerdeType>()?;
                PyAnySerdeType::OPTION {
                    value_serde_type: Py::new(py, value_serde_type)?,
                }
            }
            "pickle" => PyAnySerdeType::PICKLE {},
            "pythonserde" => {
                let python_serde_bytes_hex =
                    data.get_item("python_serde_pkl")?.extract::<String>()?;
                let python_serde = py
                    .import("pickle")?
                    .getattr("loads")?
                    .call1((PyBytes::new(
                        py,
                        &hex::decode(python_serde_bytes_hex.as_str()).map_err(|err| {
                            PyValueError::new_err(format!(
                                "python_serde_pkl could not be decoded from hex into bytes: {}",
                                err.to_string()
                            ))
                        })?,
                    ),))?
                    .unbind();
                PyAnySerdeType::PYTHONSERDE { python_serde }
            }
            "set" => {
                let items_serde_type_data = data.get_item("items_serde_type")?;
                let items_serde_type = get_before_validator_fn(handler, schema_validator)?
                    .call1((items_serde_type_data,))?
                    .extract::<PyAnySerdeType>()?;
                PyAnySerdeType::SET {
                    items_serde_type: Py::new(py, items_serde_type)?,
                }
            }
            "string" => PyAnySerdeType::STRING {},
            "tuple" => {
                let item_serde_types_data = data
                    .get_item("item_serde_types")?
                    .extract::<Vec<Bound<'_, PyAny>>>()?;
                let item_serde_types = item_serde_types_data
                    .iter()
                    .map(|item_serde_type_data| {
                        Ok(get_before_validator_fn(handler, schema_validator)?
                            .call1((item_serde_type_data,))?
                            .extract::<PyAnySerdeType>()?)
                    })
                    .collect::<PyResult<Vec<_>>>()?;
                PyAnySerdeType::TUPLE { item_serde_types }
            }
            "typeddict" => {
                let mut key_serde_type_dict = BTreeMap::new();
                for (key, serde_type_data) in data
                    .get_item("key_serde_type_dict")?
                    .downcast_into::<PyDict>()?
                    .into_iter()
                {
                    let key = key.extract::<String>()?;
                    let value = get_before_validator_fn(handler, schema_validator)?
                        .call1((serde_type_data,))?
                        .extract::<PyAnySerdeType>()?;
                    key_serde_type_dict.insert(key, value);
                }
                PyAnySerdeType::TYPEDDICT {
                    key_serde_type_dict,
                }
            }
            "union" => {
                let option_serde_types_data = data
                    .get_item("option_serde_types")?
                    .extract::<Vec<Bound<'_, PyAny>>>()?;
                let option_serde_types = option_serde_types_data
                    .iter()
                    .map(|option_serde_type_data| {
                        Ok(get_before_validator_fn(handler, schema_validator)?
                            .call1((option_serde_type_data,))?
                            .extract::<PyAnySerdeType>()?)
                    })
                    .collect::<PyResult<Vec<_>>>()?;
                let option_choice_fn_bytes_hex =
                    data.get_item("option_choice_fn_pkl")?.extract::<String>()?;
                let option_choice_fn = py
                    .import("pickle")?
                    .getattr("loads")?
                    .call1((PyBytes::new(
                        py,
                        &hex::decode(option_choice_fn_bytes_hex.as_str()).map_err(|err| {
                            PyValueError::new_err(format!(
                                "option_choice_fn_pkl could not be decoded from hex into bytes: {}",
                                err.to_string()
                            ))
                        })?,
                    ),))?
                    .downcast_into::<PyFunction>()?
                    .unbind();
                PyAnySerdeType::UNION {
                    option_serde_types,
                    option_choice_fn,
                }
            }
            v => Err(PyValueError::new_err(format!("Unexpected type: {v}")))?,
        };

        Ok(pyany_serde_type.into_pyobject(py)?.into_any().unbind())
    };
    PyCFunction::new_closure(_py, None, None, func)
}

#[pymethods]
impl PyAnySerdeType {
    fn as_pickleable<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        Ok(PickleablePyAnySerdeType(Some(Some(self.clone())))
            .into_pyobject(py)?
            .into_any())
    }

    // pydantic methods
    #[classmethod]
    fn __get_pydantic_core_schema__<'py>(
        cls: &Bound<'py, PyType>,
        _source_type: Bound<'py, PyAny>,
        handler: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let py = cls.py();
        let generate_schema = handler.getattr("generate_schema")?;
        let pydantic_core = py.import("pydantic_core")?;
        let schema_validator = pydantic_core.getattr("SchemaValidator")?;
        let core_schema = pydantic_core.getattr("core_schema")?;

        let str_schema = core_schema.getattr("str_schema")?;
        let typed_dict_schema = core_schema.getattr("typed_dict_schema")?;
        let list_schema = core_schema.getattr("list_schema")?;
        let dict_schema = core_schema.getattr("dict_schema")?;
        let any_schema = core_schema.getattr("any_schema")?;
        let typed_dict_field = core_schema.getattr("typed_dict_field")?;

        let pyany_serde_type_reference_schema = core_schema
            .call_method1("definition_reference_schema", ("pyany_serde_type_schema",))?;
        let pyany_serde_type_reference_schema_field =
            typed_dict_field.call1((&pyany_serde_type_reference_schema,))?;

        let union_list = PyAnySerdeType::VARIANTS
            .iter()
            .map(|pyany_serde_type_variant| {
                let pyany_serde_type_field = pyany_serde_type_variant.to_ascii_lowercase();
                let typed_dict_fields = PyDict::new(py);
                typed_dict_fields.set_item(
                    "type",
                    typed_dict_field.call1((str_schema.call(
                        (),
                        Some(&PyDict::from_sequence(
                            &vec![(
                                "pattern",
                                vec![
                                    "^".to_owned(),
                                    pyany_serde_type_field.clone(),
                                    "$".to_owned(),
                                ]
                                .join("")
                                .into_pyobject(py)?
                                .into_any(),
                            )]
                            .into_pyobject(py)?,
                        )?),
                    )?,))?,
                )?;
                match pyany_serde_type_field.as_str() {
                    "dataclass" => {
                        typed_dict_fields.set_item(
                            "dataclass_pkl",
                            typed_dict_field.call1((str_schema.call0()?,))?,
                        )?;
                        typed_dict_fields.set_item(
                            "init_strategy",
                            typed_dict_field.call1((
                                generate_schema.call1((InitStrategy::type_object(py),))?,
                            ))?,
                        )?;
                        typed_dict_fields.set_item(
                            "field_serde_type_dict",
                            typed_dict_field.call1((dict_schema.call1((
                                str_schema.call0()?,
                                &pyany_serde_type_reference_schema,
                            ))?,))?,
                        )?;
                    }
                    "dict" => {
                        typed_dict_fields.set_item(
                            "keys_serde_type",
                            &pyany_serde_type_reference_schema_field,
                        )?;
                        typed_dict_fields.set_item(
                            "values_serde_type",
                            &pyany_serde_type_reference_schema_field,
                        )?;
                    }
                    "list" => {
                        typed_dict_fields.set_item(
                            "items_serde_type",
                            &pyany_serde_type_reference_schema_field,
                        )?;
                    }
                    "numpy" => {
                        typed_dict_fields.set_item(
                            "dtype",
                            typed_dict_field.call1((str_schema.call(
                                (),
                                Some(&PyDict::from_sequence(
                                    &vec![(
                                        "pattern",
                                        vec![
                                            "^(".to_owned(),
                                            NumpyDtype::iter()
                                                .map(|dtype_str| dtype_str.to_string())
                                                .collect::<Vec<_>>()
                                                .join("|"),
                                            ")$".to_owned(),
                                        ]
                                        .join(""),
                                    )]
                                    .into_pyobject(py)?,
                                )?),
                            )?,))?,
                        )?;
                        typed_dict_fields.set_item(
                            "config",
                            typed_dict_field.call1((
                                generate_schema.call1((NumpySerdeConfig::type_object(py),))?,
                            ))?,
                        )?;
                    }
                    "option" => {
                        typed_dict_fields.set_item(
                            "value_serde_type",
                            &pyany_serde_type_reference_schema_field,
                        )?;
                    }
                    "pythonserde" => {
                        typed_dict_fields.set_item(
                            "python_serde_pkl",
                            typed_dict_field.call1((str_schema.call0()?,))?,
                        )?;
                    }
                    "set" => {
                        typed_dict_fields.set_item(
                            "items_serde_type",
                            &pyany_serde_type_reference_schema_field,
                        )?;
                    }
                    "tuple" => {
                        typed_dict_fields.set_item(
                            "item_serde_types",
                            typed_dict_field.call1((
                                list_schema.call1((&pyany_serde_type_reference_schema,))?,
                            ))?,
                        )?;
                    }
                    "typeddict" => {
                        typed_dict_fields.set_item(
                            "key_serde_type_dict",
                            typed_dict_field.call1((dict_schema.call1((
                                str_schema.call0()?,
                                &pyany_serde_type_reference_schema,
                            ))?,))?,
                        )?;
                    }
                    "union" => {
                        typed_dict_fields.set_item(
                            "option_serde_types",
                            typed_dict_field.call1((
                                list_schema.call1((&pyany_serde_type_reference_schema,))?,
                            ))?,
                        )?;
                        typed_dict_fields.set_item(
                            "option_choice_fn_pkl",
                            typed_dict_field.call1((str_schema.call0()?,))?,
                        )?;
                    }
                    _ => (),
                };
                Ok(typed_dict_schema.call1((typed_dict_fields,))?)
            })
            .collect::<PyResult<Vec<_>>>()?;
        let pyany_serde_type_union_schema = core_schema.call_method(
            "union_schema",
            (union_list,),
            Some(&PyDict::from_sequence(
                &vec![("ref", "pyany_serde_type_schema")].into_pyobject(py)?,
            )?),
        )?;

        let pyany_serde_type_python_schema =
            core_schema.call_method1("is_instance_schema", (PyAnySerdeType::type_object(py),))?;
        let pyany_serde_type_json_or_python_schema = core_schema.call_method1(
            "json_or_python_schema",
            (
                core_schema.call_method1(
                    "chain_schema",
                    (vec![
                        core_schema.call_method1(
                            "no_info_before_validator_function",
                            (
                                wrap_pyfunction!(check_for_unpickling, py)?,
                                any_schema.call0()?,
                            ),
                        )?,
                        pyany_serde_type_union_schema.clone(),
                        core_schema.call_method1(
                            "no_info_before_validator_function",
                            (
                                get_before_validator_fn(&handler, &schema_validator)?,
                                &pyany_serde_type_python_schema,
                            ),
                        )?,
                    ],),
                )?,
                pyany_serde_type_python_schema,
            ),
        )?;
        core_schema.call_method(
            "definitions_schema",
            (&pyany_serde_type_json_or_python_schema,),
            Some(&PyDict::from_sequence(
                &vec![("definitions", vec![&pyany_serde_type_union_schema])].into_pyobject(py)?,
            )?),
        )
    }

    fn to_json(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let data = PyDict::new(py);
            data.set_item("type", self.to_string().to_ascii_lowercase())?;
            if let PyAnySerdeType::DATACLASS {
                clazz,
                init_strategy,
                field_serde_type_dict,
            } = self
            {
                data.set_item(
                    "dataclass_pkl",
                    py.import("pickle")?
                        .getattr("dumps")?
                        .call1((clazz,))?
                        .call_method0("hex")?,
                )?;
                data.set_item("init_strategy", init_strategy.to_json()?)?;
                data.set_item(
                    "field_serde_type_dict",
                    field_serde_type_dict
                        .iter()
                        .map(|(key, field_serde_type)| Ok((key, field_serde_type.to_json()?)))
                        .collect::<PyResult<BTreeMap<_, _>>>()?,
                )?;
            } else if let PyAnySerdeType::DICT {
                keys_serde_type,
                values_serde_type,
            } = self
            {
                data.set_item(
                    "keys_serde_type",
                    keys_serde_type.extract::<PyAnySerdeType>(py)?.to_json()?,
                )?;
                data.set_item(
                    "values_serde_type",
                    values_serde_type.extract::<PyAnySerdeType>(py)?.to_json()?,
                )?;
            } else if let PyAnySerdeType::LIST { items_serde_type } = self {
                data.set_item(
                    "items_serde_type",
                    items_serde_type.extract::<PyAnySerdeType>(py)?.to_json()?,
                )?;
            } else if let PyAnySerdeType::NUMPY { dtype, config } = self {
                data.set_item("dtype", dtype.to_string())?;
                data.set_item("config", config.to_json()?)?;
            } else if let PyAnySerdeType::OPTION { value_serde_type } = self {
                data.set_item(
                    "value_serde_type",
                    value_serde_type.extract::<PyAnySerdeType>(py)?.to_json()?,
                )?;
            } else if let PyAnySerdeType::PYTHONSERDE { python_serde } = self {
                data.set_item(
                    "python_serde_pkl",
                    py.import("pickle")?
                        .getattr("dumps")?
                        .call1((python_serde,))?
                        .call_method0("hex")?,
                )?;
            } else if let PyAnySerdeType::SET { items_serde_type } = self {
                data.set_item(
                    "items_serde_type",
                    items_serde_type.extract::<PyAnySerdeType>(py)?.to_json()?,
                )?;
            } else if let PyAnySerdeType::TUPLE { item_serde_types } = self {
                data.set_item(
                    "item_serde_types",
                    item_serde_types
                        .iter()
                        .map(|item_serde_type| item_serde_type.to_json())
                        .collect::<PyResult<Vec<_>>>()?,
                )?;
            } else if let PyAnySerdeType::TYPEDDICT {
                key_serde_type_dict,
            } = self
            {
                data.set_item(
                    "key_serde_type_dict",
                    key_serde_type_dict
                        .iter()
                        .map(|(key, field_serde_type)| Ok((key, field_serde_type.to_json()?)))
                        .collect::<PyResult<BTreeMap<_, _>>>()?,
                )?;
            } else if let PyAnySerdeType::UNION {
                option_serde_types,
                option_choice_fn,
            } = self
            {
                data.set_item(
                    "option_serde_types",
                    option_serde_types
                        .iter()
                        .map(|item_serde_type| item_serde_type.to_json())
                        .collect::<PyResult<Vec<_>>>()?,
                )?;
                data.set_item(
                    "option_choice_fn_pkl",
                    py.import("pickle")?
                        .getattr("dumps")?
                        .call1((option_choice_fn,))?
                        .call_method0("hex")?,
                )?;
            }
            Ok(data.into_any().unbind())
        })
    }
}
