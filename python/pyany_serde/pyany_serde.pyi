from typing import (
    Any,
    Callable,
    Dict,
    Generic,
    List,
    Optional,
    Set,
    Tuple,
    TypeVar,
    Union,
    _TypedDict,
)

try:
    from typing import TypeVarTuple, Unpack
except ImportError:
    from typing_extensions import TypeVarTuple, Unpack

from numpy import dtype, ndarray

from .python_serde import PythonSerde

PythonType = TypeVar("PythonType")
KeyPythonType = TypeVar("KeyPythonType")
ValuePythonType = TypeVar("ValuePythonType")
ItemsPythonType = TypeVar("ItemsPythonType")
PythonTypes = TypeVarTuple("PythonTypes")

class DynPyAnySerde(Generic[PythonType]): ...

class DynPyAnySerdeFactory:
    @staticmethod
    def bool_serde() -> DynPyAnySerde[bool]: ...
    @staticmethod
    def bytes_serde() -> DynPyAnySerde[bytes]: ...
    @staticmethod
    def complex_serde() -> DynPyAnySerde[complex]: ...
    @staticmethod
    def dict_serde(
        key_serde_option: Optional[DynPyAnySerde[KeyPythonType]],
        value_serde_option: Optional[DynPyAnySerde[ValuePythonType]],
    ) -> DynPyAnySerde[Dict[KeyPythonType, ValuePythonType]]: ...
    @staticmethod
    def dynamic_serde() -> DynPyAnySerde[Any]: ...
    @staticmethod
    def float_serde() -> DynPyAnySerde[float]: ...
    @staticmethod
    def int_serde() -> DynPyAnySerde[int]: ...
    @staticmethod
    def list_serde(
        items_serde_option: Optional[DynPyAnySerde[ItemsPythonType]],
    ) -> DynPyAnySerde[List[ItemsPythonType]]: ...
    @staticmethod
    def numpy_dynamic_shape_serde(py_dtype: dtype) -> DynPyAnySerde[ndarray]: ...
    @staticmethod
    def option_serde(
        value_serde_option: Optional[DynPyAnySerde[PythonType]],
    ) -> DynPyAnySerde[Optional[PythonType]]: ...
    @staticmethod
    def pickle_serde() -> DynPyAnySerde[Any]: ...
    @staticmethod
    def python_serde_serde(
        python_serde: PythonSerde[PythonType],
    ) -> DynPyAnySerde[PythonType]: ...
    @staticmethod
    def set_serde(
        items_serde_option: Optional[DynPyAnySerde[ItemsPythonType]],
    ) -> DynPyAnySerde[Set[ItemsPythonType]]: ...
    @staticmethod
    def string_serde() -> DynPyAnySerde[str]: ...
    @staticmethod
    def tuple_serde(
        item_serdes: List[Optional[DynPyAnySerde]],
    ) -> DynPyAnySerde[Tuple[Unpack[PythonTypes]]]: ...
    @staticmethod
    def typed_dict_serde(
        serde_dict: Dict[str, Optional[DynPyAnySerde]]
    ) -> DynPyAnySerde[_TypedDict]: ...
    @staticmethod
    def union_serde(
        serde_options: List[Optional[DynPyAnySerde]],
        serde_choice_fn: Callable[[Union[Unpack[PythonTypes]]], int],
    ) -> DynPyAnySerde[Union[Unpack[PythonTypes]]]: ...
