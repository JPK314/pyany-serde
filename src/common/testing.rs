#[cfg(test)]
pub fn initialize_python() -> pyo3::PyResult<()> {
    use pyo3::prelude::*;
    use which;
    use widestring;
    // Due to https://github.com/ContinuumIO/anaconda-issues/issues/11439,
    // we first need to set PYTHONHOME. To do so, we will look for whatever
    // directory on PATH currently has python.exe.
    let python_exe = which::which("python").unwrap();
    let python_home = python_exe.parent().unwrap();
    // The Python C API uses null-terminated UTF-16 strings, so we need to
    // encode the path into that format here.
    // We could use the Windows FFI modules provided in the standard library,
    // but we want this to work cross-platform, so we do things more manually.
    unsafe {
        pyo3::ffi::Py_SetPythonHome(
            widestring::WideCString::from_str(python_home.to_str().unwrap())
                .unwrap()
                .as_ptr(),
        );
    }
    // Once we've set the configuration we need, we can go on and manually
    // initialize PyO3.
    pyo3::prepare_freethreaded_python();
    // Now add cwd to python path
    Python::with_gil::<_, PyResult<_>>(|py| {
        Ok(py
            .import("sys")?
            .getattr("path")?
            .call_method1("insert", (0, std::env::current_dir()?.to_str().unwrap()))?
            .unbind())
    })?;
    Ok(())
}
