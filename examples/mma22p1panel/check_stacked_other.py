#!/usr/bin/env python3
"""Audit four source-design stacked relevance tests against independent R."""
import csv
from decimal import Decimal, localcontext
import hashlib
import json
import math
from pathlib import Path
import subprocess
import sys

if len(sys.argv) != 5:
  raise SystemExit('usage: check_stacked_other.py MOMprecise.dat result.jsonl stacked_other_stages.csv reference-directory')
source, output, design, reference = map(Path, sys.argv[1:])
assert hashlib.sha256(source.read_bytes()).hexdigest() == '332c875eaa00ecc08a16c75424e4c56aa3fcf13c6ee5049768ed3d08a2bfef9c'

def read(path):
  with path.open(newline='') as stream:
    return list(csv.DictReader(stream))

def near(actual, expected, label):
  a, b = float(actual), float(expected)
  assert math.isfinite(a) and math.isfinite(b) and abs(a-b) <= 1e-9 + 1e-9*abs(b), (label, a, b)

def precise_wald(coefficients, covariance):
  with localcontext() as context:
    context.prec = 60
    b = [Decimal(str(x)) for x in coefficients]; n = len(b)
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
refs = read(reference/'tests.csv')
assert len(fits) == len(tests) == len(refs) == 4
# Source-specific observed bounds; covariance cross-substitution diagnoses amplified F differences.
cases = [('dkids', 3e-8, 5e-11), ('dageh', 1e-8, 5e-11),
         ('dagesq', 3e-6, 4e-7), ('ddisab', 2e-7, 1e-11)]
for fit, test, ref, (name, f_bound, cov_bound) in zip(fits, tests, refs, cases):
  assert ref['name'] == name and fit['outcome'] == name and fit['covariance'] == 'cluster(id)'
  assert fit['observation_count'] == 4256 and fit['sample']['dropped_observations'] == 0
  assert fit['include_intercept'] and len(fit['coefficients']) == 73
  coeff = read(reference/f'{name}-coefficients.csv')
  assert len(coeff) == 73 and fit['covariance_parameter_names'] == [r['name'] for r in coeff]
  near(fit['r_squared'],ref['r_squared'],name+'.r2')
  for row, r in zip(fit['coefficients'], coeff):
    assert row['name'] == r['name']
    for field, key in (('value','beta'),('standard_error','se'),('p_value','p_value')):
      near(row[field],r[key],f"{name}.{row['name']}.{field}")
  matrix = read(reference/f'{name}-covariance.csv')
  assert len(matrix) == 73 and all(len(r)==73 for r in matrix)
  native = fit['covariance_matrix']
  assert len(native)==73 and all(len(r)==73 for r in native)
  rv = [[float(value) for value in row.values()] for row in matrix]
  assert all(rv[i][i] > 0 for i in range(73))
  # Scale entry errors by the corresponding standard deviations, including
  # near-zero covariances and the substantially larger age-squared response.
  for i in range(73):
    for j in range(73):
      assert math.isfinite(native[i][j])
      assert abs(native[i][j]-rv[i][j]) <= 3e-9*math.sqrt(rv[i][i]*rv[j][j])
  assert max(abs(native[i][j]-rv[i][j]) for i in range(73) for j in range(73)) < cov_bound
  assert test['is_chi2'] is False and test['df']==72 and test['df_residual']==531
  assert test['constraints']==[r['name']+' = 0' for r in coeff[1:]]
  b = [row['value'] for row in fit['coefficients']][1:]; br = [r['beta'] for r in coeff][1:]
  v = [row[1:] for row in native[1:]]; vr = [list(row.values())[1:] for row in matrix[1:]]
  own = precise_wald(b,v)
  near(test['statistic'],own,name+'.native-covariance.F')
  near(precise_wald(br,v),own,name+'.reference-beta.F')
  near(precise_wald(b,vr),ref['statistic'],name+'.reference-covariance.F')
  assert abs(test['statistic']-float(ref['statistic'])) < f_bound
# Evaluate the F survival function at native statistics, independently of the small F differences.
fs = [float(t['statistic']) for t in tests]
assert all(math.isfinite(f) and f >= 0 for f in fs)
expr = 'cat(sprintf("%.17g\\n",pf(c('+','.join(map(repr,fs))+'),72,531,lower.tail=FALSE)))'
pvalues = subprocess.check_output(['Rscript','-e',expr],text=True).splitlines()
assert len(pvalues)==4
for test, p in zip(tests,pvalues):
  actual, expected = test['p_value'], float(p)
  assert expected > 0 and abs(actual-expected) <= 1e-10*expected + 1e-320, (actual,expected)
print('PASS: original stacked source design, four clustered fits and F tests')
