#!/usr/bin/env python3
"""Audit the original-source clustered GMM workflow against independent R."""
import csv
import hashlib
import json
import math
from pathlib import Path
import sys

if len(sys.argv) != 5:
  raise SystemExit('usage: check_gmm.py MOMprecise.dat result.jsonl panel_gmm_design.csv reference-directory')
source, output, design, reference = map(Path, sys.argv[1:])
assert hashlib.sha256(source.read_bytes()).hexdigest() == '332c875eaa00ecc08a16c75424e4c56aa3fcf13c6ee5049768ed3d08a2bfef9c'

def read(path):
  with path.open(newline='') as stream:
    return list(csv.DictReader(stream))

def near(actual, expected, label):
  a, b = float(actual), float(expected)
  assert math.isfinite(a) and math.isfinite(b) and abs(a-b) <= 1e-9 + 1e-9*abs(b), (label,a,b)

actual, expected = read(design), read(reference/'design.csv')
assert len(actual) == len(expected) == 4256 and set(actual[0]) == set(expected[0])
for a,e in zip(actual,expected):
  for key in e:
    near(a[key],e[key],f"row {e['id']}/{e['year']} {key}")
leaves = [json.loads(line) for line in output.read_text().splitlines()]
fits = [v['data'] for v in leaves if v['result_type']=='ClusterGmmResult']
assert len(fits)==1
fit=fits[0]
assert fit['estimator']=='cluster_two_step_gmm' and fit['outcome']=='dlnhr'
assert fit['cluster_variable']=='id' and fit['cluster_count']==532
assert fit['observation_count']==4256 and not fit['include_intercept']
assert fit['sample']['total_observations']==4256 and fit['sample']['dropped_observations']==0
assert fit['sample']['retained_indices']==list(range(4256))
assert fit['sample']['cluster_groups']==[str(int(float(row['id']))) for row in expected]
assert fit['reference_distribution']=='asymptotic_standard_normal' and fit['covariance_correction']==1
refs=read(reference/'coefficients.csv'); assert len(refs)==len(fit['coefficients'])==5
assert fit['predictors']==fit['covariance_parameter_names']==[r['name'] for r in refs]
for a,r in zip(fit['coefficients'],refs):
  assert a['name']==r['name']
  for field,key in [('value','beta'),('standard_error','fixed_se'),('author_updated_standard_error','updated_se'),('p_value','p_value'),('confidence_lower','confidence_lower'),('confidence_upper','confidence_upper')]:
    near(a[field],r[key],a['name']+'.'+field)
for mode,key in [('fixed','fixed_weight_sandwich_covariance'),('updated','author_updated_inverse_information_covariance')]:
  matrix=read(reference/f'{mode}_covariance.csv'); native=fit[key]
  assert len(matrix)==len(native)==5 and all(len(r)==5 for r in matrix) and all(len(r)==5 for r in native)
  for i,row in enumerate(matrix):
    for j,column in enumerate(row): near(native[i][j],row[column],f'{mode}[{i},{j}]')
diag=read(reference/'diagnostics.csv'); assert len(diag)==1; diag=diag[0]
assert fit['overidentification_df']==4
for field,key in [('fixed_weight_j','fixed_j'),('author_updated_j','updated_j'),('fixed_weight_j_p_value','fixed_p'),('author_updated_j_p_value','updated_p')]:
  near(fit[field],diag[key],field)
# Selected author-log values reflect Stata float storage; no digit-level parity claimed.
wage=fit['coefficients'][0]
assert wage['name']=='dlnwg'
assert abs(wage['value']-0.54679602)<0.000001
assert abs(wage['author_updated_standard_error']-0.32762396)<0.000001
assert abs(fit['author_updated_j']-5.4503878)<0.00001
print('PASS: original 4,256 source designs and predictions, both GMM covariance and objective conventions')
