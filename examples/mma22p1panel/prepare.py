#!/usr/bin/env python3
"""Convert the pinned, two-line-per-record author data without clobbering."""
import csv
import hashlib
import os
from pathlib import Path
import sys
import tempfile

if len(sys.argv) != 3:
  raise SystemExit('usage: prepare.py MOMprecise.dat output.csv')
source, output = map(Path, sys.argv[1:])
data = source.read_bytes()
if hashlib.sha256(data).hexdigest() != '332c875eaa00ecc08a16c75424e4c56aa3fcf13c6ee5049768ed3d08a2bfef9c':
  raise SystemExit('unexpected source hash')
tokens = data.decode('ascii').split()
if len(tokens) != 5320 * 8:
  raise SystemExit('unexpected source dimensions')
records = [tokens[i:i + 8] for i in range(0, len(tokens), 8)]
for i, row in enumerate(records):
  if float(row[6]) != i // 10 + 1 or float(row[7]) != 1979 + i % 10:
    raise SystemExit('unexpected source panel keys or ordering')
staged = None
try:
  with tempfile.NamedTemporaryFile(mode='w', newline='', dir=output.parent, prefix='.mom22-', delete=False) as stream:
    staged = Path(stream.name)
    writer = csv.writer(stream, lineterminator='\n')
    writer.writerow(['lnhr', 'lnwg', 'kids', 'ageh', 'agesq', 'disab', 'id', 'year'])
    writer.writerows(records)
  os.link(staged, output)
finally:
  if staged is not None:
    staged.unlink()
