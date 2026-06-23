# pyright: reportMissingImports=false, reportUnknownVariableType=false

import pickle

from pyany_serde import InitStrategy
from pyany_serde.pickling_init_strategy_tests import validate_eq


def test_all():
    expected = InitStrategy.ALL()
    actual = pickle.loads(pickle.dumps(expected))
    validate_eq(expected, actual, "$")


def test_some():
    expected = InitStrategy.SOME(kwargs=["a", "b"])
    actual = pickle.loads(pickle.dumps(expected))
    validate_eq(expected, actual, "$")


def test_none():
    expected = InitStrategy.NONE()
    actual = pickle.loads(pickle.dumps(expected))
    validate_eq(expected, actual, "$")
