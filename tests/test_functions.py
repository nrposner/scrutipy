import scrutipy_rs
from scrutipy_rs import grim_scalar


def test_grim():
    print("Testing")
    result = grim_scalar("5.19", 40)
    assert not result
