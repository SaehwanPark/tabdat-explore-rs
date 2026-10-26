#!/usr/bin/env python3
"""Explicit development-only fixed-width PSID conversion; never overwrite output."""
import csv
from pathlib import Path
import sys

FIELDS = [('intnum',0,4),('persnum',4,7),('age',7,9),('educatn',9,11),
  ('earnings',11,17),('hours',17,21),('sex',21,22),('kids',22,24),('married',24,25)]


def convert(source, target):
  # Validate every record before creating a target; raw codes/ordinals are retained.
  records = []
  for ordinal, line in enumerate(Path(source).read_text(encoding='ascii').splitlines(), 1):
    assert len(line) == 25, f'record {ordinal}: expected exactly25 columns'
    values = []
    for name, start, stop in FIELDS:
      text = line[start:stop].strip()
      assert text.isascii() and (text.isdecimal() or not text), (ordinal, name, text)
      assert text or name == 'educatn', (ordinal, name, 'unexpected missing field')
      values.append(int(text) if text else '')
    records.append([ordinal, *values])
  assert records, 'empty source'
  with open(target, 'x', newline='') as stream:
    writer = csv.writer(stream)
    writer.writerow(['source_row', *[name for name, _, _ in FIELDS]])
    writer.writerows(records)


if __name__ == '__main__':
  assert len(sys.argv) == 3, 'usage: prepare.py psidf3050.dat NEW_psid.csv'
  convert(*sys.argv[1:])
