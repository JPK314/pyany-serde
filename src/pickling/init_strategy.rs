use pyo3::{
    prelude::*,
    types::{PyTuple, PyType},
};

use crate::pyany_serde_impl::{InitStrategy, InitStrategyKind};

#[pymethods]
impl InitStrategy {
    fn __reduce__<'py>(
        &self,
        py: Python<'py>,
    ) -> PyResult<(Bound<'py, PyType>, Bound<'py, PyTuple>)> {
        let class = InitStrategyKind::from(self).type_object(py);
        let args = match self {
            InitStrategy::SOME { kwargs } => PyTuple::new(py, [kwargs])?,
            _ => PyTuple::empty(py),
        };
        Ok((class, args))
    }
}
