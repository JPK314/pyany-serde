use std::{collections::BTreeMap, str::FromStr};

use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyDict, PyFunction, PyType},
    PyTypeInfo,
};
use strum::{IntoEnumIterator, VariantNames};

use crate::{
    common::NumpyDtype,
    pydantic::{
        common::ValidationContext,
        init_strategy::{
            get_init_strategy_typed_dict_schema, init_strategy_constructor_aux,
            init_strategy_serializer,
        },
        numpy_serde_config::{
            get_numpy_serde_config_typed_dict_schema, numpy_serde_config_constructor_aux,
            numpy_serde_config_serializer,
        },
        unpickling::unpickle_field,
    },
    PyAnySerdeType,
};

fn pyany_serde_type_constructor_aux<'py>(
    py: Python<'py>,
    data: &Bound<'py, PyAny>,
    context: &mut ValidationContext,
) -> PyResult<PyAnySerdeType> {
    let cur_path = context.path.clone();
    let pyany_serde_type_field = data
        .get_item("type")?
        .extract::<String>()?
        .to_ascii_lowercase();
    let pyany_serde_type = match pyany_serde_type_field.as_str() {
        "bool" => PyAnySerdeType::BOOL {},
        "bytes" => PyAnySerdeType::BYTES {},
        "complex" => PyAnySerdeType::COMPLEX {},
        "dataclass" => {
            let clazz = unpickle_field(py, data, "dataclass_pkl", context)?.unbind();
            context.path = format!("{cur_path}.init_strategy");
            let init_strategy =
                init_strategy_constructor_aux(data.get_item("init_strategy")?, context)?;
            let mut field_serde_type_dict = BTreeMap::new();
            for (key, serde_type_data) in data
                .get_item("field_serde_type_dict")?
                .cast_into::<PyDict>()?
                .into_iter()
            {
                let key = key.extract::<String>()?;
                context.path = format!("{cur_path}[{key}]");
                let value = pyany_serde_type_constructor_aux(py, &serde_type_data, context)?;
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
            context.path = format!("{cur_path}.keys_serde_type");
            let keys_serde_type =
                pyany_serde_type_constructor_aux(py, &keys_serde_type_data, context)?;
            let values_serde_type_data = data.get_item("values_serde_type")?;
            context.path = format!("{cur_path}.values_serde_type");
            let values_serde_type =
                pyany_serde_type_constructor_aux(py, &values_serde_type_data, context)?;
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
            context.path = format!("{cur_path}.items_serde_type");
            let items_serde_type =
                pyany_serde_type_constructor_aux(py, &items_serde_type_data, context)?;
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
            context.path = format!("{cur_path}.config");
            let numpy_serde_config =
                numpy_serde_config_constructor_aux(py, data.get_item("config")?, context)?;
            PyAnySerdeType::NUMPY {
                dtype,
                config: numpy_serde_config,
            }
        }
        "option" => {
            let value_serde_type_data = data.get_item("value_serde_type")?;
            context.path = format!("{cur_path}.value_serde_type");
            let value_serde_type =
                pyany_serde_type_constructor_aux(py, &value_serde_type_data, context)?;
            PyAnySerdeType::OPTION {
                value_serde_type: Py::new(py, value_serde_type)?,
            }
        }
        "pickle" => PyAnySerdeType::PICKLE {},
        "pythonserde" => PyAnySerdeType::PYTHONSERDE {
            python_serde: unpickle_field(py, data, "python_serde_pkl", context)?.unbind(),
        },
        "set" => {
            let items_serde_type_data = data.get_item("items_serde_type")?;
            context.path = format!("{cur_path}.items_serde_type");
            let items_serde_type =
                pyany_serde_type_constructor_aux(py, &items_serde_type_data, context)?;
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
                .enumerate()
                .map(|(idx, item_serde_type_data)| {
                    context.path = format!("{cur_path}[{idx}]");
                    let item_serde_type =
                        pyany_serde_type_constructor_aux(py, item_serde_type_data, context)?;
                    Ok(item_serde_type)
                })
                .collect::<PyResult<Vec<_>>>()?;
            PyAnySerdeType::TUPLE { item_serde_types }
        }
        "typeddict" => {
            let mut key_serde_type_dict = BTreeMap::new();
            for (key, serde_type_data) in data
                .get_item("key_serde_type_dict")?
                .cast_into::<PyDict>()?
                .into_iter()
            {
                let key = key.extract::<String>()?;
                context.path = format!("{cur_path}[{key}]");
                let value = pyany_serde_type_constructor_aux(py, &serde_type_data, context)?;
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
                .enumerate()
                .map(|(idx, option_serde_type_data)| {
                    context.path = format!("{cur_path}[{idx}]");
                    let option_serde_type =
                        pyany_serde_type_constructor_aux(py, option_serde_type_data, context)?;
                    Ok(option_serde_type)
                })
                .collect::<PyResult<Vec<_>>>()?;
            let option_choice_fn = unpickle_field(py, data, "option_choice_fn_pkl", context)?
                .cast_into::<PyFunction>()?
                .unbind();
            PyAnySerdeType::UNION {
                option_serde_types,
                option_choice_fn,
            }
        }
        v => Err(PyValueError::new_err(format!("Unexpected type: {v}")))?,
    };
    context.path = cur_path;

    Ok(pyany_serde_type)
}

#[pyfunction]
fn pyany_serde_type_constructor_with_info<'py>(
    py: Python<'py>,
    data: Bound<'py, PyAny>,
    info: Bound<'py, PyAny>,
) -> PyResult<Bound<'py, PyAnySerdeType>> {
    let mut context = ValidationContext::from_info(&info)?;
    Bound::new(
        py,
        pyany_serde_type_constructor_aux(py, &data, &mut context)?,
    )
}

#[pyfunction]
pub fn pyany_serde_type_serializer<'py>(
    py: Python<'py>,
    pyany_serde_type: &PyAnySerdeType,
) -> PyResult<Bound<'py, PyDict>> {
    let data = PyDict::new(py);
    data.set_item("type", pyany_serde_type.to_string().to_ascii_lowercase())?;
    if let PyAnySerdeType::DATACLASS {
        clazz,
        init_strategy,
        field_serde_type_dict,
    } = pyany_serde_type
    {
        data.set_item(
            "dataclass_pkl",
            py.import("pickle")?
                .getattr("dumps")?
                .call1((clazz,))?
                .call_method0("hex")?,
        )?;
        data.set_item(
            "init_strategy",
            init_strategy_serializer(py, init_strategy)?,
        )?;
        data.set_item(
            "field_serde_type_dict",
            field_serde_type_dict
                .iter()
                .map(|(key, field_serde_type)| {
                    Ok((key, pyany_serde_type_serializer(py, field_serde_type)?))
                })
                .collect::<PyResult<BTreeMap<_, _>>>()?,
        )?;
    } else if let PyAnySerdeType::DICT {
        keys_serde_type,
        values_serde_type,
    } = pyany_serde_type
    {
        data.set_item(
            "keys_serde_type",
            pyany_serde_type_serializer(py, &keys_serde_type.extract::<PyAnySerdeType>(py)?)?,
        )?;
        data.set_item(
            "values_serde_type",
            pyany_serde_type_serializer(py, &values_serde_type.extract::<PyAnySerdeType>(py)?)?,
        )?;
    } else if let PyAnySerdeType::LIST { items_serde_type } = pyany_serde_type {
        data.set_item(
            "items_serde_type",
            pyany_serde_type_serializer(py, &items_serde_type.extract::<PyAnySerdeType>(py)?)?,
        )?;
    } else if let PyAnySerdeType::NUMPY { dtype, config } = pyany_serde_type {
        data.set_item("dtype", dtype.to_string())?;
        data.set_item("config", numpy_serde_config_serializer(py, config)?)?;
    } else if let PyAnySerdeType::OPTION { value_serde_type } = pyany_serde_type {
        data.set_item(
            "value_serde_type",
            pyany_serde_type_serializer(py, &value_serde_type.extract::<PyAnySerdeType>(py)?)?,
        )?;
    } else if let PyAnySerdeType::PYTHONSERDE { python_serde } = pyany_serde_type {
        data.set_item(
            "python_serde_pkl",
            py.import("pickle")?
                .getattr("dumps")?
                .call1((python_serde,))?
                .call_method0("hex")?,
        )?;
    } else if let PyAnySerdeType::SET { items_serde_type } = pyany_serde_type {
        data.set_item(
            "items_serde_type",
            pyany_serde_type_serializer(py, &items_serde_type.extract::<PyAnySerdeType>(py)?)?,
        )?;
    } else if let PyAnySerdeType::TUPLE { item_serde_types } = pyany_serde_type {
        data.set_item(
            "item_serde_types",
            item_serde_types
                .iter()
                .map(|item_serde_type| pyany_serde_type_serializer(py, item_serde_type))
                .collect::<PyResult<Vec<_>>>()?,
        )?;
    } else if let PyAnySerdeType::TYPEDDICT {
        key_serde_type_dict,
    } = pyany_serde_type
    {
        data.set_item(
            "key_serde_type_dict",
            key_serde_type_dict
                .iter()
                .map(|(key, field_serde_type)| {
                    Ok((key, pyany_serde_type_serializer(py, field_serde_type)?))
                })
                .collect::<PyResult<BTreeMap<_, _>>>()?,
        )?;
    } else if let PyAnySerdeType::UNION {
        option_serde_types,
        option_choice_fn,
    } = pyany_serde_type
    {
        data.set_item(
            "option_serde_types",
            option_serde_types
                .iter()
                .map(|item_serde_type| pyany_serde_type_serializer(py, &item_serde_type))
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
    Ok(data)
}

#[pymethods]
impl PyAnySerdeType {
    // pydantic methods
    #[classmethod]
    fn __get_pydantic_core_schema__<'py>(
        cls: &Bound<'py, PyType>,
        _source_type: Bound<'py, PyAny>,
        _handler: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let py = cls.py();
        let pydantic_core = py.import("pydantic_core")?;
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
                            &[(
                                "pattern",
                                [
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
                            typed_dict_field.call1((get_init_strategy_typed_dict_schema(
                                py,
                                None,
                                &core_schema,
                            )?,))?,
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
                                    &[(
                                        "pattern",
                                        [
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
                            typed_dict_field.call1((get_numpy_serde_config_typed_dict_schema(
                                py,
                                None,
                                &core_schema,
                            )?,))?,
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
        let union_schema = core_schema.call_method(
            "union_schema",
            (union_list,),
            Some(&PyDict::from_sequence(
                &[("ref", "pyany_serde_type_schema")].into_pyobject(py)?,
            )?),
        )?;
        let is_instance_schema =
            core_schema.call_method1("is_instance_schema", (PyAnySerdeType::type_object(py),))?;
        let json_schema = core_schema.call_method1(
            "chain_schema",
            ([
                union_schema.clone(),
                core_schema.call_method1(
                    "with_info_before_validator_function",
                    (
                        wrap_pyfunction!(pyany_serde_type_constructor_with_info, py)?,
                        any_schema.call0()?,
                    ),
                )?,
            ],),
        )?;
        let python_schema =
            core_schema.call_method1("union_schema", ([&is_instance_schema, &json_schema],))?;
        let json_or_python_schema = core_schema.call_method(
            "json_or_python_schema",
            (json_schema, python_schema),
            Some(&PyDict::from_sequence(
                &[(
                    "serialization",
                    core_schema.call_method(
                        "plain_serializer_function_ser_schema",
                        (wrap_pyfunction!(pyany_serde_type_serializer, py)?,),
                        Some(&PyDict::from_sequence(
                            &[("return_schema", union_schema.clone())].into_pyobject(py)?,
                        )?),
                    )?,
                )]
                .into_pyobject(py)?,
            )?),
        )?;
        core_schema.call_method(
            "definitions_schema",
            (&json_or_python_schema,),
            Some(&PyDict::from_sequence(
                &[("definitions", [union_schema])].into_pyobject(py)?,
            )?),
        )
    }
}
