#!/usr/bin/env python3
"""Prepare the pinned author NSW/PSID ASCII data without overwriting output."""
import csv
import hashlib
from pathlib import Path
import sys

if len(sys.argv) != 3:
  raise SystemExit('usage: prepare.py nswpsid.da1 output.csv')
source, output = map(Path, sys.argv[1:])
data = source.read_bytes()
assert hashlib.sha256(data).hexdigest() == '810f6dbfbe0ffbcc52676946c330f49d4d8bf5cafc06980e912ca945ac7f4e44', 'unexpected source hash'
records = [line.split() for line in data.decode('ascii').splitlines() if line.strip()]
assert len(records) == 2675 and all(len(row) == 11 for row in records)
with output.open('x', newline='') as stream:
  writer = csv.writer(stream, lineterminator='\n')
  writer.writerow(['source_row','treat','age','educ','black','hisp','marr','re74','re75','re78','u74_source','u75_source'])
  writer.writerows([[i,*row] for i,row in enumerate(records,1)])
