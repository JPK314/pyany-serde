# pyright: reportMissingImports=false, reportUnknownVariableType=false, reportUnknownMemberType=false, reportUnknownParameterType=false
from typing import Any

from pyany_serde import InitStrategy
from pyany_serde.pydantic_init_strategy_tests import validate_eq

from pydantic import BaseModel


class MyModel(BaseModel):
    my_field: InitStrategy


def validate_model_construction_flows(
    expected: InitStrategy, my_field_dict: dict[str, Any], model_json: str
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


def test_all():
    expected = InitStrategy.ALL()
    my_field_dict = {"type": "all"}
    model_json = """
{
    "my_field": {
        "type": "all"
    }
}
"""
    validate_model_construction_flows(expected, my_field_dict, model_json)


def test_some():
    expected = InitStrategy.SOME(["a", "b"])
    my_field_dict = {"type": "some", "kwargs": ["a", "b"]}
    model_json = """
{
    "my_field": {
        "type": "some",
        "kwargs": ["a", "b"]
    }
}
"""
    validate_model_construction_flows(expected, my_field_dict, model_json)


def test_none():
    expected = InitStrategy.NONE()
    my_field_dict = {"type": "none"}
    model_json = """
{
    "my_field": {
        "type": "none"
    }
}
"""
    validate_model_construction_flows(expected, my_field_dict, model_json)
