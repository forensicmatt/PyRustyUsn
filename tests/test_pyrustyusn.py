import pytest
from pyrustyusn import PyUsnParser


def test_it_fails_on_non_file_object():
    with pytest.raises(TypeError):
        parser = PyUsnParser(3)
