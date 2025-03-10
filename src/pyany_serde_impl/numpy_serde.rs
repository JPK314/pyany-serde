use bytemuck::{cast_slice, AnyBitPattern, NoUninit};
use numpy::ndarray::ArrayD;
use numpy::{Element, PyArrayDyn, PyArrayMethods, PyUntypedArrayMethods};
use numpy::{IntoPyArray, PyArray};
use pyo3::exceptions::asyncio::InvalidStateError;
use pyo3::exceptions::PyValueError;
use pyo3::types::{PyBytes, PyCFunction, PyDict, PyTuple, PyType};
use pyo3::{prelude::*, PyTypeInfo};
use strum_macros::Display;

use crate::communication::{
    append_bool_vec, append_bytes_vec, append_usize, append_usize_vec, retrieve_bool,
    retrieve_usize,
};
use crate::{
    common::{get_bytes_to_alignment, NumpyDtype},
    communication::{append_bytes, retrieve_bytes},
    PyAnySerde,
};

fn append_usize_option_vec(v: &mut Vec<u8>, val_option: &Option<usize>) {
    if let Some(val) = val_option {
        append_bool_vec(v, true);
        append_usize_vec(v, *val);
    } else {
        append_bool_vec(v, false);
    }
}

fn retrieve_usize_option(buf: &[u8], mut offset: usize) -> PyResult<(Option<usize>, usize)> {
    let has_val;
    (has_val, offset) = retrieve_bool(buf, offset)?;
    if has_val {
        let val;
        (val, offset) = retrieve_usize(buf, offset)?;
        Ok((Some(val), offset))
    } else {
        Ok((None, offset))
    }
}

fn append_python_pkl_option_vec(v: &mut Vec<u8>, obj_option: &Option<PyObject>) -> PyResult<()> {
    if let Some(obj) = obj_option {
        append_bool_vec(v, true);
        Python::with_gil::<_, PyResult<_>>(|py| {
            let preprocessor_fn_py_bytes = py
                .import("pickle")?
                .getattr("dumps")?
                .call1((obj,))?
                .downcast_into::<PyBytes>()?;
            append_bytes_vec(v, preprocessor_fn_py_bytes.as_bytes());
            Ok(())
        })?;
    } else {
        append_bool_vec(v, false);
    }
    Ok(())
}

fn retrieve_python_pkl_option(
    buf: &[u8],
    mut offset: usize,
) -> PyResult<(Option<PyObject>, usize)> {
    let has_obj;
    (has_obj, offset) = retrieve_bool(buf, offset)?;
    if has_obj {
        Python::with_gil::<_, PyResult<_>>(|py| {
            let obj_bytes;
            (obj_bytes, offset) = retrieve_bytes(buf, offset)?;
            Ok((
                Some(
                    py.import("pickle")?
                        .getattr("loads")?
                        .call1((PyBytes::new(py, obj_bytes).into_pyobject(py)?,))?
                        .unbind(),
                ),
                offset,
            ))
        })
    } else {
        Ok((None, offset))
    }
}

#[pyclass]
#[derive(Clone)]
pub struct PickleableNumpySerdeConfig(pub Option<NumpySerdeConfig>);

#[pymethods]
impl PickleableNumpySerdeConfig {
    #[new]
    #[pyo3(signature = (*args))]
    fn new<'py>(args: Bound<'py, PyTuple>) -> PyResult<Self> {
        let vec_args = args.iter().collect::<Vec<_>>();
        if vec_args.len() > 1 {
            return Err(PyValueError::new_err(format!(
                "PickleableNumpySerdeConfig constructor takes 0 or 1 parameters, received {}",
                args.as_any().repr()?.to_str()?
            )));
        }
        if vec_args.len() == 1 {
            Ok(PickleableNumpySerdeConfig(
                vec_args[0].extract::<Option<NumpySerdeConfig>>()?,
            ))
        } else {
            Ok(PickleableNumpySerdeConfig(None))
        }
    }
    pub fn __getstate__(&self) -> PyResult<Vec<u8>> {
        Ok(match self.0.as_ref().unwrap() {
            NumpySerdeConfig::DYNAMIC {
                preprocessor_fn,
                postprocessor_fn,
            } => {
                let mut bytes = vec![0];
                append_python_pkl_option_vec(&mut bytes, preprocessor_fn)?;
                append_python_pkl_option_vec(&mut bytes, postprocessor_fn)?;
                bytes
            }
            NumpySerdeConfig::STATIC {
                preprocessor_fn,
                postprocessor_fn,
                shape,
                allocation_pool_min_size,
                allocation_pool_max_size,
            } => {
                let mut bytes = vec![1];
                append_python_pkl_option_vec(&mut bytes, preprocessor_fn)?;
                append_python_pkl_option_vec(&mut bytes, postprocessor_fn)?;
                append_usize_vec(&mut bytes, shape.len());
                for &dim in shape.iter() {
                    append_usize_vec(&mut bytes, dim);
                }
                append_usize_vec(&mut bytes, *allocation_pool_min_size);
                append_usize_option_vec(&mut bytes, allocation_pool_max_size);
                bytes
            }
        })
    }
    pub fn __setstate__(&mut self, state: Vec<u8>) -> PyResult<()> {
        let buf = &state[..];
        let type_byte = buf[0];
        let mut offset = 1;
        self.0 = Some(match type_byte {
            0 => {
                let preprocessor_fn;
                (preprocessor_fn, offset) = retrieve_python_pkl_option(buf, offset)?;
                let postprocessor_fn;
                (postprocessor_fn, _) = retrieve_python_pkl_option(buf, offset)?;
                NumpySerdeConfig::DYNAMIC {
                    preprocessor_fn,
                    postprocessor_fn,
                }
            }
            1 => {
                let preprocessor_fn;
                (preprocessor_fn, offset) = retrieve_python_pkl_option(buf, offset)?;
                let postprocessor_fn;
                (postprocessor_fn, offset) = retrieve_python_pkl_option(buf, offset)?;
                let shape_len;
                (shape_len, offset) = retrieve_usize(buf, offset)?;
                let mut shape = Vec::with_capacity(shape_len);
                for _ in 0..shape_len {
                    let dim;
                    (dim, offset) = retrieve_usize(buf, offset)?;
                    shape.push(dim);
                }
                let allocation_pool_min_size;
                (allocation_pool_min_size, offset) = retrieve_usize(buf, offset)?;
                let allocation_pool_max_size;
                (allocation_pool_max_size, _) = retrieve_usize_option(buf, offset)?;
                NumpySerdeConfig::STATIC {
                    preprocessor_fn,
                    postprocessor_fn,
                    shape,
                    allocation_pool_min_size,
                    allocation_pool_max_size,
                }
            }
            v => Err(InvalidStateError::new_err(format!(
                "Got invalid type byte for NumpySerdeConfig: {v}"
            )))?,
        });
        Ok(())
    }
}

#[pyclass]
#[derive(Debug, Clone, Display)]
pub enum NumpySerdeConfig {
    #[pyo3(constructor = (preprocessor_fn = None, postprocessor_fn = None))]
    DYNAMIC {
        preprocessor_fn: Option<PyObject>,
        postprocessor_fn: Option<PyObject>,
    },
    #[pyo3(constructor = (shape, preprocessor_fn = None, postprocessor_fn = None, allocation_pool_min_size = 0, allocation_pool_max_size = None))]
    STATIC {
        shape: Vec<usize>,
        preprocessor_fn: Option<PyObject>,
        postprocessor_fn: Option<PyObject>,
        allocation_pool_min_size: usize,
        allocation_pool_max_size: Option<usize>,
    },
}

macro_rules! create_union {
    ($handler:expr, $py:expr, $($type:ident),+) => {{
        let mut union_list = Vec::new();
        $(
            union_list.push(
                $handler.call_method1(
                    "generate_schema",
                    (paste::paste! { [<NumpySerdeConfig_ $type>]::type_object($py) },)
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
        let preprocessor_fn_hex_option = data
            .get_item("preprocessor_fn_pkl")?
            .extract::<Option<String>>()?;
        let preprocessor_fn_option = preprocessor_fn_hex_option
            .map(|preprocessor_fn_hex| {
                Ok::<_, PyErr>(
                    py.import("pickle")?
                        .getattr("loads")?
                        .call1((PyBytes::new(
                            py,
                            &hex::decode(preprocessor_fn_hex.as_str()).map_err(|err| {
                                PyValueError::new_err(format!(
                                    "python_serde_pkl could not be decoded from hex into bytes: {}",
                                    err.to_string()
                                ))
                            })?,
                        ),))?
                        .unbind(),
                )
            })
            .transpose()?;
        let postprocessor_fn_hex_option = data
            .get_item("postprocessor_fn_pkl")?
            .extract::<Option<String>>()?;
        let postprocessor_fn_option = postprocessor_fn_hex_option
            .map(|postprocessor_fn_hex| {
                Ok::<_, PyErr>(
                    py.import("pickle")?
                        .getattr("loads")?
                        .call1((PyBytes::new(
                            py,
                            &hex::decode(postprocessor_fn_hex.as_str()).map_err(|err| {
                                PyValueError::new_err(format!(
                                    "python_serde_pkl could not be decoded from hex into bytes: {}",
                                    err.to_string()
                                ))
                            })?,
                        ),))?
                        .unbind(),
                )
            })
            .transpose()?;
        if cls.eq(NumpySerdeConfig_DYNAMIC::type_object(py))? {
            Ok(NumpySerdeConfig::DYNAMIC {
                preprocessor_fn: preprocessor_fn_option,
                postprocessor_fn: postprocessor_fn_option,
            }
            .into_pyobject(py)?
            .into_any()
            .unbind())
        } else if cls.eq(NumpySerdeConfig_STATIC::type_object(py))? {
            let shape = data.get_item("shape")?.extract::<Vec<usize>>()?;
            let allocation_pool_min_size = data
                .get_item("allocation_pool_min_size")?
                .extract::<usize>()?;
            let allocation_pool_max_size = data
                .get_item("allocation_pool_max_size")?
                .extract::<Option<usize>>()?;
            if allocation_pool_max_size.is_some()
                && allocation_pool_min_size > allocation_pool_max_size.unwrap()
            {
                Err(PyValueError::new_err(format!(
                    "Validation error: allocation_pool_min_size ({}) cannot be greater than allocation_pool_max_size ({})", allocation_pool_min_size, allocation_pool_max_size.unwrap()
                )))?
            }
            Ok(NumpySerdeConfig::STATIC {
                preprocessor_fn: preprocessor_fn_option,
                postprocessor_fn: postprocessor_fn_option,
                shape,
                allocation_pool_min_size,
                allocation_pool_max_size,
            }
            .into_pyobject(py)?
            .into_any()
            .unbind())
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
    let int_schema = core_schema.getattr("int_schema")?;
    let str_schema = core_schema.getattr("str_schema")?;
    let list_schema = core_schema.getattr("list_schema")?;
    let nullable_schema = core_schema.getattr("nullable_schema")?;
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
    typed_dict_fields.set_item(
        "preprocessor_fn_pkl",
        typed_dict_field.call1((nullable_schema.call1((str_schema.call0()?,))?,))?,
    )?;
    typed_dict_fields.set_item(
        "postprocessor_fn_pkl",
        typed_dict_field.call1((nullable_schema.call1((str_schema.call0()?,))?,))?,
    )?;
    if cls.eq(NumpySerdeConfig_STATIC::type_object(py))? {
        typed_dict_fields.set_item(
            "shape",
            typed_dict_field.call1((list_schema.call1((int_schema.call(
                (),
                Some(&PyDict::from_sequence(&vec![("ge", 0)].into_pyobject(py)?)?),
            )?,))?,))?,
        )?;
        typed_dict_fields.set_item(
            "allocation_pool_min_size",
            typed_dict_field.call1((int_schema.call(
                (),
                Some(&PyDict::from_sequence(&vec![("ge", 0)].into_pyobject(py)?)?),
            )?,))?,
        )?;
        typed_dict_fields.set_item(
            "allocation_pool_max_size",
            typed_dict_field.call1((nullable_schema.call1((int_schema.call(
                (),
                Some(&PyDict::from_sequence(&vec![("ge", 0)].into_pyobject(py)?)?),
            )?,))?,))?,
        )?;
    }
    typed_dict_schema.call1((typed_dict_fields,))
}

#[pymethods]
impl NumpySerdeConfig {
    // pydantic methods
    #[classmethod]
    fn __get_pydantic_core_schema__<'py>(
        cls: &Bound<'py, PyType>,
        _source_type: Bound<'py, PyAny>,
        handler: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let py = cls.py();
        let core_schema = py.import("pydantic_core")?.getattr("core_schema")?;
        if cls.eq(NumpySerdeConfig::type_object(py))? {
            let union_list = create_union!(handler, py, DYNAMIC, STATIC)?;
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
            match self {
                NumpySerdeConfig::DYNAMIC {
                    preprocessor_fn,
                    postprocessor_fn,
                } => {
                    data.set_item(
                        "preprocessor_fn_pkl",
                        py.import("pickle")?
                            .getattr("dumps")?
                            .call1((preprocessor_fn,))?
                            .call_method0("hex")?,
                    )?;
                    data.set_item(
                        "postprocessor_fn_pkl",
                        py.import("pickle")?
                            .getattr("dumps")?
                            .call1((postprocessor_fn,))?
                            .call_method0("hex")?,
                    )?;
                }
                NumpySerdeConfig::STATIC {
                    preprocessor_fn,
                    postprocessor_fn,
                    shape,
                    allocation_pool_min_size,
                    allocation_pool_max_size,
                } => {
                    data.set_item(
                        "preprocessor_fn_pkl",
                        py.import("pickle")?
                            .getattr("dumps")?
                            .call1((preprocessor_fn,))?
                            .call_method0("hex")?,
                    )?;
                    data.set_item(
                        "postprocessor_fn_pkl",
                        py.import("pickle")?
                            .getattr("dumps")?
                            .call1((postprocessor_fn,))?
                            .call_method0("hex")?,
                    )?;
                    data.set_item("shape", shape)?;
                    data.set_item("allocation_pool_min_size", allocation_pool_min_size)?;
                    data.set_item("allocation_pool_max_size", allocation_pool_max_size)?;
                }
            }
            Ok(data.into_any().unbind())
        })
    }
}

#[derive(Clone)]
pub struct NumpySerde<T: Element> {
    pub config: NumpySerdeConfig,
    pub allocation_pool: Vec<Py<PyArrayDyn<T>>>,
}

impl<T: Element + AnyBitPattern + NoUninit> NumpySerde<T> {
    pub fn append_inner<'py>(
        &mut self,
        buf: &mut [u8],
        mut offset: usize,
        array: &Bound<'py, PyArrayDyn<T>>,
    ) -> PyResult<usize> {
        match &self.config {
            NumpySerdeConfig::DYNAMIC { .. } => {
                let shape = array.shape();
                offset = append_usize(buf, offset, shape.len());
                for dim in shape.iter() {
                    offset = append_usize(buf, offset, *dim);
                }
                let obj_vec = array.to_vec()?;
                offset = offset + get_bytes_to_alignment::<T>(buf.as_ptr() as usize + offset);
                offset = append_bytes(buf, offset, cast_slice::<T, u8>(&obj_vec))?;
            }
            NumpySerdeConfig::STATIC { .. } => {
                let obj_vec = array.to_vec()?;
                offset = offset + get_bytes_to_alignment::<T>(buf.as_ptr() as usize + offset);
                offset = append_bytes(buf, offset, cast_slice::<T, u8>(&obj_vec))?;
            }
        }
        Ok(offset)
    }

    pub fn retrieve_inner<'py>(
        &mut self,
        py: Python<'py>,
        buf: &[u8],
        mut offset: usize,
    ) -> PyResult<(Bound<'py, PyArrayDyn<T>>, usize)> {
        let py_array = match &self.config {
            NumpySerdeConfig::DYNAMIC { .. } => {
                let shape_len;
                (shape_len, offset) = retrieve_usize(buf, offset)?;
                let mut shape = Vec::with_capacity(shape_len);
                for _ in 0..shape_len {
                    let dim;
                    (dim, offset) = retrieve_usize(buf, offset)?;
                    shape.push(dim);
                }
                offset = offset + get_bytes_to_alignment::<T>(buf.as_ptr() as usize + offset);
                let obj_bytes;
                (obj_bytes, offset) = retrieve_bytes(buf, offset)?;
                let array_vec = cast_slice::<u8, T>(obj_bytes).to_vec();
                ArrayD::from_shape_vec(shape, array_vec)
                    .map_err(|err| {
                        InvalidStateError::new_err(format!(
                            "Failed create Numpy array of T from shape and Vec<T>: {}",
                            err
                        ))
                    })?
                    .into_pyarray(py)
            }
            NumpySerdeConfig::STATIC {
                shape,
                allocation_pool_min_size,
                allocation_pool_max_size,
                ..
            } => {
                offset = offset + get_bytes_to_alignment::<T>(buf.as_ptr() as usize + offset);
                let obj_bytes;
                (obj_bytes, offset) = retrieve_bytes(buf, offset)?;
                let array_vec = cast_slice::<u8, T>(obj_bytes).to_vec();
                let py_array;
                if allocation_pool_max_size.is_none() || allocation_pool_max_size.unwrap() > 0 {
                    // Take two random elements from the pool
                    let pool_size = self.allocation_pool.len();
                    let idx1 = fastrand::usize(..pool_size);
                    let idx2 = fastrand::usize(..pool_size);
                    let e1 = &self.allocation_pool[idx1];
                    let e2 = &self.allocation_pool[idx2];
                    let e1_free = e1.get_refcnt(py) > 1;
                    let e2_free = e2.get_refcnt(py) > 1;
                    if e1_free && e2_free {
                        py_array = e1.clone_ref(py).into_bound(py);
                        if self.allocation_pool.len() > *allocation_pool_min_size {
                            self.allocation_pool.swap_remove(idx2);
                        }
                    } else if e1_free {
                        py_array = e1.clone_ref(py).into_bound(py);
                    } else if e2_free {
                        py_array = e2.clone_ref(py).into_bound(py);
                    } else {
                        let arr: Bound<'_, PyArray<T, _>> =
                            unsafe { PyArrayDyn::new(py, &shape[..], false) };
                        if allocation_pool_max_size.is_none()
                            || self.allocation_pool.len() < allocation_pool_max_size.unwrap()
                        {
                            self.allocation_pool.push(arr.clone().unbind());
                        }
                        py_array = arr;
                    }
                    unsafe { py_array.as_slice_mut().unwrap().copy_from_slice(&array_vec) };
                } else {
                    py_array = ArrayD::from_shape_vec(&shape[..], array_vec)
                        .map_err(|err| {
                            InvalidStateError::new_err(format!(
                                "Failed create Numpy array of T from shape and Vec<T>: {}",
                                err
                            ))
                        })?
                        .into_pyarray(py);
                }
                py_array
            }
        };

        Ok((py_array, offset))
    }
}

macro_rules! create_numpy_pyany_serde {
    ($ty: ty, $config: ident) => {{
        let mut allocation_pool = Vec::new();
        if let NumpySerdeConfig::STATIC {
            ref shape,
            allocation_pool_min_size,
            ..
        } = $config
        {
            if allocation_pool_min_size > 0 {
                Python::with_gil(|py| {
                    for _ in 0..allocation_pool_min_size {
                        let arr: Bound<'_, PyArray<$ty, _>> =
                            unsafe { PyArrayDyn::new(py, &shape[..], false) };
                        allocation_pool.push(arr.unbind());
                    }
                });
            }
        };

        Box::new(NumpySerde::<$ty> {
            config: $config,
            allocation_pool,
        })
    }};
}

pub fn get_numpy_serde(dtype: NumpyDtype, config: NumpySerdeConfig) -> Box<dyn PyAnySerde> {
    match dtype {
        NumpyDtype::INT8 => {
            create_numpy_pyany_serde!(i8, config)
        }
        NumpyDtype::INT16 => {
            create_numpy_pyany_serde!(i16, config)
        }
        NumpyDtype::INT32 => {
            create_numpy_pyany_serde!(i32, config)
        }
        NumpyDtype::INT64 => {
            create_numpy_pyany_serde!(i64, config)
        }
        NumpyDtype::UINT8 => {
            create_numpy_pyany_serde!(u8, config)
        }
        NumpyDtype::UINT16 => {
            create_numpy_pyany_serde!(u16, config)
        }
        NumpyDtype::UINT32 => {
            create_numpy_pyany_serde!(u32, config)
        }
        NumpyDtype::UINT64 => {
            create_numpy_pyany_serde!(u64, config)
        }
        NumpyDtype::FLOAT32 => {
            create_numpy_pyany_serde!(f32, config)
        }
        NumpyDtype::FLOAT64 => {
            create_numpy_pyany_serde!(f64, config)
        }
    }
}

impl<T: Element + AnyBitPattern + NoUninit> PyAnySerde for NumpySerde<T> {
    fn append<'py>(
        &mut self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        let preprocessor_fn_option = match &self.config {
            NumpySerdeConfig::DYNAMIC {
                preprocessor_fn, ..
            } => preprocessor_fn,
            NumpySerdeConfig::STATIC {
                preprocessor_fn, ..
            } => preprocessor_fn,
        };
        match preprocessor_fn_option {
            Some(preprocessor_fn) => self.append_inner(
                buf,
                offset,
                preprocessor_fn
                    .bind(obj.py())
                    .call1((obj,))?
                    .downcast::<PyArrayDyn<T>>()?,
            ),
            None => self.append_inner(buf, offset, obj.downcast::<PyArrayDyn<T>>()?),
        }
    }

    fn retrieve<'py>(
        &mut self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let (array, offset) = self.retrieve_inner(py, buf, offset)?;

        let postprocessor_fn_option = match &self.config {
            NumpySerdeConfig::DYNAMIC {
                postprocessor_fn, ..
            } => postprocessor_fn,
            NumpySerdeConfig::STATIC {
                postprocessor_fn, ..
            } => postprocessor_fn,
        };

        Ok(match postprocessor_fn_option {
            Some(postprocessor_fn) => (postprocessor_fn.bind(py).call1((array,))?, offset),
            None => (array.into_any(), offset),
        })
    }
}
