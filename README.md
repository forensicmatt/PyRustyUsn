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

# Scripts
## usn_dump.py
### Usage
```
usage: usn_dump.py [-h] -s SOURCE [-v] [--debug {ERROR,WARN,INFO,DEBUG}]

Parse USN records from a file, extracted unallocated, or volume handle (file system or volume shadow).

optional arguments:
  -h, --help            show this help message and exit
  -s SOURCE, --source SOURCE
                        The USN Journal, directory, or a logical volume
                        (logical volume handle: \\.\C: or
                        \\?\GLOBALROOT\Device\HarddiskVolumeShadowCopy6). If
                        source is a directory, it will recurse through the
                        folders looking for files that end with $J.
  -v, --is_volume       The source is a volume handle.
  --debug {ERROR,WARN,INFO,DEBUG}
                        Debug level [default=ERROR]
```

### Examples
Parse an extracted journal and count the records parsed.
```
D:\Testing>usn_dump.py -s D:\Images\CTF_DEFCON_2018\Image3-Desktop\KAPE\E\$Extend\$J > j_dump.jsonl

D:\Testing>rg -U -c "" j_dump.jsonl
325673
```

Parse usn records from a live journal and count the records parsed.
```
d:\Testing>usn_dump.py -s \\.\E: > e_live.jsonl

d:\Testing>rg -U -c "" e_live.jsonl
325673
```

Parse usn records from a volume shadow and count the records parsed.
```
d:\Testing>vssadmin list shadows /for=e:
vssadmin 1.1 - Volume Shadow Copy Service administrative command-line tool
(C) Copyright 2001-2013 Microsoft Corp.

Contents of shadow copy set ID: {95312ff2-8cc7-48e1-953f-382de6134b41}
   Contained 1 shadow copies at creation time: 8/1/2018 10:42:55 PM
      Shadow Copy ID: {96cf998b-cba8-4561-9679-e7acb25c9311}
         Original Volume: (?)\\?\Volume{f144c361-0000-0000-0000-602200000000}\
         Shadow Copy Volume: \\?\GLOBALROOT\Device\HarddiskVolumeShadowCopy9
         Originating Machine: DESKTOP-1N4R894
         Service Machine: DESKTOP-1N4R894
         Provider: 'Microsoft Software Shadow Copy provider 1.0'
         Type: ClientAccessibleWriters
         Attributes: Persistent, Client-accessible, No auto release, Differential, Auto recovered

Contents of shadow copy set ID: {dba3e4ca-1aa5-4484-b02b-2aaa618dd8b2}
   Contained 1 shadow copies at creation time: 8/3/2018 12:44:31 AM
      Shadow Copy ID: {25791e95-f7f5-4f08-a1ac-e01443e85c52}
         Original Volume: (?)\\?\Volume{f144c361-0000-0000-0000-602200000000}\
         Shadow Copy Volume: \\?\GLOBALROOT\Device\HarddiskVolumeShadowCopy10
         Originating Machine: DESKTOP-1N4R894
         Service Machine: DESKTOP-1N4R894
         Provider: 'Microsoft Software Shadow Copy provider 1.0'
         Type: ClientAccessibleWriters
         Attributes: Persistent, Client-accessible, No auto release, Differential, Auto recovered
....

d:\Testing>usn_dump.py -s \\?\GLOBALROOT\Device\HarddiskVolumeShadowCopy9 > ShadowCopy9.jsonl

d:\Testing>rg -U -c "" ShadowCopy9.jsonl
11564
```