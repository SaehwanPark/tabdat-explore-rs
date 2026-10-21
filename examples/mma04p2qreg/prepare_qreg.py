#!/usr/bin/env python3
"""Convert the pinned author ASCII source; development preparation, not a runtime dependency."""
import csv
import hashlib
from pathlib import Path
import shlex
import sys

source, target = map(Path, sys.argv[1:])
raw = source.read_bytes()
if hashlib.sha256(raw).hexdigest() != 'eb1b422fa29a096e608ff5f8912ee1b9fcecc749d227fbfd37b19d3b18db8b0b':
  raise SystemExit('unexpected qreg0902.asc SHA256; refuse conversion')
tokens = shlex.split(raw.decode('ascii'))
assert len(tokens) == 5999 * 9
# Validate everything before creating the destination; x mode refuses replacement.
rows = [[i + 1] + tokens[9*i:9*i+9] for i in range(5999)]
for row in rows:
  for j in [1, 2, 3, 6, 7, 8, 9]:
    if row[j] != '.':
      float(row[j])
with target.open('x', newline='') as f:
  writer = csv.writer(f)
  writer.writerow(['source_row', 'sex', 'age', 'educyr98', 'farm', 'urban98',
                   'hhsize', 'lhhexp1', 'lhhex12m', 'lnrlfood'])
  writer.writerows([[v if v != '.' else '' for v in row] for row in rows])
