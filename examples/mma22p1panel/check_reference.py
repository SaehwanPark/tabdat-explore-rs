#!/usr/bin/env python3
"""Check the source sample, design, coefficients and covariance against R."""
import csv
import hashlib
import json
import math
from pathlib import Path
import sys

if len(sys.argv) != 5:
  raise SystemExit('usage: check_reference.py MOMprecise.dat result.jsonl panel_iv_design.csv reference-directory')
source, output, design, reference = map(Path, sys.argv[1:])
assert hashlib.sha256(source.read_bytes()).hexdigest() == '332c875eaa00ecc08a16c75424e4c56aa3fcf13c6ee5049768ed3d08a2bfef9c'

def read(path):
  with path.open(newline='') as stream:
    return list(csv.DictReader(stream))


def near(actual, expected, label):
  a, b = float(actual), float(expected)
  assert math.isfinite(a) and math.isfinite(b) and abs(a-b) <= 1e-9 + 1e-9*abs(b), (label, a, b)

actual, expected = read(design), read(reference/'design.csv')
assert len(actual) == len(expected) == 4256
assert set(actual[0]) == set(expected[0])
for a, e in zip(actual, expected):
  for key in e:
    near(a[key], e[key], f"row {e['id']}/{e['year']} {key}")
leaves = [json.loads(line) for line in output.read_text().splitlines()]
fits = [v['data'] for v in leaves if v['result_type'] == 'RegressionResult']
assert len(fits) == 2
coefficients = read(reference/'coefficients.csv')
for fit, mode in zip(fits, ('classical', 'cluster')):
  assert fit['observation_count'] == 4256 and fit['sample']['dropped_observations'] == 0
  assert not fit['include_intercept'] and len(fit['coefficients']) == 5
  for row, ref in zip(fit['coefficients'], coefficients):
    assert row['name'] == ref['name']
    near(row['value'], ref['beta'], f"{mode}.{row['name']}.beta")
    near(row['standard_error'], ref[mode], f"{mode}.{row['name']}.se")
    near(row['p_value'], ref[mode+'_p'], f"{mode}.{row['name']}.p")
  matrix = read(reference/f'{mode}.csv')
  for i, ref in enumerate(matrix):
    for j, key in enumerate(ref):
      near(fit['covariance_matrix'][i][j], ref[key], f'{mode}.cov[{i},{j}]')
# Table 22.2 displays three decimal places; the old Stata log uses float storage.
author = [
  (0.2091087, 0.3886332, 0.3741705),
  (-0.0296864, 0.0437001, 0.0293678),
  (0.0263880, 0.0289908, 0.0153921),
  (-0.0003411, 0.0003688, 0.0001837),
  (0.0004020, 0.0429076, 0.0667719),
]
for i, (beta, classical, cluster) in enumerate(author):
  assert abs(fits[0]['coefficients'][i]['value'] - beta) < 0.0005
  assert abs(fits[0]['coefficients'][i]['standard_error'] - classical) < 0.0005
  assert abs(fits[1]['coefficients'][i]['standard_error'] - cluster) < 0.0005
assert len(fits[0]['first_stage']['rows']) == 5
print('PASS: source sample, 4,256 designs and predictions, both fits and covariance matrices')
