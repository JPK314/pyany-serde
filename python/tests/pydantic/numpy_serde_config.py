# pyright: reportMissingImports=false, reportUnknownVariableType=false

import pickle
from typing import Literal

import numpy as np
from pyany_serde import NumpySerdeConfig
from pyany_serde.pydantic_numpy_serde_config_tests import (
    validate_dynamic,
    validate_static,
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


def validate_dynamic2(v: NumpySerdeConfig):
    validate_dynamic(v, preprocessor_fn, postprocessor_fn)


def validate_static2(v: NumpySerdeConfig):
    validate_static(v, preprocessor_fn, postprocessor_fn)


def test_constructor_basic():
    v = MyModel(
        my_field=NumpySerdeConfig.DYNAMIC(
            preprocessor_fn=preprocessor_fn, postprocessor_fn=postprocessor_fn
        )
    )
    validate_dynamic2(v.my_field)
    v = MyModel(
        my_field=NumpySerdeConfig.STATIC(
            shape=(2,),
            preprocessor_fn=preprocessor_fn,
            postprocessor_fn=postprocessor_fn,
            allocation_pool_min_size=0,
            allocation_pool_max_size=10,
            allocation_pool_warning_size=1,
        )
    )
    validate_static2(v.my_field)


def test_constructor_dict():
    v = MyModel(
        my_field={
            "type": "dynamic",
            "preprocessor_fn_pkl": pickle.dumps(preprocessor_fn).hex(),
            "postprocessor_fn_pkl": pickle.dumps(postprocessor_fn).hex(),
        }  # pyright: ignore [reportArgumentType]
    )
    validate_dynamic2(v.my_field)
    v = MyModel(
        my_field={
            "type": "static",
            "shape": [2],
            "preprocessor_fn_pkl": pickle.dumps(preprocessor_fn).hex(),
            "postprocessor_fn_pkl": pickle.dumps(postprocessor_fn).hex(),
            "allocation_pool_min_size": 0,
            "allocation_pool_max_size": 10,
            "allocation_pool_warning_size": 1,
        }  # pyright: ignore [reportArgumentType]
    )
    validate_static2(v.my_field)


def test_model_validate_json():
    v = MyModel.model_validate_json(f"""
{{
    "my_field": {{
        "type": "dynamic",
        "preprocessor_fn_pkl": "{pickle.dumps(preprocessor_fn).hex()}",
        "postprocessor_fn_pkl": "{pickle.dumps(postprocessor_fn).hex()}"
    }}
}}
""")
    validate_dynamic2(v.my_field)
    v = MyModel.model_validate_json(f"""
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
""")
    validate_static2(v.my_field)


def test_model_validate_instance():
    v = MyModel.model_validate(
        MyModel(
            my_field=NumpySerdeConfig.DYNAMIC(
                preprocessor_fn=preprocessor_fn, postprocessor_fn=postprocessor_fn
            )
        )
    )
    validate_dynamic2(v.my_field)
    v = MyModel.model_validate(
        MyModel(
            my_field=NumpySerdeConfig.STATIC(
                shape=(2,),
                preprocessor_fn=preprocessor_fn,
                postprocessor_fn=postprocessor_fn,
                allocation_pool_max_size=10,
                allocation_pool_min_size=0,
                allocation_pool_warning_size=1,
            )
        )
    )
    validate_static2(v.my_field)


def test_model_validate_dict():
    v = MyModel.model_validate(
        {
            "my_field": {
                "type": "dynamic",
                "preprocessor_fn_pkl": pickle.dumps(preprocessor_fn).hex(),
                "postprocessor_fn_pkl": pickle.dumps(postprocessor_fn).hex(),
            }
        }
    )
    validate_dynamic2(v.my_field)
    v = MyModel.model_validate(
        {
            "my_field": {
                "type": "static",
                "shape": [2],
                "preprocessor_fn_pkl": pickle.dumps(preprocessor_fn).hex(),
                "postprocessor_fn_pkl": pickle.dumps(postprocessor_fn).hex(),
                "allocation_pool_min_size": 0,
                "allocation_pool_max_size": 10,
                "allocation_pool_warning_size": 1,
            }
        }
    )
    validate_static2(v.my_field)
