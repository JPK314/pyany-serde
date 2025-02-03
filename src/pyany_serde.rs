use pyo3::exceptions::asyncio::InvalidStateError;
use pyo3::prelude::*;

use dyn_clone::{clone_trait_object, DynClone};
use pyo3::types::PyString;

use crate::pyany_serde_impl::{
    get_numpy_dynamic_shape_serde, BoolSerde, BytesSerde, ComplexSerde, DictSerde, DynamicSerde,
    FloatSerde, IntSerde, ListSerde, OptionSerde, PickleSerde, SetSerde, StringSerde, TupleSerde,
    TypedDictSerde,
};
use crate::pyany_serde_type::PyAnySerdeType;

pub trait PyAnySerde: DynClone {
    fn append<'py>(
        &mut self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize>;
    fn retrieve<'py>(
        &mut self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)>;
    fn get_enum(&self) -> &PyAnySerdeType;
    fn get_enum_bytes(&self) -> &[u8];
}

clone_trait_object!(PyAnySerde);

impl TryFrom<PyAnySerdeType> for Box<dyn PyAnySerde> {
    type Error = PyErr;

    fn try_from(value: PyAnySerdeType) -> Result<Self, Self::Error> {
        match value {
            PyAnySerdeType::PICKLE => Ok(Box::new(PickleSerde::new()?)),
            PyAnySerdeType::INT => Ok(Box::new(IntSerde::new())),
            PyAnySerdeType::FLOAT => Ok(Box::new(FloatSerde::new())),
            PyAnySerdeType::COMPLEX => Ok(Box::new(ComplexSerde::new())),
            PyAnySerdeType::BOOLEAN => Ok(Box::new(BoolSerde::new())),
            PyAnySerdeType::STRING => Ok(Box::new(StringSerde::new())),
            PyAnySerdeType::BYTES => Ok(Box::new(BytesSerde::new())),
            PyAnySerdeType::DYNAMIC => Ok(Box::new(DynamicSerde::new()?)),
            PyAnySerdeType::NUMPY { dtype } => Ok(get_numpy_dynamic_shape_serde(dtype)),
            PyAnySerdeType::LIST { items } => Ok(Box::new(ListSerde::new(Some((*items).try_into()?)))),
            PyAnySerdeType::SET { items } => Ok(Box::new(SetSerde::new(Some((*items).try_into()?)))),
            PyAnySerdeType::TUPLE { items } => Ok(Box::new(TupleSerde::new(
                items
                    .into_iter()
                    .map(|item| item.try_into().map(|pyany_serde| Some(pyany_serde)))
                    .collect::<PyResult<_>>()?,
            )?)),
            PyAnySerdeType::DICT { keys, values } => Ok(Box::new(DictSerde::new(
                Some((*keys).try_into()?),
                Some((*values).try_into()?),
            ))),
            PyAnySerdeType::TYPEDDICT { kv_pairs } => Python::with_gil(|py| {
                let serde_kv_list = kv_pairs.into_iter().map(|(key, item_serde)| {
                    item_serde.try_into().map(|pyany_serde| (PyString::new(py, key.as_str()).unbind(), Some(pyany_serde)))
                }).collect::<PyResult<_>>()?;
                Ok(Box::new(TypedDictSerde::new(serde_kv_list)?) as Box<dyn PyAnySerde>)
            }),
            PyAnySerdeType::OPTION { value } => Ok(Box::new(OptionSerde::new(Some((*value).try_into()?)))),
            PyAnySerdeType::OTHER => Err(InvalidStateError::new_err("Tried to deserialize an OTHER type of Serde which cannot be dynamically determined / reconstructed. Ensure the RustSerde used is passed to both the EPI and EP explicitly.")),
        }
    }
}

impl<'py> TryFrom<Bound<'py, PyAny>> for Box<dyn PyAnySerde> {
    type Error = PyErr;

    fn try_from(value: Bound<'py, PyAny>) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl<'py, 'a> TryFrom<&'a Bound<'py, PyAny>> for Box<dyn PyAnySerde> {
    type Error = PyErr;

    fn try_from(value: &'a Bound<'py, PyAny>) -> Result<Self, Self::Error> {
        <&pyo3::Bound<'_, pyo3::PyAny> as TryInto<PyAnySerdeType>>::try_into(value)?.try_into()
    }
}
