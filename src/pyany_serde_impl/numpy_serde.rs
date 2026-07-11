use std::env;

use bytemuck::{AnyBitPattern, NoUninit, cast_slice};
use enum_kinds::EnumKind;
use numpy::ndarray::ArrayD;
use numpy::{Element, PyArrayDyn, PyArrayMethods, PyUntypedArrayMethods};
use numpy::{IntoPyArray, PyArray};
use pyo3::exceptions::PyValueError;
use pyo3::exceptions::asyncio::InvalidStateError;
use pyo3::sync::PyOnceLock;
use pyo3::types::{PyDict, PyList, PyType};
use pyo3::{PyTypeInfo, intern, prelude::*};
use strum_macros::{Display, EnumIter};

use crate::communication::{append_bytes_vec, append_usize, append_usize_vec, retrieve_usize};
use crate::{
    PyAnySerde,
    common::{NumpyDtype, get_bytes_to_alignment},
    communication::{append_bytes, retrieve_bytes},
};

// TODO: remove preprocessor and postprocessor fns
#[pyclass(from_py_object)]
#[derive(Debug, Clone, Display, EnumKind)]
#[enum_kind(NumpySerdeConfigKind, derive(Display, EnumIter))]
pub enum NumpySerdeConfig {
    #[pyo3(constructor = (preprocessor_fn = None, postprocessor_fn = None))]
    DYNAMIC {
        preprocessor_fn: Option<Py<PyAny>>,
        postprocessor_fn: Option<Py<PyAny>>,
    },
    #[pyo3(constructor = (shape, preprocessor_fn = None, postprocessor_fn = None, allocation_pool_min_size = 0, allocation_pool_max_size = None, allocation_pool_warning_size = Some(10000)))]
    STATIC {
        shape: Vec<usize>,
        preprocessor_fn: Option<Py<PyAny>>,
        postprocessor_fn: Option<Py<PyAny>>,
        allocation_pool_min_size: usize,
        allocation_pool_max_size: Option<usize>,
        allocation_pool_warning_size: Option<usize>,
    },
}

impl NumpySerdeConfigKind {
    pub fn type_object<'py>(self, py: Python<'py>) -> Bound<'py, PyType> {
        match self {
            NumpySerdeConfigKind::DYNAMIC => NumpySerdeConfig_DYNAMIC::type_object(py),
            NumpySerdeConfigKind::STATIC => NumpySerdeConfig_STATIC::type_object(py),
        }
    }
    pub fn from_type_object<'py>(
        to: &Bound<'py, PyType>,
    ) -> PyResult<Option<NumpySerdeConfigKind>> {
        let py = to.py();
        if to.eq(NumpySerdeConfig::type_object(py))? {
            return Ok(None);
        }
        if to.eq(NumpySerdeConfig_DYNAMIC::type_object(py))? {
            return Ok(Some(NumpySerdeConfigKind::DYNAMIC));
        }
        if to.eq(NumpySerdeConfig_STATIC::type_object(py))? {
            return Ok(Some(NumpySerdeConfigKind::STATIC));
        }
        Err(PyValueError::new_err(format!(
            "Unexpected value PyType {}",
            to.repr()?
        )))
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
                for &dim in shape.iter() {
                    offset = append_usize(buf, offset, dim);
                }
                let obj_vec = array.to_vec()?;
                offset = offset + get_bytes_to_alignment::<T>(buf.as_ptr() as usize + offset);
                offset = append_bytes(buf, offset, cast_slice::<T, u8>(&obj_vec));
            }
            NumpySerdeConfig::STATIC { .. } => {
                let obj_vec = array.to_vec()?;
                offset = offset + get_bytes_to_alignment::<T>(buf.as_ptr() as usize + offset);
                offset = append_bytes(buf, offset, cast_slice::<T, u8>(&obj_vec));
            }
        }
        Ok(offset)
    }

    fn append_inner_vec<'py>(
        &mut self,
        v: &mut Vec<u8>,
        start_addr: Option<usize>,
        array: &Bound<'py, PyArrayDyn<T>>,
    ) -> PyResult<()> {
        let Some(start_addr) = start_addr else {
            Err(InvalidStateError::new_err(
                "Tried to serialize numpy data, but there was no start_addr provided so there's no way to know how to align the data. (was this called from inside a preprocessor function?)",
            ))?
        };
        match &self.config {
            NumpySerdeConfig::DYNAMIC { .. } => {
                let shape = array.shape();
                append_usize_vec(v, shape.len());
                for &dim in shape.iter() {
                    append_usize_vec(v, dim);
                }
                let obj_vec = array.to_vec()?;
                v.append(&mut vec![
                    0;
                    get_bytes_to_alignment::<T>(start_addr + v.len())
                ]);
                append_bytes_vec(v, cast_slice::<T, u8>(&obj_vec));
            }
            NumpySerdeConfig::STATIC { .. } => {
                let obj_vec = array.to_vec()?;
                v.append(&mut vec![
                    0;
                    get_bytes_to_alignment::<T>(start_addr + v.len())
                ]);
                append_bytes_vec(v, cast_slice::<T, u8>(&obj_vec));
            }
        }
        Ok(())
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
                            "Failed create Numpy array of T from shape and Vec<T>: {err}"
                        ))
                    })?
                    .into_pyarray(py)
            }
            NumpySerdeConfig::STATIC {
                shape,
                allocation_pool_min_size,
                allocation_pool_max_size,
                allocation_pool_warning_size,
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
                    let e1_free = unsafe { pyo3::ffi::Py_REFCNT(e1.as_ptr()) } == 1;
                    let e2_free = unsafe { pyo3::ffi::Py_REFCNT(e2.as_ptr()) } == 1;
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
                        if let Some(allocation_pool_warning_size) = allocation_pool_warning_size
                            && pool_size > *allocation_pool_warning_size
                            && pool_size.is_multiple_of(100)
                        {
                            let recursion_depth =
                                env::var("PYANY_SERDE_NUMPY_ALLOCATION_WARNING_RECUSION_DEPTH")
                                    .map(|v| v.parse::<usize>().unwrap_or(5))
                                    .unwrap_or(5);
                            println!(
                                "Warning: the allocation pool for this Numpy PyAny serde instance is currently {pool_size}, which is larger than the warning limit set ({allocation_pool_warning_size}). Here is a random element from the allocation pool and a dict of the types of its referrers (and the referrers of those referrers, etc, up to the recursion depth set by PYANY_SERDE_NUMPY_ALLOCATION_WARNING_RECUSION_DEPTH (5 by default)):"
                            );
                            let mut total_in_use = 0;
                            for item in self.allocation_pool.iter() {
                                if unsafe { pyo3::ffi::Py_REFCNT(item.as_ptr()) } > 1 {
                                    total_in_use += 1;
                                }
                            }
                            println!(
                                "Number of elements in allocation pool which are currently in use: {total_in_use}"
                            );
                            let idx = fastrand::usize(..pool_size);
                            let e = &self.allocation_pool[idx];
                            println!(
                                "{}\n\n",
                                get_ref_types(e.bind(py), recursion_depth)?.repr()?
                            );
                        }
                    }
                    unsafe { py_array.as_slice_mut().unwrap().copy_from_slice(&array_vec) };
                } else {
                    py_array = ArrayD::from_shape_vec(&shape[..], array_vec)
                        .map_err(|err| {
                            InvalidStateError::new_err(format!(
                                "Failed create Numpy array of T from shape and Vec<T>: {err}"
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

#[macro_export]
macro_rules! create_numpy_pyany_serde {
    ($ty: ty, $config: expr) => {{
        let mut allocation_pool = Vec::new();
        let new_config;
        if let NumpySerdeConfig::STATIC {
            shape,
            preprocessor_fn,
            postprocessor_fn,
            allocation_pool_min_size,
            allocation_pool_max_size,
            allocation_pool_warning_size,
        } = $config
        {
            let allocation_pool_min_size = allocation_pool_min_size.max(2);
            if allocation_pool_max_size.map(|v| v > 0).unwrap_or(true) {
                let starting_pool_size = allocation_pool_min_size
                    .min(allocation_pool_max_size.unwrap_or(allocation_pool_min_size));
                Python::attach(|py| {
                    for _ in 0..starting_pool_size {
                        let arr: Bound<'_, numpy::PyArray<$ty, _>> =
                            unsafe { numpy::PyArrayDyn::new(py, &shape[..], false) };
                        allocation_pool.push(arr.unbind());
                    }
                });
            }
            new_config = NumpySerdeConfig::STATIC {
                shape,
                preprocessor_fn,
                postprocessor_fn,
                allocation_pool_min_size,
                allocation_pool_max_size,
                allocation_pool_warning_size,
            };
        } else {
            new_config = $config;
        }

        Box::new(NumpySerde::<$ty> {
            config: new_config,
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
                    .cast::<PyArrayDyn<T>>()?,
            ),
            None => self.append_inner(buf, offset, obj.cast::<PyArrayDyn<T>>()?),
        }
    }

    fn append_vec<'py>(
        &mut self,
        v: &mut Vec<u8>,
        start_addr: Option<usize>,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<()> {
        let preprocessor_fn_option = match &self.config {
            NumpySerdeConfig::DYNAMIC {
                preprocessor_fn, ..
            } => preprocessor_fn,
            NumpySerdeConfig::STATIC {
                preprocessor_fn, ..
            } => preprocessor_fn,
        };
        match preprocessor_fn_option {
            Some(preprocessor_fn) => self.append_inner_vec(
                v,
                start_addr,
                preprocessor_fn
                    .bind(obj.py())
                    .call1((obj,))?
                    .cast::<PyArrayDyn<T>>()?,
            ),
            None => self.append_inner_vec(v, start_addr, obj.cast::<PyArrayDyn<T>>()?),
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
            Some(postprocessor_fn) => (postprocessor_fn.bind(py).call1((array, offset))?, offset),
            None => (array.into_any(), offset),
        })
    }
}

static GC: PyOnceLock<Py<PyModule>> = PyOnceLock::new();
fn get_ref_types<'py>(o: &Bound<'py, PyAny>, recursion: usize) -> PyResult<Bound<'py, PyAny>> {
    let py = o.py();
    let gc = GC
        .get_or_try_init(py, || Ok::<_, PyErr>(py.import("gc")?.unbind()))?
        .bind(o.py());
    let referrers = gc
        .call_method1(intern!(py, "get_referrers"), (o,))?
        .cast_into::<PyList>()?;
    if recursion > 0 {
        Ok(PyDict::from_sequence(
            &referrers
                .iter()
                .map(|referrer| {
                    Ok::<_, PyErr>((
                        referrer.get_type().repr()?.to_string(),
                        get_ref_types(&referrer, recursion - 1)?,
                    ))
                })
                .collect::<PyResult<Vec<_>>>()?
                .into_pyobject(py)?,
        )?
        .into_any())
    } else {
        referrers
            .iter()
            .map(|referrer| Ok::<_, PyErr>(referrer.get_type().repr()?.to_string()))
            .collect::<PyResult<Vec<_>>>()?
            .into_pyobject(py)
    }
}
