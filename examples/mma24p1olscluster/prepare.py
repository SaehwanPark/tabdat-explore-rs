#!/usr/bin/env python3
"""Convert the pinned publisher ASCII export to ordered numeric CSV."""
import csv
import hashlib
import shlex
import sys
from pathlib import Path

SOURCE_SHA256 = '74cb37e8c14609e382d334e368a54f63bccd4e952d02bcbf535baf19115d4061'
FARM = {'non farm': 0, 'farm': 1}
EDUC = {'Never': 0, 'Nhatre': 1, '<cap I': 2, 'Cap I': 3, 'Cap II': 4,
        'Cap III': 5, 'Nghe SC': 6, 'THCN': 7, 'DHDC': 8, 'DHCD': 9,
        'Thac sy': 10, 'PTS': 11, 'TS': 12}


def main():
  if len(sys.argv) != 3:
    raise SystemExit('usage: prepare.py vietnam_ex1.asc vietnam.csv')
  source, target = map(Path, sys.argv[1:])
  raw = source.read_bytes()
  if hashlib.sha256(raw).hexdigest() != SOURCE_SHA256:
    raise SystemExit('unexpected vietnam_ex1.asc SHA256; refuse conversion')
  tokens = shlex.split(raw.decode('ascii'))
  if len(tokens) != 5999 * 8:
    raise SystemExit('unexpected logical record count')
  rows = []
  for i in range(5999):
    outcome, age, sex, size, farm, educ, expenditure, commune = tokens[8*i:8*i+8]
    if farm not in FARM or educ not in EDUC:
      raise SystemExit(f'unknown category at source row {i+1}')
    rows.append([i+1, '' if outcome == '.' else outcome, age, sex, size,
                 FARM[farm], EDUC[educ], expenditure, commune])
  if sum(row[1] == '' for row in rows) != 993:
    raise SystemExit('unexpected missing-outcome count')
  # Validate before exclusive creation, so source/owner files are never replaced.
  with target.open('x', newline='') as out:
    writer = csv.writer(out)
    writer.writerow(['source_row', 'lnexp12m', 'age', 'sex', 'hhsize', 'farm',
                     'educ', 'lnhhexp', 'commune'])
    writer.writerows(rows)


if __name__ == '__main__':
  main()
