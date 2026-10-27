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
assert len(fits[0]['first_stage']['rows']) == 5
print('PASS: source sample, 4,256 designs and predictions, both fits and covariance matrices')
