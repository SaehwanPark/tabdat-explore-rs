#!/usr/bin/env python3
"""Convert the pinned, wrapped publisher ASCII to ordered numeric CSV."""
import csv
import hashlib
import shlex
import sys
from pathlib import Path


def main():
  if len(sys.argv)!=3:
    raise SystemExit('usage: prepare.py vietnam_ex2.asc vietnam_individual.csv')
  source, target = map(Path,sys.argv[1:])
  raw = source.read_bytes()
  if hashlib.sha256(raw).hexdigest()!='e876f8ee17791881c6378de2761436e1e72e1eda84347089a9dc365ed290c85d':
    raise SystemExit('unexpected vietnam_ex2.asc SHA256; refuse conversion')
  tokens = shlex.split(raw.decode('ascii'))
  if len(tokens)!=27766*12:
    raise SystemExit('unexpected logical record count')
  rows = [tokens[12*i:12*i+12] for i in range(27766)]
  if [i+1 for i,r in enumerate(rows) if '.' in r] != [27734] or rows[27733] != ['.']*12:
    raise SystemExit('unexpected missing-data pattern')
  for row in rows:
    for token in row:
      if token!='.':
        float(token)
  # Validate the pinned source before exclusive output creation.
  with target.open('x',newline='') as f:
    writer = csv.writer(f)
    writer.writerow(['source_row','pharvis','lnhhexp','age','sex','married','educ',
                     'illness','injury','illdays','actdays','insurance','commune'])
    writer.writerows([[i+1]+['' if token=='.' else token for token in row]
                      for i,row in enumerate(rows)])


if __name__=='__main__':
  main()
