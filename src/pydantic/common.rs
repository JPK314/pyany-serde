use pyo3::prelude::*;

#[pyclass(skip_from_py_object)]
#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub prompt_for_unpickle: bool,
    pub model_field: Option<String>,
    pub path: String,
}

impl ValidationContext {
    pub fn from_info<'py>(info: &Bound<'py, PyAny>) -> PyResult<ValidationContext> {
        let validation_context_option = info
            .getattr("context")?
            .extract::<Option<Bound<'_, ValidationContext>>>()?;
        let (prompt_for_unpickle, model_field, path) =
            if let Some(validation_context) = validation_context_option {
                (
                    validation_context.borrow().prompt_for_unpickle,
                    validation_context.borrow().model_field.clone(),
                    validation_context.borrow().path.clone(),
                )
            } else {
                (
                    true,
                    info.getattr("field_name")?.extract::<Option<String>>()?,
                    "$".to_string(),
                )
            };
        Ok(ValidationContext {
            prompt_for_unpickle,
            model_field,
            path,
        })
    }
}
