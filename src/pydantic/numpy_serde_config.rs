use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyDict, PyType},
};
use strum::IntoEnumIterator;

use crate::{
    pyany_serde_impl::{NumpySerdeConfig, NumpySerdeConfigKind},
    pydantic::{common::ValidationContext, unpickling::unpickle_field_option},
};

pub fn numpy_serde_config_constructor_aux<'py>(
    py: Python<'py>,
    data: Bound<'py, PyAny>,
    context: &mut ValidationContext,
) -> PyResult<NumpySerdeConfig> {
    // get fields that exist on all variants first
    let preprocessor_fn_option =
        unpickle_field_option(py, &data, "preprocessor_fn_pkl", context)?.map(|v| v.unbind());
    let postprocessor_fn_option =
        unpickle_field_option(py, &data, "postprocessor_fn_pkl", context)?.map(|v| v.unbind());

    let type_field = data
        .get_item("type")?
        .extract::<String>()?
        .to_ascii_lowercase();
    match type_field.as_str() {
        "dynamic" => Ok(NumpySerdeConfig::DYNAMIC {
            preprocessor_fn: preprocessor_fn_option,
            postprocessor_fn: postprocessor_fn_option,
        }),
        "static" => {
            let shape = data.get_item("shape")?.extract::<Vec<usize>>()?;
            let allocation_pool_min_size = data
                .get_item("allocation_pool_min_size")?
                .extract::<usize>()?;
            let allocation_pool_max_size = data
                .get_item("allocation_pool_max_size")?
                .extract::<Option<usize>>()?;
            let allocation_pool_warning_size = data
                .get_item("allocation_pool_warning_size")?
                .extract::<Option<usize>>()?;
            if allocation_pool_max_size.is_some()
                && allocation_pool_min_size > allocation_pool_max_size.unwrap()
            {
                Err(PyValueError::new_err(format!(
                    "Validation error: allocation_pool_min_size ({}) cannot be greater than allocation_pool_max_size ({})", allocation_pool_min_size, allocation_pool_max_size.unwrap()
                )))?
            }
            Ok(NumpySerdeConfig::STATIC {
                preprocessor_fn: preprocessor_fn_option,
                postprocessor_fn: postprocessor_fn_option,
                shape,
                allocation_pool_min_size,
                allocation_pool_max_size,
                allocation_pool_warning_size,
            })
        },
        v => Err(PyValueError::new_err(format!(
            "Unexpected value '{}' for field: {}.type. Allowed values are 'all', 'some', or 'none'.",
            v,
            context.path
        ))),
    }
}

#[pyfunction]
fn numpy_serde_config_constructor_with_info<'py>(
    py: Python<'py>,
    data: Bound<'py, PyAny>,
    info: Bound<'py, PyAny>,
) -> PyResult<Bound<'py, NumpySerdeConfig>> {
    let mut context = ValidationContext::from_info(&info)?;
    Bound::new(
        py,
        numpy_serde_config_constructor_aux(py, data, &mut context)?,
    )
}

#[pyfunction]
pub fn numpy_serde_config_serializer<'py>(
    py: Python<'py>,
    init_strategy: &NumpySerdeConfig,
) -> PyResult<Bound<'py, PyDict>> {
    let data = PyDict::new(py);
    data.set_item("type", init_strategy.to_string().to_ascii_lowercase())?;
    match init_strategy {
        NumpySerdeConfig::DYNAMIC {
            preprocessor_fn,
            postprocessor_fn,
        } => {
            let preprocessor_fn_pkl = preprocessor_fn
                .as_ref()
                .map(|preprocessor_fn| {
                    Ok::<_, PyErr>(
                        py.import("pickle")?
                            .getattr("dumps")?
                            .call1((preprocessor_fn,))?
                            .call_method0("hex")?,
                    )
                })
                .transpose()?;
            data.set_item("preprocessor_fn_pkl", preprocessor_fn_pkl)?;
            let postprocessor_fn_pkl = postprocessor_fn
                .as_ref()
                .map(|postprocessor_fn| {
                    Ok::<_, PyErr>(
                        py.import("pickle")?
                            .getattr("dumps")?
                            .call1((postprocessor_fn,))?
                            .call_method0("hex")?,
                    )
                })
                .transpose()?;
            data.set_item("postprocessor_fn_pkl", postprocessor_fn_pkl)?;
        }
        NumpySerdeConfig::STATIC {
            preprocessor_fn,
            postprocessor_fn,
            shape,
            allocation_pool_min_size,
            allocation_pool_max_size,
            allocation_pool_warning_size,
        } => {
            let preprocessor_fn_pkl = preprocessor_fn
                .as_ref()
                .map(|preprocessor_fn| {
                    Ok::<_, PyErr>(
                        py.import("pickle")?
                            .getattr("dumps")?
                            .call1((preprocessor_fn,))?
                            .call_method0("hex")?,
                    )
                })
                .transpose()?;
            data.set_item("preprocessor_fn_pkl", preprocessor_fn_pkl)?;
            let postprocessor_fn_pkl = postprocessor_fn
                .as_ref()
                .map(|postprocessor_fn| {
                    Ok::<_, PyErr>(
                        py.import("pickle")?
                            .getattr("dumps")?
                            .call1((postprocessor_fn,))?
                            .call_method0("hex")?,
                    )
                })
                .transpose()?;
            data.set_item("postprocessor_fn_pkl", postprocessor_fn_pkl)?;
            data.set_item("shape", shape)?;
            data.set_item("allocation_pool_min_size", allocation_pool_min_size)?;
            data.set_item("allocation_pool_max_size", allocation_pool_max_size)?;
            data.set_item("allocation_pool_warning_size", allocation_pool_warning_size)?;
        }
    }
    Ok(data)
}

pub fn get_numpy_serde_config_typed_dict_schema<'py>(
    py: Python<'py>,
    kind: Option<&NumpySerdeConfigKind>,
    core_schema: &Bound<'py, PyAny>,
) -> PyResult<Bound<'py, PyAny>> {
    if kind.is_none() {
        return core_schema.call_method1(
            "union_schema",
            (NumpySerdeConfigKind::iter()
                .map(|k| get_numpy_serde_config_typed_dict_schema(py, Some(&k), core_schema))
                .collect::<PyResult<Vec<_>>>()?,),
        );
    }
    let kind = kind.unwrap();
    let typed_dict_schema = core_schema.getattr("typed_dict_schema")?;
    let typed_dict_field = core_schema.getattr("typed_dict_field")?;
    let int_schema = core_schema.getattr("int_schema")?;
    let str_schema = core_schema.getattr("str_schema")?;
    let list_schema = core_schema.getattr("list_schema")?;
    let nullable_schema = core_schema.getattr("nullable_schema")?;
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
                        kind.to_string().to_ascii_lowercase(),
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
    typed_dict_fields.set_item(
        "preprocessor_fn_pkl",
        typed_dict_field.call1((nullable_schema.call1((str_schema.call0()?,))?,))?,
    )?;
    typed_dict_fields.set_item(
        "postprocessor_fn_pkl",
        typed_dict_field.call1((nullable_schema.call1((str_schema.call0()?,))?,))?,
    )?;

    if *kind == NumpySerdeConfigKind::STATIC {
        typed_dict_fields.set_item(
            "shape",
            typed_dict_field.call1((list_schema.call1((int_schema.call(
                (),
                Some(&PyDict::from_sequence(&[("ge", 0)].into_pyobject(py)?)?),
            )?,))?,))?,
        )?;
        typed_dict_fields.set_item(
            "allocation_pool_min_size",
            typed_dict_field.call1((int_schema.call(
                (),
                Some(&PyDict::from_sequence(&[("ge", 0)].into_pyobject(py)?)?),
            )?,))?,
        )?;
        typed_dict_fields.set_item(
            "allocation_pool_max_size",
            typed_dict_field.call1((nullable_schema.call1((int_schema.call(
                (),
                Some(&PyDict::from_sequence(&[("ge", 0)].into_pyobject(py)?)?),
            )?,))?,))?,
        )?;
        typed_dict_fields.set_item(
            "allocation_pool_warning_size",
            typed_dict_field.call1((nullable_schema.call1((int_schema.call(
                (),
                Some(&PyDict::from_sequence(&[("ge", 0)].into_pyobject(py)?)?),
            )?,))?,))?,
        )?;
    }
    typed_dict_schema.call1((typed_dict_fields,))
}

#[pymethods]
impl NumpySerdeConfig {
    // pydantic methods
    #[classmethod]
    fn __get_pydantic_core_schema__<'py>(
        cls: &Bound<'py, PyType>,
        _source_type: Bound<'py, PyAny>,
        _handler: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let py = cls.py();
        let core_schema = py.import("pydantic_core")?.getattr("core_schema")?;
        let kind = NumpySerdeConfigKind::from_type_object(cls)?;
        let base_schema =
            get_numpy_serde_config_typed_dict_schema(py, kind.as_ref(), &core_schema)?;
        let is_instance_schema = core_schema.getattr("is_instance_schema")?.call1((cls,))?;
        let json_schema = core_schema.call_method1(
            "chain_schema",
            ([
                base_schema.clone(),
                core_schema.call_method1(
                    "with_info_before_validator_function",
                    (
                        wrap_pyfunction!(numpy_serde_config_constructor_with_info, py)?,
                        core_schema.call_method0("any_schema")?,
                    ),
                )?,
            ],),
        )?;
        let python_schema =
            core_schema.call_method1("union_schema", ([&is_instance_schema, &json_schema],))?;
        core_schema.call_method(
            "json_or_python_schema",
            (json_schema, python_schema),
            Some(&PyDict::from_sequence(
                &[(
                    "serialization",
                    core_schema.call_method(
                        "plain_serializer_function_ser_schema",
                        (wrap_pyfunction!(numpy_serde_config_serializer, py)?,),
                        Some(&PyDict::from_sequence(
                            &[("return_schema", base_schema)].into_pyobject(py)?,
                        )?),
                    )?,
                )]
                .into_pyobject(py)?,
            )?),
        )
    }
}
