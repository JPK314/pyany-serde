# pyright: reportMissingImports=false, reportUnknownVariableType=false

from pyany_serde import InitStrategy
from pyany_serde.pydantic_init_strategy_tests import (
    validate_all,
    validate_none,
    validate_some,
)

from pydantic import BaseModel


class MyModel(BaseModel):
    my_field: InitStrategy


def test_constructor_basic():
    v = MyModel(my_field=InitStrategy.ALL())
    validate_all(v.my_field)
    v = MyModel(my_field=InitStrategy.SOME(["a", "b"]))
    validate_some(v.my_field)
    v = MyModel(my_field=InitStrategy.NONE())
    validate_none(v.my_field)


def test_constructor_dict():
    v = MyModel(my_field={"type": "all"})  # pyright: ignore [reportArgumentType]
    validate_all(v.my_field)
    v = MyModel(my_field={"type": "some", "kwargs": ["a", "b"]})  # pyright: ignore [reportArgumentType]
    validate_some(v.my_field)
    v = MyModel(my_field={"type": "none"})  # pyright: ignore [reportArgumentType]
    validate_none(v.my_field)


def test_model_validate_json():
    v = MyModel.model_validate_json("""
{
    "my_field": {
        "type": "all"
    }
}
""")
    validate_all(v.my_field)
    v = MyModel.model_validate_json("""
{
    "my_field": {
        "type": "some",
        "kwargs": ["a", "b"]
    }
}
""")
    validate_some(v.my_field)
    v = MyModel.model_validate_json("""
{
    "my_field": {
        "type": "none"
    }
}
""")
    validate_none(v.my_field)


def test_model_validate_instance():
    v = MyModel.model_validate(MyModel(my_field=InitStrategy.ALL()))
    validate_all(v.my_field)
    v = MyModel.model_validate(MyModel(my_field=InitStrategy.SOME(["a", "b"])))
    validate_some(v.my_field)
    v = MyModel.model_validate(MyModel(my_field=InitStrategy.NONE()))
    validate_none(v.my_field)


def test_model_validate_dict():
    v = MyModel.model_validate({"my_field": {"type": "all"}})
    validate_all(v.my_field)
    v = MyModel.model_validate({"my_field": {"type": "some", "kwargs": ["a", "b"]}})
    validate_some(v.my_field)
    v = MyModel.model_validate({"my_field": {"type": "none"}})
    validate_none(v.my_field)
