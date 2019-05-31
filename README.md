[![Build Status](https://dev.azure.com/matthewseyer/dfir/_apis/build/status/forensicmatt.PyRustyUsn?branchName=master)](https://dev.azure.com/matthewseyer/dfir/_build/latest?definitionId=2&branchName=master)
[![PyPI version](https://badge.fury.io/py/pyrustyusn.svg)](https://badge.fury.io/py/pyrustyusn)
# PyRustyUsn
Python bindings for RustyUsn.

This library currently implements only version 2 of USN records.

## Installation
Available on PyPi - https://pypi.org/project/pyrustyusn.

To install from PyPi - `pip install pyrustyusn`

### Wheels
Wheels are currently automatically built for python3.6 python3.7 for all 64-bit platforms (Windows, macOS, and `manylinux`).

### Installation from sources
Installation is possible for other platforms by installing from sources, this requires a nightly rust compiler and `setuptools-rust`.

Run `python setup.py install`

## Usage
```python
import sys
import ujson
import pyrustyusn


def main():
    source_location = sys.argv[1]
    usn_parser = pyrustyusn.PyUsnParser(
        source_location
    )

    for record in usn_parser.records():
        print(ujson.dumps(record))


if __name__ == "__main__":
    main()
```
