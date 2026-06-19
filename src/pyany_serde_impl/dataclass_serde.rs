use std::collections::HashSet;

use enum_kinds::EnumKind;
use pyo3::exceptions::asyncio::InvalidStateError;
use pyo3::exceptions::PyValueError;
use pyo3::types::{PyDict, PyString, PyTuple, PyType};
use pyo3::{prelude::*, PyTypeInfo};
use strum_macros::{Display, EnumIter};

use crate::communication::{append_string_vec, retrieve_string, retrieve_usize};
use crate::PyAnySerde;

#[derive(Clone)]
pub struct DataclassSerde {
    class: Py<PyAny>,
    init_strategy: InternalInitStrategy,
    field_serde_kv_list: Vec<(Py<PyString>, Box<dyn PyAnySerde>)>,
}

#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct PickleableInitStrategy(pub Option<InitStrategy>);

#[pymethods]
impl PickleableInitStrategy {
    #[new]
    #[pyo3(signature = (*args))]
    fn new<'py>(args: Bound<'py, PyTuple>) -> PyResult<Self> {
        let vec_args = args.iter().collect::<Vec<_>>();
        if vec_args.len() > 1 {
            return Err(PyValueError::new_err(format!(
                "PickleableInitStrategy constructor takes 0 or 1 parameters, received {}",
                args.as_any().repr()?.to_str()?
            )));
        }
        if vec_args.len() == 1 {
            Ok(PickleableInitStrategy(
                vec_args[0].extract::<Option<InitStrategy>>()?,
            ))
        } else {
            Ok(PickleableInitStrategy(None))
        }
    }
    pub fn __getstate__(&self) -> Vec<u8> {
        match self.0.as_ref().unwrap() {
            InitStrategy::ALL {} => vec![0],
            InitStrategy::SOME { kwargs } => {
                let mut bytes = vec![1];
                bytes.extend_from_slice(&kwargs.len().to_ne_bytes());
                for kwarg in kwargs.iter() {
                    append_string_vec(&mut bytes, kwarg);
                }
                bytes
            }
            InitStrategy::NONE {} => vec![2],
        }
    }
    pub fn __setstate__(&mut self, state: Vec<u8>) -> PyResult<()> {
        let buf = &state[..];
        let type_byte = buf[0];
        let mut offset = 1;
        self.0 = Some(match type_byte {
            0 => InitStrategy::ALL {},
            1 => {
                let n_kwargs;
                (n_kwargs, offset) = retrieve_usize(buf, offset)?;
                let mut kwargs = Vec::with_capacity(n_kwargs);
                for _ in 0..n_kwargs {
                    let kwarg;
                    (kwarg, offset) = retrieve_string(buf, offset)?;
                    kwargs.push(kwarg)
                }
                InitStrategy::SOME { kwargs }
            }
            2 => InitStrategy::NONE {},
            v => Err(InvalidStateError::new_err(format!(
                "Got invalid type byte for InitStrategy: {v}"
            )))?,
        });
        Ok(())
    }
}

#[pyclass(from_py_object)]
#[derive(Clone, Debug, PartialEq, Display, EnumKind)]
#[enum_kind(InitStrategyKind, derive(Display, EnumIter))]
pub enum InitStrategy {
    ALL {},
    SOME { kwargs: Vec<String> },
    NONE {},
}

impl InitStrategyKind {
    pub fn type_object<'py>(self, py: Python<'py>) -> Bound<'py, PyType> {
        match self {
            InitStrategyKind::ALL => InitStrategy_ALL::type_object(py),
            InitStrategyKind::SOME => InitStrategy_SOME::type_object(py),
            InitStrategyKind::NONE => InitStrategy_NONE::type_object(py),
        }
    }
    pub fn from_type_object<'py>(to: &Bound<'py, PyType>) -> PyResult<Option<InitStrategyKind>> {
        let py = to.py();
        if to.eq(InitStrategy::type_object(py))? {
            return Ok(None);
        }
        if to.eq(InitStrategy_ALL::type_object(py))? {
            return Ok(Some(InitStrategyKind::ALL));
        }
        if to.eq(InitStrategy_SOME::type_object(py))? {
            return Ok(Some(InitStrategyKind::SOME));
        }
        if to.eq(InitStrategy_NONE::type_object(py))? {
            return Ok(Some(InitStrategyKind::NONE));
        }
        Err(PyValueError::new_err(format!(
            "Unexpected value PyType {}",
            to.repr()?
        )))
    }
}

#[derive(Clone, Debug)]
pub enum InternalInitStrategy {
    ALL(Py<PyDict>),
    SOME(Py<PyDict>, HashSet<usize>),
    NONE,
}

impl DataclassSerde {
    pub fn new(
        class: Py<PyAny>,
        init_strategy: InitStrategy,
        field_serde_kv_list: Vec<(Py<PyString>, Box<dyn PyAnySerde>)>,
    ) -> PyResult<Self> {
        let internal_init_strategy = match &init_strategy {
            InitStrategy::ALL {} => Python::attach::<_, PyResult<_>>(|py| {
                let kwargs_kv_list = field_serde_kv_list
                    .iter()
                    .map(|(field, _)| (field, None::<Py<PyAny>>))
                    .collect::<Vec<_>>();
                let kwargs = PyDict::from_sequence(&kwargs_kv_list.into_pyobject(py)?)?.unbind();
                Ok(InternalInitStrategy::ALL(kwargs))
            })?,
            InitStrategy::SOME { kwargs } => Python::attach::<_, PyResult<_>>(|py| {
                let init_field_idxs = kwargs.iter().map(|init_field| field_serde_kv_list.iter().position(|(field, _)| field.to_string() == *init_field).ok_or_else(|| PyValueError::new_err(format!("field name {} provided in InitStrategy_SOME not contained in field_serde_kv_list", init_field)))).collect::<PyResult<HashSet<_>>>()?;
                let kwargs_kv_list = field_serde_kv_list
                    .iter()
                    .enumerate()
                    .filter(|(idx, _)| init_field_idxs.contains(idx))
                    .map(|(_, (field, _))| (field, None::<Py<PyAny>>))
                    .collect::<Vec<_>>();
                let kwargs = PyDict::from_sequence(&kwargs_kv_list.into_pyobject(py)?)?.unbind();
                Ok(InternalInitStrategy::SOME(kwargs, init_field_idxs))
            })?,
            InitStrategy::NONE {} => InternalInitStrategy::NONE,
        };
        Ok(DataclassSerde {
            class,
            init_strategy: internal_init_strategy,
            field_serde_kv_list,
        })
    }
}

impl PyAnySerde for DataclassSerde {
    fn append<'py>(
        &mut self,
        buf: &mut [u8],
        mut offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        for (field, pyany_serde) in self.field_serde_kv_list.iter_mut() {
            offset = pyany_serde.append(buf, offset, &obj.getattr(&*field)?)?;
        }
        Ok(offset)
    }

    fn append_vec<'py>(
        &mut self,
        v: &mut Vec<u8>,
        start_addr: Option<usize>,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<()> {
        for (field, pyany_serde) in self.field_serde_kv_list.iter_mut() {
            pyany_serde.append_vec(v, start_addr, &obj.getattr(&*field)?)?;
        }
        Ok(())
    }

    fn retrieve<'py>(
        &mut self,
        py: Python<'py>,
        buf: &[u8],
        mut offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let mut kv_list = Vec::with_capacity(self.field_serde_kv_list.len());
        for (field, pyany_serde) in self.field_serde_kv_list.iter_mut() {
            let field_value;
            (field_value, offset) = pyany_serde.retrieve(py, buf, offset)?;
            kv_list.push((field.clone_ref(py).into_bound(py), field_value));
        }
        let class = self.class.bind(py);
        let obj = match &self.init_strategy {
            InternalInitStrategy::ALL(py_kwargs) => {
                let kwargs = py_kwargs.bind(py);
                for (field, field_value) in kv_list.iter() {
                    kwargs.set_item(field, field_value)?;
                }
                class.call((), Some(kwargs))?
            }
            InternalInitStrategy::SOME(py_kwargs, init_field_idxs) => {
                let kwargs = py_kwargs.bind(py);
                let (init_kv_list, other_kv_list) = kv_list
                    .into_iter()
                    .enumerate()
                    .partition::<Vec<_>, _>(|(idx, _)| init_field_idxs.contains(idx));
                for (_, (field, field_value)) in init_kv_list.iter() {
                    kwargs.set_item(field, field_value)?;
                }
                let obj = class.call((), Some(kwargs))?;
                for (_, (field, field_value)) in other_kv_list.iter() {
                    obj.setattr(field, field_value)?;
                }
                obj
            }
            InternalInitStrategy::NONE => {
                let obj = class.call0()?;
                for (field, field_value) in kv_list.iter() {
                    obj.setattr(field, field_value)?;
                }
                obj
            }
        };
        Ok((obj, offset))
    }
}
