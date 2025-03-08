use bytemuck::{cast_slice, AnyBitPattern, NoUninit};
use numpy::PyArray;
use numpy::{Element, PyArrayDyn, PyArrayMethods};
use pyo3::prelude::*;

use crate::{
    common::{get_bytes_to_alignment, NumpyDtype},
    communication::{append_bytes, retrieve_bytes},
    PyAnySerde,
};

#[derive(Clone)]
pub struct NumpyStaticShapeSerde<T: Element> {
    pub shape: Vec<usize>,
    pub allocation_pool: Vec<Py<PyArrayDyn<T>>>,
}

impl<T: Element + AnyBitPattern + NoUninit> NumpyStaticShapeSerde<T> {
    pub fn append_inner<'py>(
        &mut self,
        buf: &mut [u8],
        mut offset: usize,
        array: &Bound<'py, PyArrayDyn<T>>,
    ) -> PyResult<usize> {
        let obj_vec = array.to_vec()?;
        offset = offset + get_bytes_to_alignment::<T>(buf.as_ptr() as usize + offset);
        offset = append_bytes(buf, offset, cast_slice::<T, u8>(&obj_vec))?;
        Ok(offset)
    }

    pub fn retrieve_inner<'py>(
        &mut self,
        py: Python<'py>,
        buf: &[u8],
        mut offset: usize,
    ) -> PyResult<(Bound<'py, PyArrayDyn<T>>, usize)> {
        offset = offset + get_bytes_to_alignment::<T>(buf.as_ptr() as usize + offset);
        let obj_bytes;
        (obj_bytes, offset) = retrieve_bytes(buf, offset)?;
        let array_vec = cast_slice::<u8, T>(obj_bytes).to_vec();

        let py_array;
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
            if self.allocation_pool.len() > 50 {
                self.allocation_pool.swap_remove(idx2);
            }
        } else if e1_free {
            py_array = e1.clone_ref(py).into_bound(py);
        } else if e2_free {
            py_array = e2.clone_ref(py).into_bound(py);
        } else {
            let arr: Bound<'_, PyArray<T, _>> =
                unsafe { PyArrayDyn::new(py, &self.shape[..], false) };
            self.allocation_pool.push(arr.clone().unbind());
            py_array = arr;
        }
        unsafe { py_array.as_slice_mut().unwrap().copy_from_slice(&array_vec) };

        Ok((py_array, offset))
    }
}

macro_rules! create_numpy_pyany_serde {
    ($ty: ty, $shape: ident) => {{
        let mut allocation_pool = Vec::new();
        Python::with_gil(|py| {
            for _ in 0..50 {
                let arr: Bound<'_, PyArray<$ty, _>> =
                    unsafe { PyArrayDyn::new(py, &$shape[..], false) };
                allocation_pool.push(arr.unbind());
            }
        });
        Box::new(NumpyStaticShapeSerde::<$ty> {
            shape: $shape,
            allocation_pool,
        })
    }};
}

pub fn get_numpy_static_shape_serde(dtype: NumpyDtype, shape: Vec<usize>) -> Box<dyn PyAnySerde> {
    match dtype {
        NumpyDtype::INT8 => {
            create_numpy_pyany_serde!(i8, shape)
        }
        NumpyDtype::INT16 => {
            create_numpy_pyany_serde!(i16, shape)
        }
        NumpyDtype::INT32 => {
            create_numpy_pyany_serde!(i32, shape)
        }
        NumpyDtype::INT64 => {
            create_numpy_pyany_serde!(i64, shape)
        }
        NumpyDtype::UINT8 => {
            create_numpy_pyany_serde!(u8, shape)
        }
        NumpyDtype::UINT16 => {
            create_numpy_pyany_serde!(u16, shape)
        }
        NumpyDtype::UINT32 => {
            create_numpy_pyany_serde!(u32, shape)
        }
        NumpyDtype::UINT64 => {
            create_numpy_pyany_serde!(u64, shape)
        }
        NumpyDtype::FLOAT32 => {
            create_numpy_pyany_serde!(f32, shape)
        }
        NumpyDtype::FLOAT64 => {
            create_numpy_pyany_serde!(f64, shape)
        }
    }
}

impl<T: Element + AnyBitPattern + NoUninit> PyAnySerde for NumpyStaticShapeSerde<T> {
    fn append<'py>(
        &mut self,
        buf: &mut [u8],
        offset: usize,
        obj: &Bound<'py, PyAny>,
    ) -> PyResult<usize> {
        self.append_inner(buf, offset, obj.downcast::<PyArrayDyn<T>>()?)
    }

    fn retrieve<'py>(
        &mut self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyAny>, usize)> {
        let (array, offset) = self.retrieve_inner(py, buf, offset)?;
        Ok((array.into_any(), offset))
    }
}
