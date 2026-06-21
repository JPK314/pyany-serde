use pyo3::{
    prelude::*,
    types::{PyTuple, PyType},
    IntoPyObjectExt,
};

use crate::pyany_serde_impl::{NumpySerdeConfig, NumpySerdeConfigKind};

#[pymethods]
impl NumpySerdeConfig {
    fn __reduce__<'py>(
        &self,
        py: Python<'py>,
    ) -> PyResult<(Bound<'py, PyType>, Bound<'py, PyTuple>)> {
        let class = NumpySerdeConfigKind::from(self).type_object(py);
        match self {
            NumpySerdeConfig::DYNAMIC {
                preprocessor_fn,
                postprocessor_fn,
            } => Ok((
                class,
                PyTuple::new(py, [preprocessor_fn, postprocessor_fn])?,
            )),
            NumpySerdeConfig::STATIC {
                shape,
                preprocessor_fn,
                postprocessor_fn,
                allocation_pool_min_size,
                allocation_pool_max_size,
                allocation_pool_warning_size,
            } => Ok((
                class,
                PyTuple::new(
                    py,
                    [
                        shape.into_pyobject(py)?,
                        preprocessor_fn.into_pyobject(py)?,
                        postprocessor_fn.into_pyobject(py)?,
                        allocation_pool_min_size.into_bound_py_any(py)?,
                        allocation_pool_max_size.into_pyobject(py)?,
                        allocation_pool_warning_size.into_pyobject(py)?,
                    ],
                )?,
            )),
        }
    }
}
