from typing import Any, Callable, Dict, List, Optional, Type, TypeVar, Union

try:
    from typing import TypeVarTuple, Unpack
except ImportError:
    from typing_extensions import TypeVarTuple, Unpack

import numpy as np
from pyany_serde import DynPyAnySerde, PyAnySerdeFactory

from .python_serde import PythonSerde

T = TypeVar("T")
KeyT = TypeVar("KeyT")
ValueT = TypeVar("ValueT")
ItemsT = TypeVar("ItemsT")
Ts = TypeVarTuple("Ts")


def bool_serde():
    return PyAnySerdeFactory.bool_serde()


def bytes_serde():
    return PyAnySerdeFactory.bytes_serde()


def complex_serde():
    return PyAnySerdeFactory.complex_serde()


def dict_serde(
    key_serde: Optional[DynPyAnySerde[KeyT]],
    value_serde: Optional[DynPyAnySerde[ValueT]],
):
    return PyAnySerdeFactory.dict_serde(key_serde, value_serde)


def dynamic_serde():
    return PyAnySerdeFactory.dynamic_serde()


def float_serde():
    return PyAnySerdeFactory.float_serde()


def int_serde():
    return PyAnySerdeFactory.int_serde()


def list_serde(items_serde: Optional[DynPyAnySerde[ItemsT]]):
    return PyAnySerdeFactory.list_serde(items_serde)


def numpy_serde(dtype: Type[np._DTypeScalar_co]):
    return PyAnySerdeFactory.numpy_dynamic_shape_serde(np.dtype(dtype))


def option_serde(value_serde: Optional[DynPyAnySerde[T]]):
    return PyAnySerdeFactory.option_serde(value_serde)


def pickle_serde():
    return PyAnySerdeFactory.pickle_serde()


def python_serde_serde(python_serde: PythonSerde[T]):
    return PyAnySerdeFactory.python_serde_serde(python_serde)


def set_serde(items_serde: Optional[DynPyAnySerde[ItemsT]]):
    return PyAnySerdeFactory.set_serde(items_serde)


def string_serde():
    return PyAnySerdeFactory.string_serde()


def tuple_serde(*item_serdes: List[Optional[DynPyAnySerde]]):
    return PyAnySerdeFactory.tuple_serde(item_serdes)


def typed_dict_serde(serde_dict: Dict[str, Optional[DynPyAnySerde]]):
    return PyAnySerdeFactory.typed_dict_serde(serde_dict)


def union_serde(
    serde_options: List[Optional[DynPyAnySerde]],
    serde_choice_fn: Callable[[Union[Unpack[Ts]]], int],
):
    return PyAnySerdeFactory.union_serde(serde_options, serde_choice_fn)
