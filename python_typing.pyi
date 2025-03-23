from __future__ import annotations

from abc import abstractmethod
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
    def append(self, buf: bytes, offset: int, obj: T) -> int:
        """
        Appends bytes of obj to buf starting at offset.
        :param buf: a memoryview to write into (DO NOT hold a reference to this memory view after this function ends!)
        :param offset: an offset into the memory view to start writing
        :param obj: the obj to write as bytes
        :return: new offset after appending bytes
        """
        raise NotImplementedError

    @abstractmethod
    def get_bytes(self, start_addr: Optional[int], obj: T) -> bytes:
        """
        :param start_addr: the starting address for where the returned bytes will be written. May be None in contexts where there is no guaranteed start address.
        :param obj: the obj to write as bytes
        :return: bytes for obj
        """
        raise NotImplementedError

    @abstractmethod
    def retrieve(self, buf: bytes, offset: int) -> Tuple[T, int]:
        """
        Retrieves obj encoded using self.append or self.get_bytes from the buffer starting at offset.
        :param buf: a memoryview to read from (DO NOT hold a reference to this memory view after this function ends!)
        :param offset: an offset into the memory view to start reading
        :return: Tuple of obj and the offset into the memory view after retrieving obj
        """
        raise NotImplementedError

class PickleableInitStrategy(Generic[T]):
    def __new__(cls, init_strategy: InitStrategy[T]) -> PickleableInitStrategy[T]: ...

class InitStrategy(Generic[T]):
    ALL = InitStrategy_ALL
    SOME = InitStrategy_SOME
    NONE = InitStrategy_NONE

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

class PickleableNumpySerdeConfig(Generic[T]):
    def __new__(cls, config: NumpySerdeConfig[T]) -> PickleableNumpySerdeConfig[T]: ...

class NumpySerdeConfig(Generic[T]):
    DYNAMIC = NumpySerdeConfig_DYNAMIC
    STATIC = NumpySerdeConfig_STATIC

class NumpySerdeConfig_DYNAMIC(NumpySerdeConfig[T]):
    def __new__(
        cls,
        preprocessor_fn: Optional[Callable[[T, int], ndarray]] = None,
        postprocessor_fn: Optional[Callable[[ndarray, int], T]] = None,
    ) -> NumpySerdeConfig_DYNAMIC: ...

class NumpySerdeConfig_STATIC(InitStrategy[T]):
    def __new__(
        cls,
        shape: Tuple[int],
        preprocessor_fn: Optional[Callable[[T], ndarray]] = None,
        postprocessor_fn: Optional[Callable[[ndarray], T]] = None,
        allocation_pool_min_size: int = 0,
        allocation_pool_max_size: Optional[int] = None,
    ) -> NumpySerdeConfig_STATIC: ...

class PickleablePyAnySerdeType(Generic[T]):
    def __new__(
        cls, pyany_serde_type: PyAnySerdeType[T]
    ) -> PickleablePyAnySerdeType[T]: ...

class PyAnySerdeType(Generic[T]):
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

    def as_pickleable(self): ...

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
        cls,
        dtype: DTypeLike,
        config: Optional[NumpySerdeConfig[T]] = NumpySerdeConfig_DYNAMIC[T],
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
        key_serde_type_dict: Dict[str, PyAnySerdeType],
    ) -> PyAnySerdeType_TYPEDDICT: ...

class PyAnySerdeType_UNION(PyAnySerdeType[Union]):
    def __new__(
        option_serde_types: List[PyAnySerdeType], option_choice_fn: Callable[[Any], int]
    ) -> PyAnySerdeType_UNION: ...
