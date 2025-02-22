use num_derive::{FromPrimitive, ToPrimitive};
use numpy::{dtype, PyArrayDescr, PyArrayDescrMethods};
use pyo3::{exceptions::PyValueError, intern, prelude::*};
use strum_macros::{Display, EnumIter, EnumString};

// Why not just use PyArrayDescr? Because PyArrayDescr doesn't allow for derivation of Debug, PartialEq, or Copy.
#[derive(
    Debug, PartialEq, Clone, Copy, FromPrimitive, ToPrimitive, Display, EnumString, EnumIter,
)]
#[strum(serialize_all = "lowercase")]
pub enum NumpyDtype {
    INT8,
    INT16,
    INT32,
    INT64,
    UINT8,
    UINT16,
    UINT32,
    UINT64,
    FLOAT32,
    FLOAT64,
}

impl<'py> IntoPyObject<'py> for NumpyDtype {
    type Target = PyArrayDescr;

    type Output = Bound<'py, PyArrayDescr>;

    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(match self {
            NumpyDtype::INT8 => dtype::<i8>(py),
            NumpyDtype::INT16 => dtype::<i16>(py),
            NumpyDtype::INT32 => dtype::<i32>(py),
            NumpyDtype::INT64 => dtype::<i64>(py),
            NumpyDtype::UINT8 => dtype::<u8>(py),
            NumpyDtype::UINT16 => dtype::<u16>(py),
            NumpyDtype::UINT32 => dtype::<u32>(py),
            NumpyDtype::UINT64 => dtype::<u64>(py),
            NumpyDtype::FLOAT32 => dtype::<f32>(py),
            NumpyDtype::FLOAT64 => dtype::<f64>(py),
        })
    }
}

impl<'py> FromPyObject<'py> for NumpyDtype {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let py = ob.py();
        let dtype_any = py
            .import(intern!(py, "numpy"))?
            .getattr(intern!(py, "dtype"))?
            .call1((ob,))?;
        let dtype = dtype_any.downcast::<PyArrayDescr>()?;
        match dtype.num() {
            1 => Ok(NumpyDtype::INT8),
            2 => Ok(NumpyDtype::UINT8),
            3 => Ok(NumpyDtype::INT16),
            4 => Ok(NumpyDtype::UINT16),
            7 => Ok(NumpyDtype::INT32),
            8 => Ok(NumpyDtype::UINT32),
            9 => Ok(NumpyDtype::INT64),
            10 => Ok(NumpyDtype::UINT64),
            11 => Ok(NumpyDtype::FLOAT32),
            12 => Ok(NumpyDtype::FLOAT64),
            _ => Err(PyValueError::new_err(format!(
                "Invalid dtype: {}",
                dtype.repr()?.to_str()?
            ))),
        }
    }
}

pub fn get_numpy_dtype(py_dtype: Py<PyArrayDescr>) -> PyResult<NumpyDtype> {
    Python::with_gil(|py| {
        let bound_dtype = py_dtype.into_bound(py);
        match bound_dtype.num() {
            1 => Ok(NumpyDtype::INT8),
            2 => Ok(NumpyDtype::UINT8),
            3 => Ok(NumpyDtype::INT16),
            4 => Ok(NumpyDtype::UINT16),
            7 => Ok(NumpyDtype::INT32),
            8 => Ok(NumpyDtype::UINT32),
            9 => Ok(NumpyDtype::INT64),
            10 => Ok(NumpyDtype::UINT64),
            11 => Ok(NumpyDtype::FLOAT32),
            12 => Ok(NumpyDtype::FLOAT64),
            _ => Err(PyValueError::new_err(format!(
                "Invalid dtype: {}",
                bound_dtype.repr()?.to_str()?
            ))),
        }
    })
}
