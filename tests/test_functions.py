import scrutipy
from scrutipy import grim_scalar
from scrutipy import grim_map_df
import pandas as pd 
import polars as pl

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

