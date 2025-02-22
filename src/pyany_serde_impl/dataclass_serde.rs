use std::collections::HashSet;

use pyo3::exceptions::asyncio::InvalidStateError;
use pyo3::exceptions::PyValueError;
use pyo3::types::{PyCFunction, PyDict, PyString, PyTuple, PyType};
use pyo3::{prelude::*, PyTypeInfo};
use strum_macros::Display;

use crate::communication::{append_string_vec, retrieve_string, retrieve_usize};
use crate::PyAnySerde;

#[derive(Clone)]
pub struct DataclassSerde {
    class: PyObject,
    init_strategy: InternalInitStrategy,
    field_serde_kv_list: Vec<(Py<PyString>, Box<dyn PyAnySerde>)>,
}

#[pyclass]
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

#[pyclass]
#[derive(Clone, Debug, PartialEq, Display)]
pub enum InitStrategy {
    ALL {},
    SOME { kwargs: Vec<String> },
    NONE {},
}

macro_rules! create_union {
    ($handler:expr, $py:expr, $($type:ident),+) => {{
        let mut union_list = Vec::new();
        $(
            union_list.push(
                $handler.call_method1(
                    "generate_schema",
                    (paste::paste! { [<InitStrategy_ $type>]::type_object($py) },)
                )?
            );
        )+
        Ok::<_, PyErr>(union_list)
    }};
}

fn get_enum_subclass_before_validator_fn<'py>(
    cls: &Bound<'py, PyType>,
) -> PyResult<Bound<'py, PyCFunction>> {
    let _py = cls.py();
    let py_cls = cls.clone().unbind();
    let func = move |args: &Bound<'_, PyTuple>,
                     _kwargs: Option<&Bound<'_, PyDict>>|
          -> PyResult<PyObject> {
        let py = args.py();
        let data = args.get_item(0)?;
        let cls = py_cls.bind(py);
        if cls.eq(InitStrategy_ALL::type_object(py))? {
            Ok(InitStrategy::ALL {}.into_pyobject(py)?.into_any().unbind())
        } else if cls.eq(InitStrategy_SOME::type_object(py))? {
            let kwargs = data.get_item("kwargs")?.extract::<Vec<String>>()?;
            Ok(InitStrategy::SOME { kwargs }
                .into_pyobject(py)?
                .into_any()
                .unbind())
        } else if cls.eq(InitStrategy_NONE::type_object(py))? {
            Ok(InitStrategy::NONE {}.into_pyobject(py)?.into_any().unbind())
        } else {
            Err(PyValueError::new_err(format!(
                "Unexpected class: {}",
                cls.repr()?.to_str()?
            )))
        }
    };
    PyCFunction::new_closure(_py, None, None, func)
}

fn get_enum_subclass_typed_dict_schema<'py>(
    cls: &Bound<'py, PyType>,
    core_schema: &Bound<'py, PyAny>,
) -> PyResult<Bound<'py, PyAny>> {
    let py = cls.py();
    let typed_dict_schema = core_schema.getattr("typed_dict_schema")?;
    let typed_dict_field = core_schema.getattr("typed_dict_field")?;
    let str_schema = core_schema.getattr("str_schema")?;
    let list_schema = core_schema.getattr("list_schema")?;
    let cls_name = cls.name()?.to_string();
    let (_, enum_subclass) = cls_name.split_once("_").unwrap();
    let typed_dict_fields = PyDict::new(py);
    typed_dict_fields.set_item(
        "type",
        typed_dict_field.call1((str_schema.call(
            (),
            Some(&PyDict::from_sequence(
                &vec![(
                    "pattern",
                    vec![
                        "^".to_owned(),
                        enum_subclass.to_ascii_lowercase(),
                        "$".to_owned(),
                    ]
                    .join("")
                    .into_pyobject(py)?
                    .into_any(),
                )]
                .into_pyobject(py)?,
            )?),
        )?,))?,
    )?;
    if cls.eq(InitStrategy_SOME::type_object(py))? {
        typed_dict_fields.set_item(
            "kwargs",
            typed_dict_field.call1((list_schema.call1((str_schema.call0()?,))?,))?,
        )?;
    }
    typed_dict_schema.call1((typed_dict_fields,))
}

#[pymethods]
impl InitStrategy {
    // pydantic methods
    #[classmethod]
    fn __get_pydantic_core_schema__<'py>(
        cls: &Bound<'py, PyType>,
        _source_type: Bound<'py, PyAny>,
        handler: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let py = cls.py();
        let core_schema = py.import("pydantic_core")?.getattr("core_schema")?;
        if cls.eq(InitStrategy::type_object(py))? {
            let union_list = create_union!(handler, py, ALL, SOME, NONE)?;
            return core_schema.call_method1("union_schema", (union_list,));
        }
        let python_schema = core_schema.getattr("is_instance_schema")?.call1((cls,))?;
        core_schema.getattr("json_or_python_schema")?.call1((
            core_schema.getattr("chain_schema")?.call1((vec![
                get_enum_subclass_typed_dict_schema(cls, &core_schema)?,
                core_schema
                    .getattr("no_info_before_validator_function")?
                    .call1((get_enum_subclass_before_validator_fn(cls)?, &python_schema))?,
            ],))?,
            python_schema,
        ))
    }

    pub fn to_json(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let data = PyDict::new(py);
            data.set_item("type", self.to_string().to_ascii_lowercase())?;
            if let InitStrategy::SOME { kwargs } = self {
                data.set_item("kwargs", kwargs)?;
            }
            Ok(data.into_any().unbind())
        })
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
        class: PyObject,
        init_strategy: InitStrategy,
        field_serde_kv_list: Vec<(Py<PyString>, Box<dyn PyAnySerde>)>,
    ) -> PyResult<Self> {
        let internal_init_strategy = match &init_strategy {
            InitStrategy::ALL {} => Python::with_gil::<_, PyResult<_>>(|py| {
                let kwargs_kv_list = field_serde_kv_list
                    .iter()
                    .map(|(field, _)| (field, None::<PyObject>))
                    .collect::<Vec<_>>();
                let kwargs = PyDict::from_sequence(&kwargs_kv_list.into_pyobject(py)?)?.unbind();
                Ok(InternalInitStrategy::ALL(kwargs))
            })?,
            InitStrategy::SOME { kwargs } => Python::with_gil::<_, PyResult<_>>(|py| {
                let init_field_idxs = kwargs.iter().map(|init_field| field_serde_kv_list.iter().position(|(field, _)| field.to_string() == *init_field).ok_or_else(|| PyValueError::new_err(format!("field name {} provided in InitStrategy_SOME not contained in field_serde_kv_list", init_field)))).collect::<PyResult<HashSet<_>>>()?;
                let kwargs_kv_list = field_serde_kv_list
                    .iter()
                    .enumerate()
                    .filter(|(idx, _)| init_field_idxs.contains(idx))
                    .map(|(_, (field, _))| (field, None::<PyObject>))
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
        &self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        let mut offset = offset;
        for (field, pyany_serde) in self.field_serde_kv_list.iter() {
            offset = pyany_serde.append(buf, offset, &obj.getattr(field)?)?;
        }
        Ok(offset)
    }

    fn retrieve<'py>(
        &self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let mut kv_list = Vec::with_capacity(self.field_serde_kv_list.len());
        let mut offset = offset;
        for (field, pyany_serde) in self.field_serde_kv_list.iter() {
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
