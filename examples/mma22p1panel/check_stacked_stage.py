#!/usr/bin/env python3
"""Audit the source first-stage design and both joint F tests against R."""
import csv
from decimal import Decimal, localcontext
import hashlib
import json
import math
from pathlib import Path
import sys

if len(sys.argv) != 5:
  raise SystemExit('usage: check_stacked_stage.py MOMprecise.dat result.jsonl stacked_stage_design.csv reference-directory')
source, output, design, reference = map(Path, sys.argv[1:])
assert hashlib.sha256(source.read_bytes()).hexdigest() == '332c875eaa00ecc08a16c75424e4c56aa3fcf13c6ee5049768ed3d08a2bfef9c'

def read(path):
  with path.open(newline='') as stream:
    return list(csv.DictReader(stream))

def near(actual, expected, label):
  a, b = float(actual), float(expected)
  assert math.isfinite(a) and math.isfinite(b) and abs(a-b) <= 1e-9 + 1e-9*abs(b), (label, a, b)

def precise_wald(coefficients, covariance):
  # Independent high-precision elimination checks the statistic for each supplied
  # covariance, even when the restrictions are ill-conditioned in double precision.
  with localcontext() as context:
    context.prec = 60
    b = [Decimal(str(x)) for x in coefficients]
    n = len(b)
    a = [[Decimal(str(x)) for x in row] + [b[i]] for i, row in enumerate(covariance)]
    for k in range(n):
      p = max(range(k, n), key=lambda i: abs(a[i][k]))
      a[k], a[p] = a[p], a[k]
      for i in range(k+1, n):
        f = a[i][k] / a[k][k]
        for j in range(k+1, n+1):
          a[i][j] -= f*a[k][j]
    solution = [Decimal(0)]*n
    for i in reversed(range(n)):
      solution[i] = (a[i][n] - sum(a[i][j]*solution[j] for j in range(i+1, n))) / a[i][i]
    return float(sum(x*y for x, y in zip(b, solution))/n)

actual, expected = read(design), read(reference/'design.csv')
assert len(actual) == len(expected) == 4256 and set(actual[0]) == set(expected[0])
for a, e in zip(actual, expected):
  for key in e:
    near(a[key], e[key], f"row {e['id']}/{e['year']} {key}")
leaves = [json.loads(line) for line in output.read_text().splitlines()]
fits = [v['data'] for v in leaves if v['result_type'] == 'RegressionResult']
tests = [v['data'] for v in leaves if v['result_type'] == 'TestResult']
coefficients, reference_tests = read(reference/'coefficients.csv'), read(reference/'tests.csv')
assert len(fits) == len(tests) == len(reference_tests) == 2 and len(coefficients) == 73
for fit, test, ref_test, mode, covariance in zip(fits, tests, reference_tests, ('cluster','classical'), ('cluster(id)','nonrobust')):
  assert ref_test['mode'] == mode and fit['covariance'] == covariance
  assert fit['observation_count'] == 4256 and fit['sample']['dropped_observations'] == 0
  assert fit['include_intercept'] and len(fit['coefficients']) == 73
  assert fit['covariance_parameter_names'] == [r['name'] for r in coefficients]
  near(fit['r_squared'], ref_test['r_squared'], mode+'.r2')
  for row, ref in zip(fit['coefficients'], coefficients):
    assert row['name'] == ref['name']
    for field, key in (('value','beta'),('standard_error',mode),('p_value',mode+'_p')):
      near(row[field], ref[key], f"{mode}.{row['name']}.{field}")
  matrix = read(reference/f'{mode}.csv')
  assert len(matrix) == 73 and all(len(r) == 73 for r in matrix)
  assert len(fit['covariance_matrix']) == 73 and all(len(r) == 73 for r in fit['covariance_matrix'])
  for i, row in enumerate(matrix):
    for j, key in enumerate(row):
      near(fit['covariance_matrix'][i][j],row[key],f'{mode}.cov[{i},{j}]')
  assert test['is_chi2'] is False and test['df'] == int(ref_test['df']) and test['df_residual'] == int(ref_test['df_residual'])
  assert test['constraints'] == [r['name']+' = 0' for r in coefficients[1:]]
  own_f = precise_wald([row['value'] for row in fit['coefficients']][1:],
                       [row[1:] for row in fit['covariance_matrix'][1:]])
  near(test['statistic'], own_f, mode+'.own-covariance.F')
  if mode == 'cluster':
    # The source's restriction covariance has correlation condition ~2.75e7.
    # Covariance rounding/accumulation differences amplify in this joint test.
    assert abs(test['statistic']-float(ref_test['statistic'])) < 2e-8
    assert max(abs(float(v)-float(r)) for row, ref in zip(fit['covariance_matrix'], matrix)
               for v, r in zip(row, ref.values())) < 2e-11
    native_beta = [row['value'] for row in fit['coefficients']][1:]
    reference_beta = [row['beta'] for row in coefficients][1:]
    native_cov = [row[1:] for row in fit['covariance_matrix'][1:]]
    reference_cov = [list(row.values())[1:] for row in matrix[1:]]
    near(precise_wald(native_beta, reference_cov), ref_test['statistic'], mode+'.reference-covariance.F')
    near(precise_wald(reference_beta, native_cov), own_f, mode+'.native-covariance.F')
  else:
    near(test['statistic'],ref_test['statistic'],mode+'.F')
  near(test['p_value'],ref_test['p_value'],mode+'.F.p')
print('PASS: original 4,256 stacked first-stage designs and predictions, fits, covariance and F tests')
