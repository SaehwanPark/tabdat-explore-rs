#!/usr/bin/env python3
"""Development-only original ASCII conversion; no network or production dependency."""
import csv
import hashlib
import math
import os
from pathlib import Path
import re
import sys
import tempfile

SOURCE_SHA256 = '460f5d731f1848c8695d78a1fbf0db348d0daced4b8b5b0c2734bffa20b3e31c'
NUMBER = re.compile(r'[+-]?(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)(?:[eE][+-]?[0-9]+)?\Z')


def convert(source, target):
  raw = Path(source).read_bytes()
  if hashlib.sha256(raw).hexdigest() != SOURCE_SHA256:
    raise ValueError('expected the pinned original mma05data.asc SHA256')
  records = []
  for ordinal, line in enumerate(raw.decode('ascii').splitlines(), 1):
    fields = line.split()
    if len(fields) != 2 or any(not NUMBER.fullmatch(v) for v in fields):
      raise ValueError(f'record{ordinal}: expected exactly two finite numeric y,x tokens')
    y, x = (float(v) for v in fields)
    if not math.isfinite(y) or not math.isfinite(x) or y <= 0:
      raise ValueError(f'record{ordinal}: expected finite x and positive finite generated y')
    records.append([ordinal, *fields])
  if len(records) != 10000:
    raise ValueError('expected exactly10000 generated observations')
  # Same-directory staging/link publication prevents clobber or partial CSV on
  # validation/write/publication failure; this is not a crash-durability promise.
  staged = None
  try:
    with tempfile.NamedTemporaryFile(mode='w', newline='', encoding='ascii',
                                    dir=Path(target).parent, prefix='.nls-prepare-',
                                    delete=False) as stream:
      staged = Path(stream.name)
      writer = csv.writer(stream)
      writer.writerow(['source_row', 'y', 'x'])
      writer.writerows(records)
    os.link(staged, target)
  finally:
    if staged is not None:
      staged.unlink()


if __name__ == '__main__':
  if len(sys.argv) != 3:
    raise SystemExit('usage: prepare.py ORIGINAL_mma05data.asc NEW_nls.csv')
  convert(*sys.argv[1:])
