use numpy::PyArrayDescr;
use pyo3::{
    prelude::*,
    types::{PyFunction, PyString},
};

use crate::{
    common::numpy_dtype_enum::get_numpy_dtype,
    dyn_pyany_serde::DynPyAnySerde,
    pyany_serde_impl::{
        bool_serde::BoolSerde, bytes_serde::BytesSerde, complex_serde::ComplexSerde,
        dict_serde::DictSerde, dynamic_serde::DynamicSerde, float_serde::FloatSerde,
        int_serde::IntSerde, list_serde::ListSerde,
        numpy_dynamic_shape_serde::get_numpy_dynamic_shape_serde, option_serde::OptionSerde,
        pickle_serde::PickleSerde, python_serde_serde::PythonSerdeSerde, set_serde::SetSerde,
        string_serde::StringSerde, tuple_serde::TupleSerde, typed_dict_serde::TypedDictSerde,
        union_serde::UnionSerde,
    },
};

#[pyclass]
pub struct DynPyAnySerdeFactory;

#[pymethods]
impl DynPyAnySerdeFactory {
    #[staticmethod]
    pub fn bool_serde() -> DynPyAnySerde {
        DynPyAnySerde(Some(Box::new(BoolSerde::new())))
    }
    #[staticmethod]
    pub fn bytes_serde() -> DynPyAnySerde {
        DynPyAnySerde(Some(Box::new(BytesSerde::new())))
    }
    #[staticmethod]
    pub fn complex_serde() -> DynPyAnySerde {
        DynPyAnySerde(Some(Box::new(ComplexSerde::new())))
    }
    #[staticmethod]
    #[pyo3(signature = (key_serde_option, value_serde_option))]
    pub fn dict_serde<'py>(
        key_serde_option: Option<DynPyAnySerde>,
        value_serde_option: Option<DynPyAnySerde>,
    ) -> DynPyAnySerde {
        DynPyAnySerde(Some(Box::new(DictSerde::new(
            key_serde_option.map(|dyn_serde| dyn_serde.0.as_ref().unwrap().clone()),
            value_serde_option.map(|dyn_serde| dyn_serde.0.as_ref().unwrap().clone()),
        ))))
    }
    #[staticmethod]
    pub fn dynamic_serde() -> PyResult<DynPyAnySerde> {
        Ok(DynPyAnySerde(Some(Box::new(DynamicSerde::new()?))))
    }
    #[staticmethod]
    pub fn float_serde() -> DynPyAnySerde {
        DynPyAnySerde(Some(Box::new(FloatSerde::new())))
    }
    #[staticmethod]
    pub fn int_serde() -> DynPyAnySerde {
        DynPyAnySerde(Some(Box::new(IntSerde::new())))
    }
    #[staticmethod]
    #[pyo3(signature = (items_serde_option))]
    pub fn list_serde(items_serde_option: Option<DynPyAnySerde>) -> DynPyAnySerde {
        DynPyAnySerde(Some(Box::new(ListSerde::new(
            items_serde_option.map(|dyn_serde| dyn_serde.0.as_ref().unwrap().clone()),
        ))))
    }
    #[staticmethod]
    pub fn numpy_dynamic_shape_serde(py_dtype: Py<PyArrayDescr>) -> PyResult<DynPyAnySerde> {
        Ok(DynPyAnySerde(Some(get_numpy_dynamic_shape_serde(
            get_numpy_dtype(py_dtype)?,
        ))))
    }
    #[staticmethod]
    #[pyo3(signature = (value_serde_option))]
    pub fn option_serde(value_serde_option: Option<DynPyAnySerde>) -> DynPyAnySerde {
        DynPyAnySerde(Some(Box::new(OptionSerde::new(
            value_serde_option.map(|dyn_serde| dyn_serde.0.as_ref().unwrap().clone()),
        ))))
    }
    #[staticmethod]
    pub fn python_serde_serde(python_serde: PyObject) -> PyResult<DynPyAnySerde> {
        Ok(DynPyAnySerde(Some(Box::new(PythonSerdeSerde::new(
            python_serde,
        )))))
    }
    #[staticmethod]
    pub fn pickle_serde() -> PyResult<DynPyAnySerde> {
        Ok(DynPyAnySerde(Some(Box::new(PickleSerde::new()?))))
    }
    #[staticmethod]
    #[pyo3(signature = (items_serde_option))]
    pub fn set_serde(items_serde_option: Option<DynPyAnySerde>) -> DynPyAnySerde {
        DynPyAnySerde(Some(Box::new(SetSerde::new(
            items_serde_option.map(|dyn_serde| dyn_serde.0.as_ref().unwrap().clone()),
        ))))
    }
    #[staticmethod]
    pub fn string_serde() -> DynPyAnySerde {
        DynPyAnySerde(Some(Box::new(StringSerde::new())))
    }
    #[staticmethod]
    pub fn tuple_serde(item_serdes: Vec<Option<DynPyAnySerde>>) -> PyResult<DynPyAnySerde> {
        Ok(DynPyAnySerde(Some(Box::new(TupleSerde::new(
            item_serdes
                .into_iter()
                .map(|dyn_serde_option| dyn_serde_option.map(|dyn_serde| dyn_serde.0.unwrap()))
                .collect(),
        )?))))
    }
    #[staticmethod]
    pub fn typed_dict_serde(
        serde_kv_list: Vec<(Py<PyString>, Option<DynPyAnySerde>)>,
    ) -> PyResult<DynPyAnySerde> {
        Ok(DynPyAnySerde(Some(Box::new(TypedDictSerde::new(
            serde_kv_list
                .into_iter()
                .map(|(key, dyn_serde_option)| {
                    (key, dyn_serde_option.map(|dyn_serde| dyn_serde.0.unwrap()))
                })
                .collect(),
        )?))))
    }
    #[staticmethod]
    pub fn union_serde(
        serde_options: Vec<Option<DynPyAnySerde>>,
        serde_choice_fn: Py<PyFunction>,
    ) -> DynPyAnySerde {
        DynPyAnySerde(Some(Box::new(UnionSerde::new(
            serde_options
                .into_iter()
                .map(|dyn_serde_option| dyn_serde_option.map(|dyn_serde| dyn_serde.0.unwrap()))
                .collect(),
            serde_choice_fn,
        ))))
    }
}
