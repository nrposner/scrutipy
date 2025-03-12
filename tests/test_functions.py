import scrutipy_rs
from scrutipy_rs import grim_scalar


def test_grim_1():
    result = grim_scalar("5.19", 40)
    assert not result


def test_grim_2():
    result = grim_scalar(5.19, 40)
    assert not result
