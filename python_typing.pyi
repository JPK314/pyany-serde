from __future__ import annotations

from abc import abstractmethod
from enum import Enum
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

from numpy import DTypeLike, _ShapeType, ndarray

T = TypeVar("T")
KeysT = TypeVar("KeysT")
ValuesT = TypeVar("ValuesT")

class PythonSerde(Generic[T]):
    @abstractmethod
    def to_bytes(self, obj: T) -> bytes:
        """
        Function to convert obj to bytes, for passing between batched agent and the agent manager.
        :return: bytes b such that from_bytes(b) == obj.
        """
        raise NotImplementedError

    @abstractmethod
    def from_bytes(self, byts: bytes) -> T:
        """
        Function to convert bytes to T, for passing between batched agent and the agent manager.
        :return: T obj such that from_bytes(to_bytes(obj)) == obj.
        """
        raise NotImplementedError

class PickleableInitStrategy(Generic[T]):
    def __new__(cls, init_strategy: InitStrategy[T]) -> PickleableInitStrategy[T]: ...

class InitStrategy(Enum, Generic[T]):
    ALL = ...
    SOME = ...
    NONE = ...

class InitStrategy_ALL(InitStrategy[T]):
    def __new__(cls) -> InitStrategy_ALL: ...

class InitStrategy_SOME(InitStrategy[T]):
    def __new__(cls, kwargs: List[str]) -> InitStrategy_ALL:
        """
        kwargs: a list of keyword arguments to pass to the constructor of the dataclass
        """
        ...

class InitStrategy_NONE(InitStrategy[T]):
    def __new__(cls) -> InitStrategy_NONE: ...

class PickleablePyAnySerdeType(Generic[T]):
    def __new__(
        cls, pyany_serde_type: PyAnySerdeType[T]
    ) -> PickleablePyAnySerdeType[T]: ...

class PyAnySerdeType(Enum, Generic[T]):
    BOOL = PyAnySerdeType_BOOL
    BYTES = PyAnySerdeType_BYTES
    COMPLEX = PyAnySerdeType_COMPLEX
    DATACLASS = PyAnySerdeType_DATACLASS
    DICT = PyAnySerdeType_DICT
    DYNAMIC = PyAnySerdeType_DYNAMIC
    FLOAT = PyAnySerdeType_FLOAT
    INT = PyAnySerdeType_INT
    LIST = PyAnySerdeType_LIST
    NUMPY = PyAnySerdeType_NUMPY
    OPTION = PyAnySerdeType_OPTION
    PICKLE = PyAnySerdeType_PICKLE
    PYTHONSERDE = PyAnySerdeType_PYTHONSERDE
    SET = PyAnySerdeType_SET
    STRING = PyAnySerdeType_STRING
    TUPLE = PyAnySerdeType_TUPLE
    TYPEDDICT = PyAnySerdeType_TYPEDDICT
    UNION = PyAnySerdeType_UNION

class PyAnySerdeType_BOOL(PyAnySerdeType[bool]):
    def __new__(cls) -> PyAnySerdeType_BOOL: ...

class PyAnySerdeType_BYTES(PyAnySerdeType[bytes]):
    def __new__(cls) -> PyAnySerdeType_BYTES: ...

class PyAnySerdeType_COMPLEX(PyAnySerdeType[complex]):
    def __new__(cls) -> PyAnySerdeType_COMPLEX: ...

class PyAnySerdeType_DATACLASS(PyAnySerdeType[T]):
    def __new__(
        cls,
        clazz: T,
        init_strategy: InitStrategy,
        field_serde_type_dict: Dict[str, PyAnySerdeType],
    ) -> PyAnySerdeType_DATACLASS[T]:
        """
        clazz: the dataclass to be serialized
        init_strategy: defines the initialization strategy
        field_serde_type_dict: dict to define the serde to be used with each field in the dataclass
        """
        ...

class PyAnySerdeType_DICT(PyAnySerdeType[Dict[KeysT, ValuesT]]):
    def __new__(
        cls,
        keys_serde_type: PyAnySerdeType[KeysT],
        values_serde_type: PyAnySerdeType[ValuesT],
    ) -> PyAnySerdeType_DICT[KeysT, ValuesT]: ...

class PyAnySerdeType_DYNAMIC(PyAnySerdeType[Any]):
    def __new__(cls) -> PyAnySerdeType_DYNAMIC: ...

class PyAnySerdeType_FLOAT(PyAnySerdeType[float]):
    def __new__(cls) -> PyAnySerdeType_FLOAT: ...

class PyAnySerdeType_INT(PyAnySerdeType[int]):
    def __new__(cls) -> PyAnySerdeType_INT: ...

class PyAnySerdeType_LIST(PyAnySerdeType[List[T]]):
    def __new__(cls, items_serde_type: PyAnySerdeType[T]) -> PyAnySerdeType_LIST[T]: ...

class PyAnySerdeType_NUMPY(PyAnySerdeType[ndarray[_ShapeType, DTypeLike]]):
    def __new__(
        cls, dtype: DTypeLike
    ) -> PyAnySerdeType_NUMPY[_ShapeType, DTypeLike]: ...

class PyAnySerdeType_OPTION(PyAnySerdeType[Optional[T]]):
    def __new__(
        cls, value_serde_type: PyAnySerdeType[T]
    ) -> PyAnySerdeType_OPTION[T]: ...

class PyAnySerdeType_PICKLE(PyAnySerdeType[Any]):
    def __new__(cls) -> PyAnySerdeType_PICKLE: ...

class PyAnySerdeType_PYTHONSERDE(PyAnySerdeType[T]):
    def __new__(
        cls, python_serde_type: PythonSerde[T]
    ) -> PyAnySerdeType_PYTHONSERDE[T]: ...

class PyAnySerdeType_SET(PyAnySerdeType[Set[T]]):
    def __new__(cls, items_serde_type: PyAnySerdeType[T]) -> PyAnySerdeType_SET[T]: ...

class PyAnySerdeType_STRING(PyAnySerdeType[str]):
    def __new__(cls) -> PyAnySerdeType_STRING: ...

class PyAnySerdeType_TUPLE(PyAnySerdeType[Tuple]):
    def __new__(item_serde_types: Tuple[PyAnySerdeType]) -> PyAnySerdeType_TUPLE: ...

class PyAnySerdeType_TYPEDDICT(PyAnySerdeType[_TypedDict]):
    def __new__(
        key_serde_type_dict: Dict[str, PyAnySerdeType]
    ) -> PyAnySerdeType_TYPEDDICT: ...

class PyAnySerdeType_UNION(PyAnySerdeType[Union]):
    def __new__(
        option_serde_types: List[PyAnySerdeType], option_choice_fn: Callable[[Any], int]
    ) -> PyAnySerdeType_UNION: ...
