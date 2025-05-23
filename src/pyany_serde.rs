use pyo3::prelude::*;
use pyo3::types::PyString;

use dyn_clone::{clone_trait_object, DynClone};

use crate::communication::{append_bool, append_bool_vec, retrieve_bool};
use crate::pyany_serde_impl::{
    get_numpy_serde, BoolSerde, BytesSerde, ComplexSerde, DataclassSerde, DictSerde, DynamicSerde,
    FloatSerde, IntSerde, ListSerde, OptionSerde, PickleSerde, PythonSerdeSerde, SetSerde,
    StringSerde, TupleSerde, TypedDictSerde, UnionSerde,
};
use crate::pyany_serde_type::PyAnySerdeType;
use crate::PickleablePyAnySerdeType;

pub trait PyAnySerde: DynClone {
    fn append<'py>(
        &mut self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize>;
    fn append_vec<'py>(
        &mut self,
        v: &mut Vec<u8>,
        start_addr: Option<usize>,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<()>;
    fn retrieve<'py>(
        &mut self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)>;
    fn append_option<'py>(
        &mut self,
        buf: &mut [u8],
        mut offset: usize,
        obj_option: &Option<&Bound<'py, PyAny>>,
    ) -> PyResult<usize> {
        if let Some(obj) = obj_option {
            offset = append_bool(buf, offset, true);
            offset = self.append(buf, offset, obj)?;
        } else {
            offset = append_bool(buf, offset, false);
        }
        Ok(offset)
    }
    fn append_option_vec<'py>(
        &mut self,
        v: &mut Vec<u8>,
        start_addr: Option<usize>,
        obj_option: &Option<&Bound<'py, PyAny>>,
    ) -> PyResult<()> {
        if let Some(obj) = obj_option {
            append_bool_vec(v, true);
            self.append_vec(v, start_addr, obj)?;
        } else {
            append_bool_vec(v, false);
        }
        Ok(())
    }
    fn retrieve_option<'py>(
        &mut self,
        py: Python<'py>,
        buf: &[u8],
        mut offset: usize,
    ) -> PyResult<(Option<Bound<'py, PyAny>>, usize)> {
        let is_some;
        (is_some, offset) = retrieve_bool(buf, offset)?;
        if is_some {
            let obj;
            (obj, offset) = self.retrieve(py, buf, offset)?;
            Ok((Some(obj), offset))
        } else {
            Ok((None, offset))
        }
    }
}

clone_trait_object!(PyAnySerde);

impl<'py, 'a> TryFrom<&'a Bound<'py, PyAnySerdeType>> for Box<dyn PyAnySerde> {
    type Error = PyErr;

    fn try_from(value: &'a Bound<'py, PyAnySerdeType>) -> Result<Self, Self::Error> {
        value.as_any().extract::<PyAnySerdeType>()?.try_into()
    }
}

impl<'a> TryFrom<&'a Py<PyAnySerdeType>> for Box<dyn PyAnySerde> {
    type Error = PyErr;

    fn try_from(value: &'a Py<PyAnySerdeType>) -> Result<Self, Self::Error> {
        Python::with_gil(|py| value.extract::<PyAnySerdeType>(py)?.try_into())
    }
}

impl TryFrom<PyAnySerdeType> for Box<dyn PyAnySerde> {
    type Error = PyErr;

    fn try_from(value: PyAnySerdeType) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl<'a> TryFrom<&'a PyAnySerdeType> for Box<dyn PyAnySerde> {
    type Error = PyErr;

    fn try_from(value: &'a PyAnySerdeType) -> Result<Self, Self::Error> {
        Ok(match value {
            PyAnySerdeType::BOOL {} => Box::new(BoolSerde {}),
            PyAnySerdeType::BYTES {} => Box::new(BytesSerde {}),
            PyAnySerdeType::COMPLEX {} => Box::new(ComplexSerde {}),
            PyAnySerdeType::DATACLASS {
                clazz,
                init_strategy,
                field_serde_type_dict,
            } => Python::with_gil::<_, PyResult<_>>(|py| {
                Ok(Box::new(DataclassSerde::new(
                    clazz.clone_ref(py),
                    init_strategy.clone(),
                    field_serde_type_dict
                        .iter()
                        .map(|(field, field_serde_type)| {
                            field_serde_type.try_into().map(|pyany_serde| {
                                (PyString::new(py, field.as_str()).unbind(), pyany_serde)
                            })
                        })
                        .collect::<PyResult<_>>()?,
                )?))
            })?,
            PyAnySerdeType::DICT {
                keys_serde_type,
                values_serde_type,
            } => Python::with_gil::<_, PyResult<_>>(|py| {
                Ok(Box::new(DictSerde {
                    keys_serde: keys_serde_type.bind(py).try_into()?,
                    values_serde: values_serde_type.bind(py).try_into()?,
                }))
            })?,
            PyAnySerdeType::DYNAMIC {} => Box::new(DynamicSerde::new()?),
            PyAnySerdeType::FLOAT {} => Box::new(FloatSerde {}),
            PyAnySerdeType::INT {} => Box::new(IntSerde {}),
            PyAnySerdeType::LIST { items_serde_type } => Box::new(ListSerde {
                items_serde: items_serde_type.try_into()?,
            }),
            PyAnySerdeType::NUMPY { dtype, config } => {
                get_numpy_serde(dtype.clone(), config.clone())
            }

            PyAnySerdeType::OPTION { value_serde_type } => Box::new(OptionSerde {
                value_serde: value_serde_type.try_into()?,
            }),
            PyAnySerdeType::PICKLE {} => Box::new(PickleSerde::new()?),
            PyAnySerdeType::PYTHONSERDE { python_serde } => {
                Python::with_gil::<_, PyResult<_>>(|py| {
                    Ok(Box::new(PythonSerdeSerde {
                        python_serde: python_serde.clone_ref(py),
                    }))
                })?
            }
            PyAnySerdeType::SET { items_serde_type } => Box::new(SetSerde {
                items_serde: items_serde_type.try_into()?,
            }),
            PyAnySerdeType::STRING {} => Box::new(StringSerde {}),
            PyAnySerdeType::TUPLE { item_serde_types } => Box::new(TupleSerde {
                item_serdes: item_serde_types
                    .into_iter()
                    .map(|item| item.try_into())
                    .collect::<PyResult<_>>()?,
            }),
            PyAnySerdeType::TYPEDDICT {
                key_serde_type_dict,
            } => Python::with_gil::<_, PyResult<_>>(|py| {
                let serde_kv_list = key_serde_type_dict
                    .into_iter()
                    .map(|(key, item_serde_type)| {
                        item_serde_type.try_into().map(|pyany_serde| {
                            (PyString::new(py, key.as_str()).unbind(), pyany_serde)
                        })
                    })
                    .collect::<PyResult<_>>()?;
                Ok(Box::new(TypedDictSerde { serde_kv_list }) as Box<dyn PyAnySerde>)
            })?,
            PyAnySerdeType::UNION {
                option_serde_types,
                option_choice_fn,
            } => Python::with_gil::<_, PyResult<_>>(|py| {
                Ok(Box::new(UnionSerde {
                    option_serdes: option_serde_types
                        .into_iter()
                        .map(|item| item.try_into())
                        .collect::<PyResult<_>>()?,
                    option_choice_fn: option_choice_fn.clone_ref(py),
                }))
            })?,
        })
    }
}

impl<'py> FromPyObject<'py> for Box<dyn PyAnySerde> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        ob.extract::<PyAnySerdeType>()
            .or_else(|_| {
                ob.extract::<PickleablePyAnySerdeType>()
                    .map(|v| v.0.unwrap().unwrap())
            })?
            .try_into()
    }
}

pub enum DynPyAnySerdeOption {
    Some(Box<dyn PyAnySerde>),
    None,
}

impl From<DynPyAnySerdeOption> for Option<Box<dyn PyAnySerde>> {
    fn from(value: DynPyAnySerdeOption) -> Self {
        match value {
            DynPyAnySerdeOption::Some(pyany_serde) => Some(pyany_serde),
            DynPyAnySerdeOption::None => None,
        }
    }
}

impl<'py> FromPyObject<'py> for DynPyAnySerdeOption {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        ob.extract::<Option<PyAnySerdeType>>()
            .or_else(|_| {
                ob.extract::<PickleablePyAnySerdeType>()
                    .map(|v| v.0.unwrap())
            })
            .map(|pyany_serde_type_option| {
                pyany_serde_type_option
                    .map(|pyany_serde_type| {
                        pyany_serde_type
                            .try_into()
                            .map(|pyany_serde| DynPyAnySerdeOption::Some(pyany_serde))
                    })
                    .unwrap_or(Ok(DynPyAnySerdeOption::None))
            })?
    }
}
