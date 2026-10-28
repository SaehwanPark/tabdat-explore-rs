#!/usr/bin/env python3
"""Audit the author-program wage Shea workflow against independent R."""
import csv
import hashlib
import json
import math
from pathlib import Path
import sys

if len(sys.argv) != 5:
  raise SystemExit('usage: check_shea.py MOMprecise.dat result.jsonl shea_design.csv reference-directory')
source, output, design, reference = map(Path, sys.argv[1:])
assert hashlib.sha256(source.read_bytes()).hexdigest() == '332c875eaa00ecc08a16c75424e4c56aa3fcf13c6ee5049768ed3d08a2bfef9c'

def read(path):
  with path.open(newline='') as stream:
    return list(csv.DictReader(stream))

def near(actual, expected, label):
  a, b = float(actual), float(expected)
  assert math.isfinite(a) and math.isfinite(b) and abs(a-b) <= 1e-9 + 1e-9*abs(b), (label,a,b)

actual, expected = read(design), read(reference/'design.csv')
assert len(actual)==len(expected)==4256 and set(actual[0])==set(expected[0])
for a,e in zip(actual,expected):
  for key in e:
    near(a[key],e[key],f"row {e['id']}/{e['year']} {key}")
leaves = [json.loads(line) for line in output.read_text().splitlines()]
fits = [v['data'] for v in leaves if v['result_type']=='RegressionResult']
assert len(fits)==7
refs = read(reference/'coefficients.csv'); fit_refs = read(reference/'fit.csv')
for i,fit in enumerate(fits,1):
  assert fit['observation_count']==4256 and fit['include_intercept'] and fit['covariance']=='nonrobust'
  assert fit['sample']['total_observations']==4256 and fit['sample']['dropped_observations']==0
  expected_coefficients = [r for r in refs if int(r['fit'])==i]
  assert fit['covariance_parameter_names']==[r['name'] for r in expected_coefficients]
  assert len(fit['coefficients'])==len(expected_coefficients)
  for a,r in zip(fit['coefficients'],expected_coefficients):
    assert a['name']==r['name']
    for field,key in [('value','value'),('standard_error','se'),('p_value','p_value')]:
      near(a[field],r[key],f'fit {i} {a["name"]} {field}')
  near(fit['r_squared'],fit_refs[i-1]['r_squared'],f'fit {i} R2')
  matrix = read(reference/f'covariance_{i}.csv'); native=fit['covariance_matrix']
  assert len(matrix)==len(native)==len(expected_coefficients)
  for j,row in enumerate(matrix):
    for k,v in enumerate(row.values()): near(native[j][k],v,f'fit {i} covariance {j},{k}')
tables = [v['data'] for v in leaves if v['result_type']=='TableResult' and
          v['data']['headers']==['observations','residual_correlation','shea_partial_r_squared','interpretation']]
assert len(tables)==1 and len(tables[0]['rows'])==1
n,rho,shea,interpretation=tables[0]['rows'][0]
assert interpretation=='Descriptive relevance only; not a joint weak-IV test or evidence of instrument validity'
ref=read(reference/'diagnostic.csv')[0]
assert n==int(ref['n'])==4256 and int(ref['groups'])==532
near(rho,ref['rho'],'correlation'); near(shea,ref['shea'],'Shea R2')
# Author program's 8-decimal result; historical source variables were stored as floats.
assert abs(shea-0.00364741)<0.000000005
print('PASS: original sample/design/predictions, seven auxiliary regressions and wage Shea diagnostic')
