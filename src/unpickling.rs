use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::env;
use std::io;
use std::io::Write;

#[pyclass(skip_from_py_object)]
#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub prompt_for_unpickle: bool,
    pub model_field: Option<String>,
    pub path: String,
}

pub fn prompt_for_unpickling(
    context: &mut PyRefMut<ValidationContext>,
    final_key: String,
) -> PyResult<()> {
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
