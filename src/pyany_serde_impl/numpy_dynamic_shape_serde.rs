use std::marker::PhantomData;

use bytemuck::{cast_slice, AnyBitPattern, NoUninit};
use numpy::IntoPyArray;
use numpy::{ndarray::ArrayD, Element, PyArrayDyn, PyArrayMethods, PyUntypedArrayMethods};
use pyo3::exceptions::asyncio::InvalidStateError;
use pyo3::prelude::*;

use crate::{
    common::{get_bytes_to_alignment, NumpyDtype},
    communication::{append_bytes, append_usize, retrieve_bytes, retrieve_usize},
    PyAnySerde,
};

#[derive(Clone)]
pub struct NumpyDynamicShapeSerde<T: Element> {
    pub dtype: PhantomData<T>,
}

impl<T: Element + AnyBitPattern + NoUninit> NumpyDynamicShapeSerde<T> {
    pub fn append_inner<'py>(
        &mut self,
        buf: &mut [u8],
        offset: usize,
        array: &Bound<'py, PyArrayDyn<T>>,
    ) -> PyResult<usize> {
        let shape = array.shape();
        let mut offset = append_usize(buf, offset, shape.len());
        for dim in shape.iter() {
            offset = append_usize(buf, offset, *dim);
        }
        let obj_vec = array.to_vec()?;
        offset = offset + get_bytes_to_alignment::<T>(buf.as_ptr() as usize + offset);
        offset = append_bytes(buf, offset, cast_slice::<T, u8>(&obj_vec))?;
        Ok(offset)
    }

    pub fn retrieve_inner<'py>(
        &mut self,
        py: Python<'py>,
        buf: &[u8],
        offset: usize,
    ) -> PyResult<(Bound<'py, PyArrayDyn<T>>, usize)> {
        let (shape_len, mut offset) = retrieve_usize(buf, offset)?;
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
        let array = ArrayD::from_shape_vec(shape, array_vec).map_err(|err| {
            InvalidStateError::new_err(format!(
                "Failed create Numpy array of T from shape and Vec<T>: {}",
                err
            ))
        })?;
        Ok((array.into_pyarray(py), offset))
    }
}

pub fn get_numpy_dynamic_shape_serde(dtype: NumpyDtype) -> Box<dyn PyAnySerde> {
    match dtype {
        NumpyDtype::INT8 => Box::new(NumpyDynamicShapeSerde::<i8> { dtype: PhantomData }),
        NumpyDtype::INT16 => Box::new(NumpyDynamicShapeSerde::<i16> { dtype: PhantomData }),
        NumpyDtype::INT32 => Box::new(NumpyDynamicShapeSerde::<i32> { dtype: PhantomData }),
        NumpyDtype::INT64 => Box::new(NumpyDynamicShapeSerde::<i64> { dtype: PhantomData }),
        NumpyDtype::UINT8 => Box::new(NumpyDynamicShapeSerde::<u8> { dtype: PhantomData }),
        NumpyDtype::UINT16 => Box::new(NumpyDynamicShapeSerde::<u16> { dtype: PhantomData }),
        NumpyDtype::UINT32 => Box::new(NumpyDynamicShapeSerde::<u32> { dtype: PhantomData }),
        NumpyDtype::UINT64 => Box::new(NumpyDynamicShapeSerde::<u64> { dtype: PhantomData }),
        NumpyDtype::FLOAT32 => Box::new(NumpyDynamicShapeSerde::<f32> { dtype: PhantomData }),
        NumpyDtype::FLOAT64 => Box::new(NumpyDynamicShapeSerde::<f64> { dtype: PhantomData }),
    }
}

impl<T: Element + AnyBitPattern + NoUninit> PyAnySerde for NumpyDynamicShapeSerde<T> {
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
