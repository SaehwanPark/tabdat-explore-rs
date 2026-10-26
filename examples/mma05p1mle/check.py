#!/usr/bin/env python3
"""Audit source, four fits and all source-row projections against base R."""
import csv
import hashlib
import json
import math
from pathlib import Path
import sys

if len(sys.argv) != 5:
  raise SystemExit('usage: check.py original.asc result.jsonl comparison_predictions.csv reference-directory')
source, output, projections, reference = map(Path, sys.argv[1:])
assert hashlib.sha256(source.read_bytes()).hexdigest() == '460f5d731f1848c8695d78a1fbf0db348d0daced4b8b5b0c2734bffa20b3e31c'
raw = [line.split() for line in source.read_text().splitlines()]
with (output.parent / 'nls.csv').open(newline='') as stream:
  prepared = list(csv.DictReader(stream))
assert len(prepared) == len(raw) == 10000
for i, (row, fields) in enumerate(zip(prepared, raw), 1):
  assert int(row['source_row']) == i and len(fields) == 2
  assert float(row['y']) == float(fields[0]) and float(row['x']) == float(fields[1])
with (reference / 'reference.csv').open(newline='') as stream:
  ref = {row['key']: float(row['value']) for row in csv.DictReader(stream)}
with projections.open(newline='') as stream:
  native = list(csv.DictReader(stream))
with (reference / 'projections.csv').open(newline='') as stream:
  expected = list(csv.DictReader(stream))
assert len(native) == len(expected) == 10000

def near(actual, target, key):
  assert math.isfinite(actual) and math.isfinite(target), key
  if key.endswith('.p') and 0 < abs(target) < 1e-20:
    assert abs(actual-target) <= abs(target)*1e-8, (key, actual, target)
  else:
    assert abs(actual-target) <= 2e-11*(1+abs(target)), (key, actual, target)

for i, (got, want) in enumerate(zip(native, expected), 1):
  assert got.keys() == want.keys() and int(got['source_row']) == int(want['source_row']) == i
  for key in got.keys() - {'source_row'}:
    near(float(got[key]),float(want[key]),f'row{i}.{key}')
results = [json.loads(line) for line in output.read_text().splitlines()]
fits = [item for item in results if item['result_type'] in ('RegressionResult','PoissonResult')]
assert [f['result_type'] for f in fits] == ['RegressionResult','RegressionResult','PoissonResult','PoissonResult']
for item, name, covariance in zip(fits, ('ols_classical','ols_robust','exp_classical','exp_robust'),
    ('nonrobust','robust','model_based_poisson_information','robust_sandwich_n_over_n_minus_1')):
  fit = item['data']; assert fit['covariance'] == covariance
  assert fit['observation_count'] == 10000 and fit['sample']['dropped_observations'] == 0
  assert fit['covariance_parameter_names'] == ['intercept','x']
  if item['result_type'] == 'PoissonResult':
    assert fit['sample']['retained_indices'] == list(range(10000))
  assert len(fit['coefficients']) == len(fit['covariance_matrix']) == 2
  for i, coefficient in enumerate(fit['coefficients'],1):
    assert coefficient['name'] == ('intercept','x')[i-1]
    for result_key, reference_key in (('value','value'),('standard_error','se'),('statistic','stat'),('p_value','p')):
      key = f'{name}.{i}.{reference_key}'; near(coefficient[result_key],ref[key],key)
    if name.startswith('exp_'):
      for result_key, reference_key in (('confidence_lower','lower'),('confidence_upper','upper')):
        key = f'{name}.{i}.{reference_key}'; near(coefficient[result_key],ref[key],key)
    assert len(fit['covariance_matrix'][i-1]) == 2
    for j, value in enumerate(fit['covariance_matrix'][i-1],1):
      key=f'{name}.cov.{i}.{j}'; near(value,ref[key],key)
  near(fit['log_likelihood'],ref['ols_log_likelihood' if name.startswith('ols_') else 'poisson_log_likelihood'],name+'.ll')
summary, = [v['data'] for v in results if v['result_type']=='TableResult' and len(v['data']['headers'])==7]
row = summary['rows'][0]
assert row[0]==10000 and row[4]==row[5]==0
for index, key in ((1,'density_log_likelihood'),(2,'exponential_r_squared'),(3,'ols_r_squared')):
  near(row[index],ref[key],key)
# Four-decimal coefficients and SEs from the author's compact table.
for key, target in {'ols_classical.1.value':-.0093,'ols_classical.2.value':.6198,
    'ols_classical.1.se':.0161,'ols_classical.2.se':.0113,
    'ols_robust.1.se':.0172,'ols_robust.2.se':.0254,
    'exp_classical.1.value':1.9829,'exp_classical.2.value':-.9896,
    'exp_classical.1.se':.0141,'exp_classical.2.se':.0099,
    'exp_robust.1.se':.0144,'exp_robust.2.se':.0099}.items():
  assert abs(ref[key]-target)<0.00005, (key,ref[key],target)
print('validated 10000 ordered source rows, 100000 projections, four fits and both likelihood conventions')
