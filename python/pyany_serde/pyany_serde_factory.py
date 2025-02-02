from __future__ import annotations

from typing import Callable, Dict, List, Optional, Type, TypeVar, Union

try:
    from typing import TypeVarTuple, Unpack
except ImportError:
    from typing_extensions import TypeVarTuple, Unpack

import numpy as np

from .pyany_serde import DynPyAnySerde, DynPyAnySerdeFactory
from .python_serde import PythonSerde

T = TypeVar("T")
KeyT = TypeVar("KeyT")
ValueT = TypeVar("ValueT")
ItemsT = TypeVar("ItemsT")
Ts = TypeVarTuple("Ts")


def bool_serde():
    return DynPyAnySerdeFactory.bool_serde()


def bytes_serde():
    return DynPyAnySerdeFactory.bytes_serde()


def complex_serde():
    return DynPyAnySerdeFactory.complex_serde()


def dict_serde(
    key_serde: Optional[DynPyAnySerde[KeyT]],
    value_serde: Optional[DynPyAnySerde[ValueT]],
):
    return DynPyAnySerdeFactory.dict_serde(key_serde, value_serde)


def dynamic_serde():
    return DynPyAnySerdeFactory.dynamic_serde()


def float_serde():
    return DynPyAnySerdeFactory.float_serde()


def int_serde():
    return DynPyAnySerdeFactory.int_serde()


def list_serde(items_serde: Optional[DynPyAnySerde[ItemsT]]):
    return DynPyAnySerdeFactory.list_serde(items_serde)


def numpy_serde(dtype: Type[np._DTypeScalar_co]):
    return DynPyAnySerdeFactory.numpy_dynamic_shape_serde(np.dtype(dtype))


def option_serde(value_serde: Optional[DynPyAnySerde[T]]):
    return DynPyAnySerdeFactory.option_serde(value_serde)


def pickle_serde():
    return DynPyAnySerdeFactory.pickle_serde()


def python_serde_serde(python_serde: PythonSerde[T]):
    return DynPyAnySerdeFactory.python_serde_serde(python_serde)


def set_serde(items_serde: Optional[DynPyAnySerde[ItemsT]]):
    return DynPyAnySerdeFactory.set_serde(items_serde)


def string_serde():
    return DynPyAnySerdeFactory.string_serde()


def tuple_serde(*item_serdes: List[Optional[DynPyAnySerde]]):
    return DynPyAnySerdeFactory.tuple_serde(item_serdes)


def typed_dict_serde(serde_dict: Dict[str, Optional[DynPyAnySerde]]):
    return DynPyAnySerdeFactory.typed_dict_serde(serde_dict)


def union_serde(
    serde_options: List[Optional[DynPyAnySerde]],
    serde_choice_fn: Callable[[Union[Unpack[Ts]]], int],
):
    return DynPyAnySerdeFactory.union_serde(serde_options, serde_choice_fn)
