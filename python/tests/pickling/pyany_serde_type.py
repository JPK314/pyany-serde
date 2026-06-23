import pickle
import struct
from dataclasses import dataclass
from typing import Any, Literal, TypedDict

import numpy as np
from numpy.typing import NDArray
from pyany_serde import InitStrategy, NumpySerdeConfig, PyAnySerdeType
from pyany_serde.pickling_pyany_serde_type_tests import (  # pyright:  ignore [reportMissingImports]
    validate_eq,  # pyright: ignore [reportUnknownVariableType]
)
from pyany_serde.python_serde import (  # pyright: ignore [reportImplicitRelativeImport]
    PythonSerde,
)
from typing_extensions import override


class MyClass:
    val: int
    val2: int

    def __init__(self, val: int, val2: int):
        self.val = val
        self.val2 = val2


@dataclass
class MyDataclass:
    a: int
    b: str


class MyTypedDict(TypedDict):
    a: int
    b: str


class MySerde(PythonSerde[MyClass]):
    fmt: str = "=ll"

    @override
    def append(self, buf: memoryview, offset: int, obj: MyClass) -> int:
        b = struct.pack(self.fmt, obj.val, obj.val2)
        end = offset + len(b)
        buf[offset:end] = b
        return end

    @override
    def get_bytes(self, start_addr: int | None, obj: MyClass) -> bytes:
        return struct.pack(self.fmt, obj.val, obj.val2)

    @override
    def retrieve(self, buf: memoryview, offset: int) -> tuple[MyClass, int]:
        size = struct.calcsize(self.fmt)
        end = offset + size
        (val, val2) = struct.unpack(self.fmt, buf[offset:end])
        return (MyClass(val, val2), end)

    # The below method is just for testing equality
    @override
    def __eq__(self, other: Any):
        return isinstance(other, MySerde)


def preprocessor_fn(v: MyClass):
    return np.array([v.val, v.val2], dtype=np.int64)


def postprocessor_fn(v: np.ndarray[tuple[Literal[2]], np.dtype[np.int64]]):
    return MyClass(v[0], v[1])


def option_choice_fn(v: int | str):
    if isinstance(v, int):
        return 0
    else:
        return 1


def test_bool():
    expected: PyAnySerdeType[bool] = PyAnySerdeType.BOOL()
    actual = pickle.loads(pickle.dumps(expected))
    validate_eq(expected, actual, "$")


def test_bytes():
    expected: PyAnySerdeType[bytes] = PyAnySerdeType.BYTES()
    actual = pickle.loads(pickle.dumps(expected))
    validate_eq(expected, actual, "$")


def test_complex():
    expected: PyAnySerdeType[complex] = PyAnySerdeType.COMPLEX()
    actual = pickle.loads(pickle.dumps(expected))
    validate_eq(expected, actual, "$")


def test_dataclass():
    expected: PyAnySerdeType[MyDataclass] = PyAnySerdeType.DATACLASS(
        MyDataclass,
        init_strategy=InitStrategy.ALL(),
        field_serde_type_dict={"a": PyAnySerdeType.INT(), "b": PyAnySerdeType.STRING()},
    )
    actual = pickle.loads(pickle.dumps(expected))
    validate_eq(expected, actual, "$")


def test_dict():
    expected: PyAnySerdeType[dict[str, int]] = PyAnySerdeType.DICT(
        keys_serde_type=PyAnySerdeType.STRING(),
        values_serde_type=PyAnySerdeType.INT(),
    )
    actual = pickle.loads(pickle.dumps(expected))
    validate_eq(expected, actual, "$")


def test_dynamic():
    expected: PyAnySerdeType[Any] = PyAnySerdeType.DYNAMIC()
    actual = pickle.loads(pickle.dumps(expected))
    validate_eq(expected, actual, "$")


def test_float():
    expected: PyAnySerdeType[float] = PyAnySerdeType.FLOAT()
    actual = pickle.loads(pickle.dumps(expected))
    validate_eq(expected, actual, "$")


def test_int():
    expected: PyAnySerdeType[int] = PyAnySerdeType.INT()
    actual = pickle.loads(pickle.dumps(expected))
    validate_eq(expected, actual, "$")


def test_list():
    expected: PyAnySerdeType[list[int]] = PyAnySerdeType.LIST(
        items_serde_type=PyAnySerdeType.INT(),
    )
    actual = pickle.loads(pickle.dumps(expected))
    validate_eq(expected, actual, "$")


def test_numpy_dynamic():
    expected: PyAnySerdeType[NDArray[np.int64]] = PyAnySerdeType.NUMPY(
        dtype=np.int64,
        config=NumpySerdeConfig.DYNAMIC(
            preprocessor_fn=preprocessor_fn,
            postprocessor_fn=postprocessor_fn,
        ),
    )
    actual = pickle.loads(pickle.dumps(expected))
    validate_eq(expected, actual, "$")


def test_numpy_static():
    expected: PyAnySerdeType[np.ndarray[tuple[Literal[2]], np.dtype[np.int64]]] = (
        PyAnySerdeType.NUMPY(
            dtype=np.int64,
            config=NumpySerdeConfig.STATIC(
                shape=(2,),
                preprocessor_fn=preprocessor_fn,
                postprocessor_fn=postprocessor_fn,
                allocation_pool_min_size=0,
                allocation_pool_max_size=10,
                allocation_pool_warning_size=1,
            ),
        )
    )
    actual = pickle.loads(pickle.dumps(expected))
    validate_eq(expected, actual, "$")


def test_option():
    expected: PyAnySerdeType[int | None] = PyAnySerdeType.OPTION(
        value_serde_type=PyAnySerdeType.INT(),
    )
    actual = pickle.loads(pickle.dumps(expected))
    validate_eq(expected, actual, "$")


def test_pickle():
    expected: PyAnySerdeType[Any] = PyAnySerdeType.PICKLE()
    actual = pickle.loads(pickle.dumps(expected))
    validate_eq(expected, actual, "$")


def test_pythonserde():
    expected: PyAnySerdeType[MyClass] = PyAnySerdeType.PYTHONSERDE(
        python_serde=MySerde(),
    )
    actual = pickle.loads(pickle.dumps(expected))
    validate_eq(expected, actual, "$")


def test_set():
    expected: PyAnySerdeType[set[int]] = PyAnySerdeType.SET(
        items_serde_type=PyAnySerdeType.INT(),
    )
    actual = pickle.loads(pickle.dumps(expected))
    validate_eq(expected, actual, "$")


def test_string():
    expected: PyAnySerdeType[str] = PyAnySerdeType.STRING()
    actual = pickle.loads(pickle.dumps(expected))
    validate_eq(expected, actual, "$")


def test_tuple():
    expected: PyAnySerdeType[tuple[int, str]] = PyAnySerdeType.TUPLE(
        item_serde_types=[
            PyAnySerdeType.INT(),
            PyAnySerdeType.STRING(),
        ]
    )
    actual = pickle.loads(pickle.dumps(expected))
    validate_eq(expected, actual, "$")


def test_typeddict():
    expected: PyAnySerdeType[MyTypedDict] = PyAnySerdeType.TYPEDDICT[MyTypedDict](
        {"a": PyAnySerdeType.INT(), "b": PyAnySerdeType.STRING()}
    )
    actual = pickle.loads(pickle.dumps(expected))
    validate_eq(expected, actual, "$")


def test_union():
    expected: PyAnySerdeType[int | str] = PyAnySerdeType.UNION(
        [PyAnySerdeType.INT(), PyAnySerdeType.STRING()],
        option_choice_fn=option_choice_fn,
    )
    actual = pickle.loads(pickle.dumps(expected))
    validate_eq(expected, actual, "$")
