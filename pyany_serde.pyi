from abc import abstractmethod
from typing import Any, Callable, Dict, Generic, List, Optional, Tuple, TypeVar, Union

from numpy import dtype

T = TypeVar("T")

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

class DynPyAnySerde: ...

class PyAnySerdeFactory:
    @staticmethod
    def bool_serde() -> DynPyAnySerde: ...
    @staticmethod
    def bytes_serde() -> DynPyAnySerde: ...
    @staticmethod
    def complex_serde() -> DynPyAnySerde: ...
    @staticmethod
    def dict_serde(
        key_serde_option: Optional[PythonSerde],
        value_serde_option: Optional[PythonSerde],
    ) -> DynPyAnySerde: ...
    @staticmethod
    def dynamic_serde() -> DynPyAnySerde: ...
    @staticmethod
    def float_serde() -> DynPyAnySerde: ...
    @staticmethod
    def int_serde() -> DynPyAnySerde: ...
    @staticmethod
    def list_serde(
        items_serde_option: Optional[PythonSerde],
    ) -> DynPyAnySerde: ...
    @staticmethod
    def numpy_dynamic_shape_serde(py_dtype: dtype) -> DynPyAnySerde: ...
    @staticmethod
    def option_serde(value_serde_option: Optional[PythonSerde]) -> DynPyAnySerde: ...
    @staticmethod
    def pickle_serde() -> DynPyAnySerde: ...
    @staticmethod
    def python_serde_serde(python_serde: PythonSerde) -> DynPyAnySerde: ...
    @staticmethod
    def set_serde(
        items_serde_option: Optional[PythonSerde],
    ) -> DynPyAnySerde: ...
    @staticmethod
    def string_serde() -> DynPyAnySerde: ...
    @staticmethod
    def tuple_serde(item_serdes: List[Optional[PythonSerde]]) -> DynPyAnySerde: ...
    @staticmethod
    def typed_dict_serde(
        serde_kv_list: Union[
            List[Tuple[str, Optional[PythonSerde]]], Dict[str, Optional[PythonSerde]]
        ]
    ) -> DynPyAnySerde: ...
    @staticmethod
    def union_serde(
        serde_options: List[Optional[PythonSerde]],
        serde_choice_fn: Callable[[Any], int],
    ): ...
