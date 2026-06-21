use pyo3::{
    prelude::*,
    types::{PyTuple, PyType},
    IntoPyObjectExt,
};

use crate::{pyany_serde_type::PyAnySerdeTypeKind, PyAnySerdeType};

#[pymethods]
impl PyAnySerdeType {
    fn __reduce__<'py>(
        &self,
        py: Python<'py>,
    ) -> PyResult<(Bound<'py, PyType>, Bound<'py, PyTuple>)> {
        let class = PyAnySerdeTypeKind::from(self).type_object(py);
        let args = match self {
            PyAnySerdeType::DATACLASS {
                clazz,
                init_strategy,
                field_serde_type_dict,
            } => PyTuple::new(
                py,
                [
                    clazz.into_bound_py_any(py)?,
                    init_strategy.clone().into_bound_py_any(py)?,
                    field_serde_type_dict.clone().into_bound_py_any(py)?,
                ],
            )?,
            PyAnySerdeType::DICT {
                keys_serde_type,
                values_serde_type,
            } => PyTuple::new(
                py,
                [
                    keys_serde_type.into_bound_py_any(py)?,
                    values_serde_type.into_bound_py_any(py)?,
                ],
            )?,
            PyAnySerdeType::LIST { items_serde_type } => {
                PyTuple::new(py, [items_serde_type.into_bound_py_any(py)?])?
            }
            PyAnySerdeType::NUMPY { dtype, config } => PyTuple::new(
                py,
                [
                    dtype.into_bound_py_any(py)?,
                    config.clone().into_bound_py_any(py)?,
                ],
            )?,
            PyAnySerdeType::OPTION { value_serde_type } => {
                PyTuple::new(py, [value_serde_type.into_bound_py_any(py)?])?
            }
            PyAnySerdeType::PYTHONSERDE { python_serde } => {
                PyTuple::new(py, [python_serde.into_bound_py_any(py)?])?
            }
            PyAnySerdeType::SET { items_serde_type } => {
                PyTuple::new(py, [items_serde_type.into_bound_py_any(py)?])?
            }
            PyAnySerdeType::TUPLE { item_serde_types } => {
                PyTuple::new(py, [item_serde_types.clone().into_bound_py_any(py)?])?
            }
            PyAnySerdeType::TYPEDDICT {
                key_serde_type_dict,
            } => PyTuple::new(py, [key_serde_type_dict.clone().into_bound_py_any(py)?])?,
            PyAnySerdeType::UNION {
                option_serde_types,
                option_choice_fn,
            } => PyTuple::new(
                py,
                [
                    option_serde_types.clone().into_bound_py_any(py)?,
                    option_choice_fn.into_bound_py_any(py)?,
                ],
            )?,
            _ => PyTuple::empty(py),
        };

        Ok((class, args))
    }
}
