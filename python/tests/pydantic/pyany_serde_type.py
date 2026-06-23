# pyright: reportMissingImports=false, reportUnknownVariableType=false, reportUnknownMemberType=false, reportMissingTypeArgument=false, reportImplicitRelativeImport=false

import pickle
import struct
from dataclasses import dataclass
from typing import Any, Literal

import numpy as np
from pyany_serde import InitStrategy, NumpySerdeConfig, PyAnySerdeType
from pyany_serde.pydantic_pyany_serde_type_tests import validate_eq
from pyany_serde.python_serde import PythonSerde
from typing_extensions import override

from pydantic import BaseModel


class MyModel(BaseModel):
    my_field: PyAnySerdeType


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


def validate_model_construction_flows(
    expected: PyAnySerdeType[Any], my_field_dict: dict[str, Any], model_json: str
):
    # constructor with instance
    actual = MyModel(my_field=expected).my_field
    validate_eq(expected, actual, "$")

    # constructor with dict
    actual = MyModel(my_field=my_field_dict).my_field  # pyright: ignore [reportArgumentType]
    validate_eq(expected, actual, "$")

    # validate json
    actual = MyModel.model_validate_json(model_json).my_field
    validate_eq(expected, actual, "$")

    # validate instance
    actual = MyModel.model_validate(MyModel(my_field=expected)).my_field
    validate_eq(expected, actual, "$")

    # validate dict
    actual = MyModel.model_validate({"my_field": my_field_dict}).my_field
    validate_eq(expected, actual, "$")


def test_bool():
    expected = PyAnySerdeType.BOOL()
    my_field_dict = {"type": "bool"}
    model_json = """
{
    "my_field": {
        "type": "bool"
    }
}
"""
    validate_model_construction_flows(expected, my_field_dict, model_json)


def test_bytes():
    expected = PyAnySerdeType.BYTES()
    my_field_dict = {"type": "bytes"}
    model_json = """
{
    "my_field": {
        "type": "bytes"
    }
}
"""
    validate_model_construction_flows(expected, my_field_dict, model_json)


def test_complex():
    expected = PyAnySerdeType.COMPLEX()
    my_field_dict = {"type": "complex"}
    model_json = """
{
    "my_field": {
        "type": "complex"
    }
}
"""
    validate_model_construction_flows(expected, my_field_dict, model_json)


def test_dataclass():
    expected = PyAnySerdeType.DATACLASS(
        MyDataclass,
        init_strategy=InitStrategy.ALL(),
        field_serde_type_dict={"a": PyAnySerdeType.INT(), "b": PyAnySerdeType.STRING()},
    )
    my_field_dict = {
        "type": "dataclass",
        "dataclass_pkl": pickle.dumps(MyDataclass).hex(),
        "init_strategy": {"type": "all"},
        "field_serde_type_dict": {"a": {"type": "int"}, "b": {"type": "string"}},
    }
    model_json = f"""
{{
    "my_field": {{
        "type": "dataclass",
        "dataclass_pkl": "{pickle.dumps(MyDataclass).hex()}",

        "init_strategy": {{
            "type": "all"
        }},
        "field_serde_type_dict": {{
            "a": {{
                "type": "int"
            }},
            "b": {{
                "type": "string"
            }}
        }}
    }}
}}
"""
    validate_model_construction_flows(expected, my_field_dict, model_json)


def test_dict():
    expected = PyAnySerdeType.DICT(
        PyAnySerdeType.STRING(),
        PyAnySerdeType.INT(),
    )
    my_field_dict = {
        "type": "dict",
        "keys_serde_type": {"type": "string"},
        "values_serde_type": {"type": "int"},
    }
    model_json = """
{
    "my_field": {
        "type": "dict",
        "keys_serde_type": {
            "type": "string"
        },
        "values_serde_type": {
            "type": "int"
        }
    }
}
"""
    validate_model_construction_flows(expected, my_field_dict, model_json)


def test_dynamic():
    expected = PyAnySerdeType.DYNAMIC()
    my_field_dict = {"type": "dynamic"}
    model_json = """
{
    "my_field": {
        "type": "dynamic"
    }
}
"""
    validate_model_construction_flows(expected, my_field_dict, model_json)


def test_float():
    expected = PyAnySerdeType.FLOAT()
    my_field_dict = {"type": "float"}
    model_json = """
{
    "my_field": {
        "type": "float"
    }
}
"""
    validate_model_construction_flows(expected, my_field_dict, model_json)


def test_int():
    expected = PyAnySerdeType.INT()
    my_field_dict = {"type": "int"}
    model_json = """
{
    "my_field": {
        "type": "int"
    }
}
"""
    validate_model_construction_flows(expected, my_field_dict, model_json)


def test_list():
    expected = PyAnySerdeType.LIST(PyAnySerdeType.INT())
    my_field_dict = {
        "type": "list",
        "items_serde_type": {"type": "int"},
    }
    model_json = """
{
    "my_field": {
        "type": "list",
        "items_serde_type": {
            "type": "int"
        }
    }
}
"""
    validate_model_construction_flows(expected, my_field_dict, model_json)


def test_numpy():
    expected = PyAnySerdeType.NUMPY(
        dtype=np.int64,
        config=NumpySerdeConfig.DYNAMIC(
            preprocessor_fn=preprocessor_fn,
            postprocessor_fn=postprocessor_fn,
        ),
    )
    my_field_dict = {
        "type": "numpy",
        "dtype": "int64",
        "config": {
            "type": "dynamic",
            "preprocessor_fn_pkl": pickle.dumps(preprocessor_fn).hex(),
            "postprocessor_fn_pkl": pickle.dumps(postprocessor_fn).hex(),
        },
    }
    model_json = f"""
{{
    "my_field": {{
        "type": "numpy",
        "dtype": "int64",
        "config": {{
            "type": "dynamic",
            "preprocessor_fn_pkl": "{pickle.dumps(preprocessor_fn).hex()}",
            "postprocessor_fn_pkl": "{pickle.dumps(postprocessor_fn).hex()}"
        }}
    }}
}}
"""
    validate_model_construction_flows(expected, my_field_dict, model_json)


def test_option():
    expected = PyAnySerdeType.OPTION(PyAnySerdeType.INT())
    my_field_dict = {
        "type": "option",
        "value_serde_type": {"type": "int"},
    }
    model_json = """
{
    "my_field": {
        "type": "option",
        "value_serde_type": {
            "type": "int"
        }
    }
}
"""
    validate_model_construction_flows(expected, my_field_dict, model_json)


def test_pickle():
    expected = PyAnySerdeType.PICKLE()
    my_field_dict = {"type": "pickle"}
    model_json = """
{
    "my_field": {
        "type": "pickle"
    }
}
"""
    validate_model_construction_flows(expected, my_field_dict, model_json)


def test_pythonserde():
    expected = PyAnySerdeType.PYTHONSERDE(MySerde())
    my_field_dict = {
        "type": "pythonserde",
        "pythonserde_pkl": pickle.dumps(MySerde()).hex(),
    }
    model_json = f"""
{{
    "my_field": {{
        "type": "pythonserde",
        "pythonserde_pkl": "{pickle.dumps(MySerde()).hex()}"
    }}
}}
"""
    validate_model_construction_flows(expected, my_field_dict, model_json)


def test_set():
    expected = PyAnySerdeType.SET(PyAnySerdeType.INT())
    my_field_dict = {
        "type": "set",
        "items_serde_type": {"type": "int"},
    }
    model_json = """
{
    "my_field": {
        "type": "set",
        "items_serde_type": {
            "type": "int"
        }
    }
}
"""
    validate_model_construction_flows(expected, my_field_dict, model_json)


def test_string():
    expected = PyAnySerdeType.STRING()
    my_field_dict = {"type": "string"}
    model_json = """
{
    "my_field": {
        "type": "string"
    }
}
"""
    validate_model_construction_flows(expected, my_field_dict, model_json)


def test_tuple():
    expected = PyAnySerdeType.TUPLE([PyAnySerdeType.INT(), PyAnySerdeType.STRING()])
    my_field_dict = {
        "type": "tuple",
        "item_serde_types": [
            {"type": "int"},
            {"type": "string"},
        ],
    }
    model_json = """
{
    "my_field": {
        "type": "tuple",
        "item_serde_types": [
            {
                "type": "int"
            },
            {
                "type": "string"
            }
        ]
    }
}
"""
    validate_model_construction_flows(expected, my_field_dict, model_json)


def test_typeddict():

    expected = PyAnySerdeType.TYPEDDICT(
        {"a": PyAnySerdeType.INT(), "b": PyAnySerdeType.STRING()}
    )
    my_field_dict = {
        "type": "typeddict",
        "key_serde_type_dict": {"a": {"type": "int"}, "b": {"type": "string"}},
    }
    model_json = """
{
    "my_field": {
        "type": "typeddict",
        "key_serde_type_dict": {
            "a": {
                "type": "int"
            },
            "b": {
                "type": "string"
            }
        }
    }
}
"""
    validate_model_construction_flows(expected, my_field_dict, model_json)


def test_union():

    expected = PyAnySerdeType.UNION(
        [PyAnySerdeType.INT(), PyAnySerdeType.STRING()],
        option_choice_fn=option_choice_fn,
    )
    my_field_dict = {
        "type": "union",
        "option_serde_types": [
            {"type": "int"},
            {"type": "string"},
        ],
        "option_choice_fn_pkl": pickle.dumps(option_choice_fn).hex(),
    }
    model_json = f"""
{{
    "my_field": {{
        "type": "union",
        "option_serde_types": [
            {{
                "type": "int"
            }},
            {{
                "type": "string"
            }}
        ],
        "option_choice_fn_pkl": "{pickle.dumps(option_choice_fn).hex()}"
    }}
}}
"""
    validate_model_construction_flows(expected, my_field_dict, model_json)
