use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyDict, PyType},
};
use strum::IntoEnumIterator;

use crate::{
    pyany_serde_impl::{InitStrategy, InitStrategyKind},
    pydantic::common::ValidationContext,
};

pub fn init_strategy_constructor_aux<'py>(
    data: Bound<'py, PyAny>,
    context: &mut ValidationContext,
) -> PyResult<InitStrategy> {
    let type_field = data
        .get_item("type")?
        .extract::<String>()?
        .to_ascii_lowercase();
    match type_field.as_str() {
        "all" => Ok(InitStrategy::ALL {}),
        "some" => {
            let kwargs = data.get_item("kwargs")?.extract::<Vec<String>>()?;
            Ok(InitStrategy::SOME { kwargs })
        }
        "none" => Ok(InitStrategy::NONE {}),
        v => Err(PyValueError::new_err(format!(
            "Unexpected value '{}' for field: {}.type. Allowed values are 'all', 'some', or 'none'.",
            v,
            context.path
        ))),
    }
}

#[pyfunction]
fn init_strategy_constructor_with_info<'py>(
    py: Python<'py>,
    data: Bound<'py, PyAny>,
    info: Bound<'py, PyAny>,
) -> PyResult<Bound<'py, InitStrategy>> {
    let mut context = ValidationContext::from_info(&info)?;
    Bound::new(py, init_strategy_constructor_aux(data, &mut context)?)
}

#[pyfunction]
pub fn init_strategy_serializer<'py>(
    py: Python<'py>,
    init_strategy: &InitStrategy,
) -> PyResult<Bound<'py, PyDict>> {
    let data = PyDict::new(py);
    data.set_item("type", init_strategy.to_string().to_ascii_lowercase())?;
    if let InitStrategy::SOME { kwargs } = init_strategy {
        data.set_item("kwargs", kwargs)?;
    }
    Ok(data)
}

pub fn get_init_strategy_typed_dict_schema<'py>(
    py: Python<'py>,
    kind: Option<&InitStrategyKind>,
    core_schema: &Bound<'py, PyAny>,
) -> PyResult<Bound<'py, PyAny>> {
    if kind.is_none() {
        return core_schema.call_method1(
            "union_schema",
            (InitStrategyKind::iter()
                .map(|k| get_init_strategy_typed_dict_schema(py, Some(&k), core_schema))
                .collect::<PyResult<Vec<_>>>()?,),
        );
    }
    let kind = kind.unwrap();
    let typed_dict_schema = core_schema.getattr("typed_dict_schema")?;
    let typed_dict_field = core_schema.getattr("typed_dict_field")?;
    let str_schema = core_schema.getattr("str_schema")?;
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
    if *kind == InitStrategyKind::SOME {
        typed_dict_fields.set_item(
            "kwargs",
            typed_dict_field
                .call1((core_schema.call_method1("list_schema", (str_schema.call0()?,))?,))?,
        )?;
    }
    typed_dict_schema.call1((typed_dict_fields,))
}

#[pymethods]
impl InitStrategy {
    // pydantic methods
    #[classmethod]
    fn __get_pydantic_core_schema__<'py>(
        cls: &Bound<'py, PyType>,
        _source_type: Bound<'py, PyAny>,
        _handler: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let py = cls.py();
        let core_schema = py.import("pydantic_core")?.getattr("core_schema")?;
        let kind = InitStrategyKind::from_type_object(cls)?;
        let base_schema = get_init_strategy_typed_dict_schema(py, kind.as_ref(), &core_schema)?;
        let is_instance_schema = core_schema.call_method1("is_instance_schema", (cls,))?;
        let json_schema = core_schema.call_method1(
            "chain_schema",
            ([
                base_schema.clone(),
                core_schema.call_method1(
                    "no_info_before_validator_function",
                    (
                        wrap_pyfunction!(init_strategy_constructor_with_info, py)?,
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
                        (wrap_pyfunction!(init_strategy_serializer, py)?,),
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
