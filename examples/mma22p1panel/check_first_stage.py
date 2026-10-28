#!/usr/bin/env python3
"""Audit the source first-stage design and both joint F tests against R."""
import csv
import hashlib
import json
import math
from pathlib import Path
import sys

if len(sys.argv) != 5:
  raise SystemExit('usage: check_first_stage.py MOMprecise.dat result.jsonl first_stage_design.csv reference-directory')
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
tests = [v['data'] for v in leaves if v['result_type'] == 'TestResult']
coefficients, reference_tests = read(reference/'coefficients.csv'), read(reference/'tests.csv')
assert len(fits) == len(tests) == len(reference_tests) == 2 and len(coefficients) == 10
for fit, test, ref_test, mode, covariance in zip(fits, tests, reference_tests, ('cluster','classical'), ('cluster(id)','nonrobust')):
  assert ref_test['mode'] == mode and fit['covariance'] == covariance
  assert fit['observation_count'] == 4256 and fit['sample']['dropped_observations'] == 0
  assert fit['include_intercept'] and len(fit['coefficients']) == 10
  assert fit['covariance_parameter_names'] == [r['name'] for r in coefficients]
  near(fit['r_squared'], ref_test['r_squared'], mode+'.r2')
  for row, ref in zip(fit['coefficients'], coefficients):
    assert row['name'] == ref['name']
    for field, key in (('value','beta'),('standard_error',mode),('p_value',mode+'_p')):
      near(row[field], ref[key], f"{mode}.{row['name']}.{field}")
  matrix = read(reference/f'{mode}.csv')
  assert len(matrix) == 10 and all(len(r) == 10 for r in matrix)
  assert len(fit['covariance_matrix']) == 10 and all(len(r) == 10 for r in fit['covariance_matrix'])
  for i, row in enumerate(matrix):
    for j, key in enumerate(row):
      near(fit['covariance_matrix'][i][j],row[key],f'{mode}.cov[{i},{j}]')
  assert test['is_chi2'] is False and test['df'] == int(ref_test['df']) and test['df_residual'] == int(ref_test['df_residual'])
  assert test['constraints'] == [r['name']+' = 0' for r in coefficients[1:]]
  near(test['statistic'],ref_test['statistic'],mode+'.F')
  near(test['p_value'],ref_test['p_value'],mode+'.F.p')
# Author log uses Stata float storage; compare to its seven-digit F and R2 at 1e-5.
for fit, test, expected_f in zip(fits, tests, (2.3790046,2.800243)):
  assert abs(fit['r_squared']-0.00590049) < 1e-5
  assert abs(test['statistic']-expected_f) < 1e-5
print('PASS: original 4,256 first-stage designs and predictions, fits, covariance and F tests')
