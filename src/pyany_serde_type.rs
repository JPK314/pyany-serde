use enum_kinds::EnumKind;
use pyo3::exceptions::PyValueError;
use pyo3::types::PyGenericAlias;
use pyo3::types::{PyFunction, PyType};
use pyo3::{prelude::*, PyTypeInfo};
use std::collections::BTreeMap;
use strum_macros::{Display, EnumIter};

use crate::common::NumpyDtype;
use crate::pyany_serde_impl::{InitStrategy, NumpySerdeConfig};

#[pyclass(from_py_object)]
#[derive(Debug, Clone, Display, EnumKind)]
#[enum_kind(PyAnySerdeTypeKind, derive(Display, EnumIter))]
pub enum PyAnySerdeType {
    BOOL {},
    BYTES {},
    COMPLEX {},
    DATACLASS {
        clazz: Py<PyAny>,
        init_strategy: InitStrategy,
        field_serde_type_dict: BTreeMap<String, PyAnySerdeType>,
    },
    DICT {
        keys_serde_type: Py<PyAnySerdeType>,
        values_serde_type: Py<PyAnySerdeType>,
    },
    DYNAMIC {},
    FLOAT {},
    INT {},
    LIST {
        items_serde_type: Py<PyAnySerdeType>,
    },
    #[pyo3(constructor = (dtype, config = NumpySerdeConfig::DYNAMIC { preprocessor_fn: None, postprocessor_fn: None }))]
    NUMPY {
        dtype: NumpyDtype,
        config: NumpySerdeConfig,
    },
    OPTION {
        value_serde_type: Py<PyAnySerdeType>,
    },
    PICKLE {},
    PYTHONSERDE {
        python_serde: Py<PyAny>,
    },
    SET {
        items_serde_type: Py<PyAnySerdeType>,
    },
    STRING {},
    TUPLE {
        item_serde_types: Vec<PyAnySerdeType>,
    },
    TYPEDDICT {
        key_serde_type_dict: BTreeMap<String, PyAnySerdeType>,
    },
    UNION {
        option_serde_types: Vec<PyAnySerdeType>,
        option_choice_fn: Py<PyFunction>,
    },
}

impl PyAnySerdeTypeKind {
    pub fn type_object<'py>(self, py: Python<'py>) -> Bound<'py, PyType> {
        match self {
            PyAnySerdeTypeKind::BOOL => PyAnySerdeType_BOOL::type_object(py),
            PyAnySerdeTypeKind::BYTES => PyAnySerdeType_BYTES::type_object(py),
            PyAnySerdeTypeKind::COMPLEX => PyAnySerdeType_COMPLEX::type_object(py),
            PyAnySerdeTypeKind::DATACLASS => PyAnySerdeType_DATACLASS::type_object(py),
            PyAnySerdeTypeKind::DICT => PyAnySerdeType_DICT::type_object(py),
            PyAnySerdeTypeKind::DYNAMIC => PyAnySerdeType_DYNAMIC::type_object(py),
            PyAnySerdeTypeKind::FLOAT => PyAnySerdeType_FLOAT::type_object(py),
            PyAnySerdeTypeKind::INT => PyAnySerdeType_INT::type_object(py),
            PyAnySerdeTypeKind::LIST => PyAnySerdeType_LIST::type_object(py),
            PyAnySerdeTypeKind::NUMPY => PyAnySerdeType_NUMPY::type_object(py),
            PyAnySerdeTypeKind::OPTION => PyAnySerdeType_OPTION::type_object(py),
            PyAnySerdeTypeKind::PICKLE => PyAnySerdeType_PICKLE::type_object(py),
            PyAnySerdeTypeKind::PYTHONSERDE => PyAnySerdeType_PYTHONSERDE::type_object(py),
            PyAnySerdeTypeKind::SET => PyAnySerdeType_SET::type_object(py),
            PyAnySerdeTypeKind::STRING => PyAnySerdeType_STRING::type_object(py),
            PyAnySerdeTypeKind::TUPLE => PyAnySerdeType_TUPLE::type_object(py),
            PyAnySerdeTypeKind::TYPEDDICT => PyAnySerdeType_TYPEDDICT::type_object(py),
            PyAnySerdeTypeKind::UNION => PyAnySerdeType_UNION::type_object(py),
        }
    }
    pub fn from_type_object<'py>(to: &Bound<'py, PyType>) -> PyResult<Option<PyAnySerdeTypeKind>> {
        let py = to.py();
        if to.eq(PyAnySerdeType::type_object(py))? {
            return Ok(None);
        }
        if to.eq(PyAnySerdeType_BOOL::type_object(py))? {
            return Ok(Some(PyAnySerdeTypeKind::BOOL));
        }
        if to.eq(PyAnySerdeType_BYTES::type_object(py))? {
            return Ok(Some(PyAnySerdeTypeKind::BYTES));
        }
        if to.eq(PyAnySerdeType_COMPLEX::type_object(py))? {
            return Ok(Some(PyAnySerdeTypeKind::COMPLEX));
        }
        if to.eq(PyAnySerdeType_DATACLASS::type_object(py))? {
            return Ok(Some(PyAnySerdeTypeKind::DATACLASS));
        }
        if to.eq(PyAnySerdeType_DICT::type_object(py))? {
            return Ok(Some(PyAnySerdeTypeKind::DICT));
        }
        if to.eq(PyAnySerdeType_DYNAMIC::type_object(py))? {
            return Ok(Some(PyAnySerdeTypeKind::DYNAMIC));
        }
        if to.eq(PyAnySerdeType_FLOAT::type_object(py))? {
            return Ok(Some(PyAnySerdeTypeKind::FLOAT));
        }
        if to.eq(PyAnySerdeType_INT::type_object(py))? {
            return Ok(Some(PyAnySerdeTypeKind::INT));
        }
        if to.eq(PyAnySerdeType_LIST::type_object(py))? {
            return Ok(Some(PyAnySerdeTypeKind::LIST));
        }
        if to.eq(PyAnySerdeType_NUMPY::type_object(py))? {
            return Ok(Some(PyAnySerdeTypeKind::NUMPY));
        }
        if to.eq(PyAnySerdeType_OPTION::type_object(py))? {
            return Ok(Some(PyAnySerdeTypeKind::OPTION));
        }
        if to.eq(PyAnySerdeType_PICKLE::type_object(py))? {
            return Ok(Some(PyAnySerdeTypeKind::PICKLE));
        }
        if to.eq(PyAnySerdeType_PYTHONSERDE::type_object(py))? {
            return Ok(Some(PyAnySerdeTypeKind::PYTHONSERDE));
        }
        if to.eq(PyAnySerdeType_SET::type_object(py))? {
            return Ok(Some(PyAnySerdeTypeKind::SET));
        }
        if to.eq(PyAnySerdeType_STRING::type_object(py))? {
            return Ok(Some(PyAnySerdeTypeKind::STRING));
        }
        if to.eq(PyAnySerdeType_TUPLE::type_object(py))? {
            return Ok(Some(PyAnySerdeTypeKind::TUPLE));
        }
        if to.eq(PyAnySerdeType_TYPEDDICT::type_object(py))? {
            return Ok(Some(PyAnySerdeTypeKind::TYPEDDICT));
        }
        if to.eq(PyAnySerdeType_UNION::type_object(py))? {
            return Ok(Some(PyAnySerdeTypeKind::UNION));
        }
        Err(PyValueError::new_err(format!(
            "Unexpected value PyType {}",
            to.repr()?
        )))
    }
}

#[pymethods]
impl PyAnySerdeType {
    // python generics support
    #[classmethod]
    #[pyo3(signature = (key, /))]
    fn __class_getitem__<'py>(
        cls: &Bound<'py, PyType>,
        key: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        Ok(PyGenericAlias::new(cls.py(), cls.as_any(), key)?.into_any())
    }
}
