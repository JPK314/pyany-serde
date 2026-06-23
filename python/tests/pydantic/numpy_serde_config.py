import pickle
from typing import Any, Literal

import numpy as np
from pyany_serde import NumpySerdeConfig
from pyany_serde.pydantic_numpy_serde_config_tests import (  # pyright:  ignore [reportMissingImports]
    validate_eq,  # pyright: ignore [reportUnknownVariableType]
)

from pydantic import BaseModel


class MyModel(BaseModel):
    my_field: NumpySerdeConfig


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


def validate_model_construction_flows(
    expected: NumpySerdeConfig, my_field_dict: dict[str, Any], model_json: str
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


def test_dynamic():
    expected = NumpySerdeConfig.DYNAMIC(
        preprocessor_fn=preprocessor_fn, postprocessor_fn=postprocessor_fn
    )
    my_field_dict = {
        "type": "dynamic",
        "preprocessor_fn_pkl": pickle.dumps(preprocessor_fn).hex(),
        "postprocessor_fn_pkl": pickle.dumps(postprocessor_fn).hex(),
    }
    model_json = f"""
{{
    "my_field": {{
        "type": "dynamic",
        "preprocessor_fn_pkl": "{pickle.dumps(preprocessor_fn).hex()}",
        "postprocessor_fn_pkl": "{pickle.dumps(postprocessor_fn).hex()}"
    }}
}}
"""
    validate_model_construction_flows(expected, my_field_dict, model_json)


def test_static():
    expected = NumpySerdeConfig.STATIC(
        shape=(2,),
        preprocessor_fn=preprocessor_fn,
        postprocessor_fn=postprocessor_fn,
        allocation_pool_min_size=0,
        allocation_pool_max_size=10,
        allocation_pool_warning_size=1,
    )
    my_field_dict = {
        "type": "static",
        "shape": [2],
        "preprocessor_fn_pkl": pickle.dumps(preprocessor_fn).hex(),
        "postprocessor_fn_pkl": pickle.dumps(postprocessor_fn).hex(),
        "allocation_pool_min_size": 0,
        "allocation_pool_max_size": 10,
        "allocation_pool_warning_size": 1,
    }
    model_json = f"""
{{
    "my_field": {{
        "type": "static",
        "shape": [2],
        "preprocessor_fn_pkl": "{pickle.dumps(preprocessor_fn).hex()}",
        "postprocessor_fn_pkl": "{pickle.dumps(postprocessor_fn).hex()}",
        "allocation_pool_min_size": 0,
        "allocation_pool_max_size": 10,
        "allocation_pool_warning_size": 1
    }}
}}
"""
    validate_model_construction_flows(expected, my_field_dict, model_json)
