use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::sync::PyOnceLock;
use pyo3::types::PyBytes;
use std::env;
use std::io;
use std::io::Write;

use crate::pydantic::common::ValidationContext;

static INTERNED_PICKLE_LOADS: PyOnceLock<Py<PyAny>> = PyOnceLock::new();

pub fn unpickle_field_option<'py>(
    py: Python<'py>,
    data: &Bound<'py, PyAny>,
    field: &str,
    context: &mut ValidationContext,
) -> PyResult<Option<Bound<'py, PyAny>>> {
    let field_hex_option = data.get_item(field)?.extract::<Option<String>>()?;
    field_hex_option
        .map(|field_hex| unpickle_field_hex(py, field, field_hex, context))
        .transpose()
}

pub fn unpickle_field<'py>(
    py: Python<'py>,
    data: &Bound<'py, PyAny>,
    field: &str,
    context: &mut ValidationContext,
) -> PyResult<Bound<'py, PyAny>> {
    let field_hex = data.get_item(field)?.extract::<String>()?;
    unpickle_field_hex(py, field, field_hex, context)
}

fn unpickle_field_hex<'py>(
    py: Python<'py>,
    field: &str,
    field_hex: String,
    context: &mut ValidationContext,
) -> PyResult<Bound<'py, PyAny>> {
    prompt_for_unpickling(context, field)?;
    let pickle_loads = INTERNED_PICKLE_LOADS
        .get_or_try_init::<_, PyErr>(py, || Ok(py.import("pickle")?.getattr("loads")?.unbind()))?
        .bind(py);
    Ok::<_, PyErr>(pickle_loads.call1((PyBytes::new(
        py,
        &hex::decode(field_hex.as_str()).map_err(|err| {
            PyValueError::new_err(format!(
                "{}.{} could not be decoded from hex into bytes: {}",
                context.path,
                field,
                err.to_string()
            ))
        })?,
    ),))?)
}

fn prompt_for_unpickling(context: &mut ValidationContext, final_key: &str) -> PyResult<()> {
    let silent_mode = env::var("PYANY_SERDE_UNPICKLE_WITHOUT_PROMPT")
        .map(|v| v.eq("1"))
        .unwrap_or(!context.prompt_for_unpickle);
    if !silent_mode {
        let fieldpath = if let Some(field_name) = context.model_field.clone() {
            format!("{}: {}.{}", field_name, context.path, final_key)
        } else {
            format!("{}.{}", context.path, final_key)
        };
        println!("WARNING: About to call unpickle on the hexadecimal-encoded binary contents of the model field {fieldpath}. If you do not trust the origins of this json, or you cannot otherwise verify the safety of this field's contents, you should not proceed.");
        print!("Proceed? 'y' for yes, 'a' for yes to all pickled strings for this model field, 'n' for no. (Default 'n'):\t");
        io::stdout().flush()?;
        let mut response = String::new();
        io::stdin().read_line(&mut response).unwrap();
        if response.trim().eq_ignore_ascii_case("y") {
            println!("Continuing with execution. If you would like to ignore this warning in the future, set the environment variable PYANY_SERDE_UNPICKLE_WITHOUT_PROMPT to \"1\".\n");
        } else if response.trim().eq_ignore_ascii_case("a") {
            println!("Continuing with execution. If you would like to ignore this warning in the future, set the environment variable PYANY_SERDE_UNPICKLE_WITHOUT_PROMPT to \"1\".\n");
            context.prompt_for_unpickle = false;
        } else {
            Err(PyValueError::new_err("Operation cancelled by user due to unpickling required to build config model from json"))?
        }
    }
    Ok(())
}
