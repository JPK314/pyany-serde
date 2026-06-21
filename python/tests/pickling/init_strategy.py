# pyright: reportMissingImports=false, reportUnknownVariableType=false

import pickle

from pyany_serde import InitStrategy
from pyany_serde.pickling_init_strategy_tests import validate_eq


def test_all():
    v = InitStrategy.ALL()
    w = pickle.dumps(v)
    x = pickle.loads(w)
    validate_eq(x, v)


def test_some():
    v = InitStrategy.SOME(kwargs=["a", "b"])
    w = pickle.dumps(v)
    x = pickle.loads(w)
    validate_eq(x, v)


def test_none():
    v = InitStrategy.NONE()
    w = pickle.dumps(v)
    x = pickle.loads(w)
    validate_eq(x, v)
