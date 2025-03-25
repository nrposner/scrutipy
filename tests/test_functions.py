import scrutipy
from scrutipy import grim_scalar
from scrutipy import grim_map_df
import pandas as pd 
import polars as pl
from scrutipy import grim_map
import pytest

def test_grim_1():
    result = grim_scalar("5.19", 40)
    assert not result


def test_grim_2():
    result = grim_scalar(5.19, 40)
    assert not result

def test_grim_map_1():
    df = pl.read_csv("data/pigs1.csv")
    bools, errors = grim_map_df(df, 1, 2, silence_numeric_warning = True) # necessary to specify the column indices in this case becase polars treats the index as the 0th column, which causes that issue
    assert bools == list([True, False, False, False, False, True, False, True, False, False, True, False])

def test_grim_map_2():
    df = pl.read_csv("data/pigs1.csv")
    bools, errors = grim_map_df(df, 1, 2, percent = True, silence_numeric_warning = True) # necessary to specify the column indices in this case becase polars treats the index as the 0th column, which causes that issue
    assert bools == list([False, False, False, False, False, False, False, False, False, False, False, False])

def test_grim_map_3():
    df = pl.read_csv("data/pigs2.csv")
    bools, errors = grim_map_df(df, 1, 2, percent = False, silence_numeric_warning = True) 
    assert bools == list([True, True, True, True, True, True])

def test_grim_map_4():
    df = pl.read_csv("data/pigs2.csv")
    bools, errors = grim_map_df(df, 1, 2, percent = True, silence_numeric_warning = True) 
    assert bools == list([False, False, True, False, False, False])

def test_grim_map_pd_1():
    df = pd.read_csv("data/pigs1.csv")
    bools, errors = grim_map(df, 1, 2, silence_numeric_warning = True)
    assert bools == list([True, False, False, False, False, True, False, True, False, False, True, False])

def test_grim_map_pd_2():
    df = pd.read_csv("data/pigs1.csv")
    bools, errors = grim_map(df, 1, 2, percent = True, silence_numeric_warning = True)
    assert bools == list([False, False, False, False, False, False, False, False, False, False, False, False])

def test_grim_map_pd_3():
    df = pd.read_csv("data/pigs2.csv")
    bools, errors = grim_map(df, 1, 2, percent = False, silence_numeric_warning = True) 
    assert bools == list([True, True, True, True, True, True])

def test_grim_map_pd_4():
    df = pd.read_csv("data/pigs2.csv")
    bools, errors = grim_map(df, 1, 2, percent = True, silence_numeric_warning = True) 
    assert bools == list([False, False, True, False, False, False])

def test_grim_map_pd_5():
    df = pd.read_csv("data/pigs2.csv")
    bools, errors = grim_map(df, "x", "n", percent = True, silence_numeric_warning = True) 
    assert bools == list([False, False, True, False, False, False])

def test_grim_map_pd_6():
    df = pd.DataFrame({"x": ["1.1", "2.2"], "n": [10, 12]})
    bools, errors = grim_map(df, "x", "n")
    assert bools == list([True, True])

def test_grim_map_pd_7():
    df = pd.DataFrame({"x": ["1.1", "2.2"], "n": [10, 12]})
    bools, errors = grim_map(df, silence_default_warning=True)
    assert bools == list([True, True])

def test_grim_map_pd_8():
    df = pd.DataFrame({"x": ["1.1", "2.2"], "n": [10, 12]})
    bools, errors = grim_map(df, 0, 1)
    assert bools == list([True, True])

def test_invalid_column_name_raises():
    df = pd.DataFrame({"x": ["1.1", "2.2"], "n": [10, 12]})

    with pytest.raises(ValueError) as excinfo:
        grim_map(df, x_col="z")

    assert "x_col" in str(excinfo.value)
    assert "'z'" in str(excinfo.value)
    assert "not found" in str(excinfo.value)

def test_index_out_of_bounds_raises():
    df = pd.DataFrame({"x": ["1.1", "2.2"], "n": [10, 12]})
    with pytest.raises(IndexError) as excinfo:
        grim_map(df, x_col=5)

    assert "x_col" in str(excinfo.value)
    assert "'5'" in str(excinfo.value)
    assert "out of bounds" in str(excinfo.value)

def test_invalid_x_dtype_raises():
    df = pd.DataFrame({"x": [[], []], "n": [10, 12]})  # list column
    with pytest.raises(TypeError, match="x_col column is composed of neither strings nor numeric types"):
        grim_map(df, silence_default_warning = True)

def test_invalid_n_dtype_raises():
    df = pd.DataFrame({"x": ["1.1", "2.2"], "n": [None, "abc"]})
    bools, errors = grim_map(df, silence_default_warning=True)
    assert bools == []
    assert errors == [0, 1]


def test_default_warning_triggered(monkeypatch):
    df = pd.DataFrame({"x": ["1.1", "2.2"], "n": [10, 12]})
    caught = []

    def fake_warn(msg, *args, **kwargs):
        caught.append(str(msg))

    import warnings
    monkeypatch.setattr(warnings, "warn", fake_warn)

    grim_map(df)
    assert any("haven't been changed from their defaults" in w for w in caught)

def test_default_warning_suppressed(monkeypatch):
    df = pd.DataFrame({"x": ["1.1", "2.2"], "n": [10, 12]})
    caught = []

    def fake_warn(msg, *args, **kwargs):
        caught.append(str(msg))

    import warnings
    monkeypatch.setattr(warnings, "warn", fake_warn)

    grim_map(df, silence_default_warning=True)
    assert not caught

def test_empty_dataframe():
    df = pd.DataFrame({"x": [], "n": []})

    with pytest.raises(TypeError, match="The x_col column is empty."):
        grim_map(df, silence_default_warning = True, silence_numeric_warning = True)

