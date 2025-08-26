import scrutipy
from scrutipy import grim_scalar
from scrutipy import grim_map_pl
import pandas as pd 
import polars as pl
from scrutipy import grim_map
import pytest
from scrutipy import closure
from scrutipy import grimmer
from scrutipy import debit
from scrutipy import debit_map_pl
from scrutipy import debit_map

def test_grim_1():
    result = grim_scalar("5.19", 40)
    assert not result

def test_grim_2():
    result = grim_scalar(5.19, 40)
    assert not result

def test_grim_map_1():
    df = pl.read_csv("data/pigs1.csv")
    bools, errors = grim_map_pl(df, 1, 2, silence_numeric_warning = True) # necessary to specify the column indices in this case becase polars treats the index as the 0th column, which causes that issue
    assert bools == list([True, False, False, False, False, True, False, True, False, False, True, False])
    assert errors == None

def test_grim_map_2():
    df = pl.read_csv("data/pigs1.csv")
    bools, errors = grim_map_pl(df, 1, 2, percent = True, silence_numeric_warning = True) # necessary to specify the column indices in this case becase polars treats the index as the 0th column, which causes that issue
    assert bools == list([False, False, False, False, False, False, False, False, False, False, False, False])

def test_grim_map_3():
    df = pl.read_csv("data/pigs2.csv")
    bools, errors = grim_map_pl(df, 1, 2, percent = False, silence_numeric_warning = True) 
    assert bools == list([True, True, True, True, True, True])
    assert errors == None

def test_grim_map_4():
    df = pl.read_csv("data/pigs2.csv")
    bools, errors = grim_map_pl(df, 1, 2, percent = True, silence_numeric_warning = True) 
    assert bools == list([False, False, True, False, False, False])
    assert errors == None

def test_grim_map_pd_1():
    df = pd.read_csv("data/pigs1.csv")
    df["x"] = df["x"].astype(str)
    bools, errors = grim_map(df, 1, 2)
    assert bools == list([True, False, False, False, False, True, False, True, False, False, True, False])
    assert errors == None

def test_grim_map_pd_2():
    df = pd.read_csv("data/pigs1.csv")
    df["x"] = df["x"].astype(str)
    bools, errors = grim_map(df, 1, 2, percent = True)
    assert bools == list([False, False, False, False, False, False, False, False, False, False, False, False])
    assert errors == None

def test_grim_map_pd_3():
    df = pd.read_csv("data/pigs2.csv")
    df["x"] = df["x"].astype(str)
    bools, errors = grim_map(df, 1, 2, percent = False) 
    assert bools == list([True, True, True, True, True, True])
    assert errors == None

def test_grim_map_pd_4():
    df = pd.read_csv("data/pigs2.csv")
    df["x"] = df["x"].astype(str)
    bools, errors = grim_map(df, 1, 2, percent = True) 
    assert bools == list([False, False, True, False, False, False])
    assert errors == None

def test_grim_map_pd_5():
    df = pd.read_csv("data/pigs2.csv")
    df["x"] = df["x"].astype(str)
    bools, errors = grim_map(df, "x", "n", percent = True) 
    assert bools == list([False, False, True, False, False, False])
    assert errors == None

def test_grim_map_pd_6():
    df = pd.DataFrame({"x": ["1.1", "2.2"], "n": [10, 12]})
    bools, errors = grim_map(df, "x", "n")
    assert bools == list([True, True])
    assert errors == None

def test_grim_map_pd_7():
    df = pd.DataFrame({"x": ["1.1", "2.2"], "n": [10, 12]})
    bools, errors = grim_map(df, silence_default_warning=True)
    assert bools == list([True, True])
    assert errors == None

def test_grim_map_pd_8():
    df = pd.DataFrame({"x": ["1.1", "2.2"], "n": [10, 12]})
    bools, errors = grim_map(df, 0, 1)
    assert bools == list([True, True])
    assert errors == None

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

def test_closure_568():
    res = len(closure(3.5, 0.57, 100, 0, 7, 0.05, 0.05))
    assert res == 568

def test_closure_empty():
    assert not closure(10.0, 2.0, 3, 1, 5, 0.1, 0.1)

def test_grimmer_1():
    b = grimmer(["1.03"],
            ["0.41"],
            [40],
            "up_or_down",
            show_reason = True
        )[0]
    assert not b

def test_grimmer_2():
    bools = grimmer(["1.03", "52.13", "9.42375"], ["0.41", "2.26", "3.86"], [40, 30, 59], items = [1, 1, 1])
    assert bools == list([False, True, False]) 

def test_grimmer_3():
    bools = grimmer(["3.92"], ["2.038807"], [50], items = [1])
    assert bools == list([True]) 

def test_debit_1(): 
    results = debit(["0.36", "0.11", "0.118974"], ["0.11", "0.31", "0.6784"], [20, 40, 100])
    assert results == list([False, True, False])

def test_debit_map_pl_1():
    df = pl.read_csv("data/debit_data.csv")
    bools, errors = debit_map_pl(df, 1, 2, 3, silence_numeric_warning = True) # necessary to specify the column indices in this case becase polars treats the index as the 0th column, which causes that issue
    assert bools == list([True, True, True, False, True, True, True])
    assert errors == None

def test_debit_map_pd():
    df = pd.read_csv("data/debit_data.csv")
    df["xs"] = df["xs"].astype(str)
    df["sds"] = df["sds"].astype(str)
    bools, errors = debit_map(df, 1, 2, 3)
    assert bools == list([True, True, True, False, True, True, True])
    assert errors == None

