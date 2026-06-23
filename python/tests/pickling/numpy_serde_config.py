import pickle
from typing import Literal

import numpy as np
from pyany_serde import NumpySerdeConfig
from pyany_serde.pickling_numpy_serde_config_tests import (  # pyright:  ignore [reportMissingImports]
    validate_eq,  # pyright: ignore [reportUnknownVariableType]
)


class MyClass:
    val: int
    val2: int

    def __init__(self, val: int, val2: int):
        self.val = val
        self.val2 = val2


def preprocessor_fn(v: MyClass):
    return np.array([v.val, v.val2], dtype=np.int64)


def postprocessor_fn(v: np.ndarray[tuple[Literal[2]], np.dtype[np.int64]]):
    return MyClass(v[0], v[1])


def test_dynamic():
    expected = NumpySerdeConfig.DYNAMIC(
        preprocessor_fn=preprocessor_fn, postprocessor_fn=postprocessor_fn
    )
    actual = pickle.loads(pickle.dumps(expected))
    validate_eq(expected, actual, "$")


def test_static():
    expected = NumpySerdeConfig.STATIC(
        shape=(2,),
        preprocessor_fn=preprocessor_fn,
        postprocessor_fn=postprocessor_fn,
        allocation_pool_max_size=10,
        allocation_pool_min_size=0,
        allocation_pool_warning_size=1,
    )
    actual = pickle.loads(pickle.dumps(expected))
    validate_eq(expected, actual, "$")
