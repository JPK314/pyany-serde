from typing import Any, Callable, Dict, List, Type

import numpy as np
from pyany_serde import DynPyAnySerde, PyAnySerdeFactory

from .python_serde import PythonSerde


def bool_serde():
    return PyAnySerdeFactory.bool_serde()


def bytes_serde():
    return PyAnySerdeFactory.bytes_serde()


def complex_serde():
    return PyAnySerdeFactory.complex_serde()


def dict_serde(key_serde: DynPyAnySerde, value_serde: DynPyAnySerde):
    return PyAnySerdeFactory.dict_serde(key_serde, value_serde)


def dynamic_serde():
    return PyAnySerdeFactory.dynamic_serde()


def float_serde():
    return PyAnySerdeFactory.float_serde()


def int_serde():
    return PyAnySerdeFactory.int_serde()


def list_serde(items_serde: DynPyAnySerde):
    return PyAnySerdeFactory.list_serde(items_serde)


def numpy_serde(dtype: Type[np._DTypeScalar_co]):
    return PyAnySerdeFactory.numpy_dynamic_shape_serde(np.dtype(dtype))


def option_serde(value_serde: DynPyAnySerde):
    return PyAnySerdeFactory.option_serde(value_serde)


def pickle_serde():
    return PyAnySerdeFactory.pickle_serde()


def python_serde_serde(python_serde: PythonSerde):
    return PyAnySerdeFactory.python_serde_serde(python_serde)


def set_serde(items_serde: DynPyAnySerde):
    return PyAnySerdeFactory.set_serde(items_serde)


def string_serde():
    return PyAnySerdeFactory.string_serde()


def tuple_serde(*item_serdes: List[DynPyAnySerde]):
    return PyAnySerdeFactory.tuple_serde(item_serdes)


def typed_dict_serde(serde_dict: Dict[str, DynPyAnySerde]):
    return PyAnySerdeFactory.typed_dict_serde(serde_dict)


def union_serde(
    serde_options: List[DynPyAnySerde],
    serde_choice_fn: Callable[[Any], int],
):
    return PyAnySerdeFactory.union_serde(serde_options, serde_choice_fn)
