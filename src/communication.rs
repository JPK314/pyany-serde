use std::mem::size_of;
use std::os::raw::c_double;

use pyo3::exceptions::asyncio::InvalidStateError;
use pyo3::prelude::*;

use paste::paste;

use crate::PyAnySerde;

macro_rules! define_primitive_communication {
    ($type:ty) => {
        paste! {
            pub fn [<append_ $type>](buf: &mut [u8], offset: usize, val: $type) -> usize {
                let end = offset + size_of::<$type>();
                buf[offset..end].copy_from_slice(&val.to_ne_bytes());
                end
            }

            pub fn [<retrieve_ $type>](buf: &[u8], offset: usize) -> PyResult<($type, usize)> {
                let end = offset + size_of::<$type>();
                Ok(($type::from_ne_bytes(buf[offset..end].try_into()?), end))
            }
        }
    };
}

define_primitive_communication!(usize);
define_primitive_communication!(c_double);
define_primitive_communication!(i64);
define_primitive_communication!(u64);
define_primitive_communication!(f32);
define_primitive_communication!(f64);

pub fn append_bool(buf: &mut [u8], offset: usize, val: bool) -> usize {
    let end = offset + size_of::<u8>();
    buf[offset..end].copy_from_slice(&(val as u8).to_ne_bytes());
    end
}

pub fn append_bool_vec(v: &mut Vec<u8>, val: bool) {
    v.extend_from_slice(&(val as u8).to_ne_bytes());
}

pub fn retrieve_bool(buf: &[u8], offset: usize) -> PyResult<(bool, usize)> {
    let end = offset + size_of::<bool>();
    let val = match u8::from_ne_bytes(buf[offset..end].try_into()?) {
        0 => Ok(false),
        1 => Ok(true),
        v => Err(InvalidStateError::new_err(format!(
            "tried to retrieve bool from shared_memory but got value {}",
            v
        ))),
    }?;
    Ok((val, end))
}

pub fn append_usize_vec(v: &mut Vec<u8>, u: usize) {
    v.extend_from_slice(&u.to_ne_bytes());
}

pub fn append_bytes_vec(v: &mut Vec<u8>, bytes: &[u8]) {
    append_usize_vec(v, bytes.len());
    v.extend_from_slice(bytes);
}

pub fn append_string_vec(v: &mut Vec<u8>, s: &String) {
    append_bytes_vec(v, s.as_bytes());
}

pub fn retrieve_string(buf: &[u8], offset: usize) -> PyResult<(String, usize)> {
    let (string_bytes, offset) = retrieve_bytes(buf, offset)?;
    Ok((String::from_utf8(string_bytes.to_vec())?, offset))
}

pub fn insert_bytes(buf: &mut [u8], offset: usize, bytes: &[u8]) -> PyResult<usize> {
    let end = offset + bytes.len();
    buf[offset..end].copy_from_slice(bytes);
    Ok(end)
}

pub fn append_bytes(buf: &mut [u8], offset: usize, bytes: &[u8]) -> PyResult<usize> {
    let bytes_len = bytes.len();
    let start = append_usize(buf, offset, bytes_len);
    let end = start + bytes.len();
    buf[start..end].copy_from_slice(bytes);
    Ok(end)
}

pub fn retrieve_bytes(buf: &[u8], offset: usize) -> PyResult<(&[u8], usize)> {
    let (len, start) = retrieve_usize(buf, offset)?;
    let end = start + len;
    Ok((&buf[start..end], end))
}

pub fn append_python_option_bound<'py, F>(
    buf: &mut [u8],
    mut offset: usize,
    obj_option: &Option<&Bound<'py, PyAny>>,
    serde_option: &mut Option<&mut Box<dyn PyAnySerde>>,
    err: F,
) -> PyResult<usize>
where
    F: FnOnce() -> PyErr,
{
    if let Some(obj) = obj_option {
        offset = append_bool(buf, offset, true);
        offset = serde_option
            .as_deref_mut()
            .ok_or_else(err)?
            .append(buf, offset, obj)?;
    } else {
        offset = append_bool(buf, offset, false);
    }
    Ok(offset)
}

pub fn append_python_option<'py, F>(
    py: Python<'py>,
    buf: &mut [u8],
    mut offset: usize,
    obj_option: &Option<&PyObject>,
    serde_option: &mut Option<&mut Box<dyn PyAnySerde>>,
    err: F,
) -> PyResult<usize>
where
    F: FnOnce() -> PyErr,
{
    if let Some(obj) = obj_option {
        offset = append_bool(buf, offset, true);
        offset = serde_option
            .as_deref_mut()
            .ok_or_else(err)?
            .append(buf, offset, obj.bind(py))?;
    } else {
        offset = append_bool(buf, offset, false);
    }
    Ok(offset)
}

pub fn retrieve_python_option<'py, F>(
    py: Python<'py>,
    buf: &mut [u8],
    mut offset: usize,
    serde_option: &mut Option<&mut Box<dyn PyAnySerde>>,
    err: F,
) -> PyResult<(Option<Bound<'py, PyAny>>, usize)>
where
    F: FnOnce() -> PyErr,
{
    let is_some;
    (is_some, offset) = retrieve_bool(buf, offset)?;
    let obj_option = if is_some {
        let obj;
        (obj, offset) = serde_option
            .as_deref_mut()
            .ok_or_else(err)?
            .retrieve(py, buf, offset)?;
        Some(obj)
    } else {
        None
    };
    Ok((obj_option, offset))
}
