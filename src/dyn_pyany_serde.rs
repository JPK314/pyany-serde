use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::sync::GILOnceCell;
use pyo3::types::{PyCapsule, PyString, PyType};

use crate::pyany_serde::PyAnySerde;
use crate::pyany_serde_type::retrieve_pyany_serde_type;

// Recursive expansion of pyclass macro with modifications to PyTypeInfo trait
// =====================================

#[derive(Clone)]
#[repr(C)]
pub struct DynPyAnySerde(pub Option<Box<dyn PyAnySerde>>);

struct DynPyAnySerdeCapsule(GILOnceCell<Py<PyCapsule>>);

unsafe impl Send for DynPyAnySerdeCapsule {}

unsafe impl Sync for DynPyAnySerdeCapsule {}

impl ::pyo3::types::DerefToPyAny for DynPyAnySerde {}

unsafe impl ::pyo3::type_object::PyTypeInfo for DynPyAnySerde {
    const NAME: &'static str = "DynPyAnySerde";
    const MODULE: ::std::option::Option<&'static str> = ::core::option::Option::Some("pyany_serde");
    #[inline]
    fn type_object_raw(py: ::pyo3::Python<'_>) -> *mut ::pyo3::ffi::PyTypeObject {
        static CAPSULE: DynPyAnySerdeCapsule = DynPyAnySerdeCapsule(GILOnceCell::new());

        let capsule = CAPSULE
            .0
            .get_or_try_init::<_, PyErr>(py, || {
                let py_module = py.import(PyString::new(py, "pyany_serde")).unwrap();
                let py_ty = py_module.getattr("DynPyAnySerde")?;
                let binding = py_ty
                    .getattr("__get_lazy_type_object__")
                    .map_err(|err| PyRuntimeError::new_err(format!("{err}")))?
                    .call0()?;
                Ok(binding.downcast_into::<PyCapsule>()?.unbind())
            })
            .unwrap()
            .bind(py);
        let orig_py_type_ref = unsafe { capsule.reference::<Py<PyType>>() };
        orig_py_type_ref.bind(py).as_type_ptr()
    }
}
impl ::pyo3::PyClass for DynPyAnySerde {
    type Frozen = ::pyo3::pyclass::boolean_struct::False;
}
impl<'a, 'py> ::pyo3::impl_::extract_argument::PyFunctionArgument<'a, 'py> for &'a DynPyAnySerde {
    type Holder = ::std::option::Option<::pyo3::PyRef<'py, DynPyAnySerde>>;
    #[inline]
    fn extract(
        obj: &'a ::pyo3::Bound<'py, ::pyo3::PyAny>,
        holder: &'a mut Self::Holder,
    ) -> ::pyo3::PyResult<Self> {
        ::pyo3::impl_::extract_argument::extract_pyclass_ref(obj, holder)
    }
}
impl<'a, 'py> ::pyo3::impl_::extract_argument::PyFunctionArgument<'a, 'py>
    for &'a mut DynPyAnySerde
{
    type Holder = ::std::option::Option<::pyo3::PyRefMut<'py, DynPyAnySerde>>;
    #[inline]
    fn extract(
        obj: &'a ::pyo3::Bound<'py, ::pyo3::PyAny>,
        holder: &'a mut Self::Holder,
    ) -> ::pyo3::PyResult<Self> {
        ::pyo3::impl_::extract_argument::extract_pyclass_ref_mut(obj, holder)
    }
}
#[allow(deprecated)]
impl ::pyo3::IntoPy<::pyo3::PyObject> for DynPyAnySerde {
    fn into_py(self, py: ::pyo3::Python<'_>) -> ::pyo3::PyObject {
        ::pyo3::IntoPy::into_py(::pyo3::Py::new(py, self).unwrap(), py)
    }
}
impl<'py> ::pyo3::conversion::IntoPyObject<'py> for DynPyAnySerde {
    type Target = Self;
    type Output = ::pyo3::Bound<'py, <Self as ::pyo3::conversion::IntoPyObject<'py>>::Target>;
    type Error = ::pyo3::PyErr;
    fn into_pyobject(
        self,
        py: ::pyo3::Python<'py>,
    ) -> ::std::result::Result<
        <Self as ::pyo3::conversion::IntoPyObject>::Output,
        <Self as ::pyo3::conversion::IntoPyObject>::Error,
    > {
        ::pyo3::Bound::new(py, self)
    }
}
impl ::pyo3::impl_::pyclass::PyClassImpl for DynPyAnySerde {
    const IS_BASETYPE: bool = false;
    const IS_SUBCLASS: bool = false;
    const IS_MAPPING: bool = false;
    const IS_SEQUENCE: bool = false;
    type BaseType = ::pyo3::PyAny;
    type ThreadChecker = ::pyo3::impl_::pyclass::ThreadCheckerImpl;
    type PyClassMutability =  << ::pyo3::PyAny as ::pyo3::impl_::pyclass::PyClassBaseType> ::PyClassMutability as ::pyo3::impl_::pycell::PyClassMutability> ::MutableChild;
    type Dict = ::pyo3::impl_::pyclass::PyClassDummySlot;
    type WeakRef = ::pyo3::impl_::pyclass::PyClassDummySlot;
    type BaseNativeType = ::pyo3::PyAny;
    fn items_iter() -> ::pyo3::impl_::pyclass::PyClassItemsIter {
        use ::pyo3::impl_::pyclass::*;
        let collector = PyClassImplCollector::<Self>::new();
        static INTRINSIC_ITEMS: PyClassItems = PyClassItems {
            methods: &[],
            slots: &[],
        };
        PyClassItemsIter::new(&INTRINSIC_ITEMS, collector.py_methods())
    }
    fn doc(py: ::pyo3::Python<'_>) -> ::pyo3::PyResult<&'static ::std::ffi::CStr> {
        use ::pyo3::impl_::pyclass::*;
        static DOC: ::pyo3::sync::GILOnceCell<::std::borrow::Cow<'static, ::std::ffi::CStr>> =
            ::pyo3::sync::GILOnceCell::new();
        DOC.get_or_try_init(py, || {
            let collector = PyClassImplCollector::<Self>::new();
            build_pyclass_doc(
                <Self as ::pyo3::PyTypeInfo>::NAME,
                c"",
                collector.new_text_signature(),
            )
        })
        .map(::std::ops::Deref::deref)
    }
    fn lazy_type_object() -> &'static ::pyo3::impl_::pyclass::LazyTypeObject<Self> {
        use ::pyo3::impl_::pyclass::LazyTypeObject;
        static TYPE_OBJECT: LazyTypeObject<DynPyAnySerde> = LazyTypeObject::new();
        &TYPE_OBJECT
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
impl DynPyAnySerde {}

impl DynPyAnySerde {
    #[doc(hidden)]
    pub const _PYO3_DEF: ::pyo3::impl_::pymodule::AddClassToModule<Self> =
        ::pyo3::impl_::pymodule::AddClassToModule::new();
}
#[doc(hidden)]
#[allow(non_snake_case)]
impl DynPyAnySerde {}

#[pymethods]
impl DynPyAnySerde {
    #[new]
    fn new() -> Self {
        DynPyAnySerde(None)
    }
    fn __getstate__(&self) -> Vec<u8> {
        self.0.as_ref().unwrap().get_enum_bytes().to_vec()
    }
    fn __setstate__(&mut self, state: Vec<u8>) -> PyResult<()> {
        let (serde_enum, _) = retrieve_pyany_serde_type(&state[..], 0)?;
        self.0 = Some(serde_enum.try_into()?);
        Ok(())
    }
    #[staticmethod]
    fn __get_lazy_type_object__(py: Python) -> PyResult<Bound<pyo3::types::PyCapsule>> {
        let py_type = <Self as pyo3::impl_::pyclass::PyClassImpl>::lazy_type_object()
            .get_or_init(py)
            .clone()
            .unbind();
        pyo3::types::PyCapsule::new(py, py_type, None)
    }
}
