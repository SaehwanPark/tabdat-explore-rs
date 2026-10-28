#!/usr/bin/env python3
"""Audit the original-source OLS workflow against independent R and author facts."""
import csv
import hashlib
import json
import math
from pathlib import Path
import sys

if len(sys.argv) != 5:
  raise SystemExit('usage: check_ols.py MOMprecise.dat result.jsonl fd_ols_design.csv reference-directory')
source, output, design, reference = map(Path, sys.argv[1:])
assert hashlib.sha256(source.read_bytes()).hexdigest() == '332c875eaa00ecc08a16c75424e4c56aa3fcf13c6ee5049768ed3d08a2bfef9c'

def read(path):
  with path.open(newline='') as stream:
    return list(csv.DictReader(stream))

def near(actual, expected, label):
  a, b = float(actual), float(expected)
  assert math.isfinite(a) and math.isfinite(b) and abs(a-b) <= 1e-9 + 1e-9*abs(b), (label, a, b)

actual, expected = read(design), read(reference/'design.csv')
assert len(actual) == len(expected) == 4256 and set(actual[0]) == set(expected[0])
for a, e in zip(actual, expected):
  for key in e:
    near(a[key], e[key], f"row {e['id']}/{e['year']} {key}")
leaves = [json.loads(line) for line in output.read_text().splitlines()]
fits = [v['data'] for v in leaves if v['result_type'] == 'RegressionResult']
assert len(fits) == 3
coefficients = read(reference/'coefficients.csv')
assert len(coefficients) == 5
author = [
  (0.1115114, 0.0230566, 0.0791674, 0.0960926),
  (-0.0062887, 0.0116719, 0.0110570, 0.0109558),
  (0.0066935, 0.0212744, 0.0243788, 0.0123390),
  (-0.0000797, 0.0002644, 0.0003147, 0.0001551),
  (-0.0352603, 0.0199796, 0.0364021, 0.0452557),
]
for m, (fit, mode) in enumerate(zip(fits, ('classical','hc1','cluster'))):
  assert fit['observation_count'] == 4256 and fit['sample']['dropped_observations'] == 0
  assert not fit['include_intercept'] and len(fit['coefficients']) == 5
  assert fit['covariance'] == ('nonrobust', 'robust', 'cluster(id)')[m]
  assert fit['covariance_parameter_names'] == [row['name'] for row in coefficients]
  for i, (row, ref) in enumerate(zip(fit['coefficients'], coefficients)):
    assert row['name'] == ref['name']
    near(row['value'], ref['beta'], f"{mode}.{row['name']}.beta")
    near(row['standard_error'], ref[mode], f"{mode}.{row['name']}.se")
    near(row['p_value'], ref[mode+'_p'], f"{mode}.{row['name']}.p")
    assert abs(row['value']-author[i][0]) < 0.0005
    assert abs(row['standard_error']-author[i][m+1]) < 0.0005
  matrix = read(reference/f'{mode}.csv')
  assert len(matrix) == 5 and all(len(row) == 5 for row in matrix)
  assert len(fit['covariance_matrix']) == 5 and all(len(row) == 5 for row in fit['covariance_matrix'])
  for i, ref in enumerate(matrix):
    for j, key in enumerate(ref):
      near(fit['covariance_matrix'][i][j], ref[key], f'{mode}.cov[{i},{j}]')
print('PASS: 4,256 source designs/predictions, three OLS fits and full covariances')
