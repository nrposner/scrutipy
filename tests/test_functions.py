import scrutipy_rs
from scrutipy_rs import grim_scalar
from scrutipy_rs import grim_map_df
import pandas as pd 

def test_grim_1():
    result = grim_scalar("5.19", 40)
    assert not result


def test_grim_2():
    result = grim_scalar(5.19, 40)
    assert not result

def test_grim_map_1():

    df = pd.read_csv("data/pigs1.csv")

    bools, errors = grim_map_df(df)

    assert bools == list(True, False, False, False, False, True, False, True, False, False, True, False)
