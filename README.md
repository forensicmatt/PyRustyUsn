# PyRustyUsn
Python bindings for RustyUsn.

This library currently implements only version 2 of USN records.

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
