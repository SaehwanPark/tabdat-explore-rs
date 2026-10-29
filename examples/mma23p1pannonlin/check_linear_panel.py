#!/usr/bin/env python3
"""Audit exact source/design, three linear-panel fits and all predictions."""
import csv
import hashlib
import json
import math
from pathlib import Path
import sys

if len(sys.argv) not in (4, 5):
  raise SystemExit('usage: check_linear_panel.py work-dir reference-dir result.jsonl [original-source.asc]')
work, reference_dir, output = map(Path, sys.argv[1:4])
if len(sys.argv) == 5 and hashlib.sha256(Path(sys.argv[4]).read_bytes()).hexdigest() != '87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8':
  raise AssertionError('original-source SHA-256 mismatch')
with (reference_dir / 'reference.csv').open(newline='') as f:
  references = list(csv.DictReader(f))
with (reference_dir / 'predictions.csv').open(newline='') as f:
  predictions = {(int(row['id']), int(row['year'])): row for row in csv.DictReader(f)}
with (work / 'patents.csv').open(newline='') as f:
  source = {(int(row['id']), int(row['year'])): row for row in csv.DictReader(f)}

def close(actual, expected, label):
  actual, expected = float(actual), float(expected)
  tol = 1e-9 + 1e-10 * abs(expected)
  if not math.isfinite(actual) or abs(actual - expected) > tol:
    raise AssertionError(f'{label}: {actual} != {expected} (tolerance {tol})')

with output.open() as f:
  events = list(map(json.loads, f))
def one(kind):
  values = [e['data'] for e in events if e.get('result_type') == kind]
  if len(values) != 1:
    raise AssertionError(f'expected one {kind}, got {len(values)}')
  return values[0]
def two(kind):
  values = [e['data'] for e in events if e.get('result_type') == kind]
  if len(values) != 2:
    raise AssertionError(f'expected two {kind}, got {len(values)}')
  return values
pool, fe, re = one('RegressionResult'), one('FixedEffectsResult'), one('RandomEffectsResult')
pl, fl = two('LincomResult')
pt, ft = two('TestResult')
models = {'pooled': pool, 'fe': fe, 're': re, 'pooled_lincom': pl, 'fe_lincom': fl, 'pooled_test': pt, 'fe_test': ft}
assert pool['covariance'] == 'cluster(id)' and pool['estimator'] == 'ols'
assert fe['covariance'] == 'nonrobust' and fe['common_intercept_inference'] == 'normalization_point_only'
assert re['estimator'] == 'balanced_swamy_arora_random_effects_gls' and re['covariance'] == 'nonrobust'
assert re['reference_distribution'] == 'normal' and fe['reference_distribution'] == 'student_t'
assert all(m['sample']['dropped_observations'] == 0 for m in (pool, fe, re))
for row in references:
  value = models[row['model']]
  for key in row['path'].strip('/').split('/'):
    value = value[int(key)] if isinstance(value, list) else value[key]
  expected = float(row['value'])
  if row['path'].endswith('p_value') and expected > 0:
    if not math.isfinite(float(value)) or abs(float(value) - expected) > 1e-9 * expected:
      raise AssertionError(f"{row['model']}{row['path']}: {value} != {expected} (relative tolerance 1e-9)")
  else:
    close(value, expected, row['model'] + row['path'])
count = 0
previous = None
with (work / 'linear_panel_predictions.csv').open(newline='') as f:
  for row in csv.DictReader(f):
    key = (int(row['id']), int(row['year']))
    if previous is not None and key <= previous:
      raise AssertionError('prediction rows not strictly ordered')
    previous = key
    actual_source = source.pop(key)
    expected = predictions.pop(key)
    for name, value in actual_source.items():
      if value == '':
        if row[name] != '' or expected[name] != '':
          raise AssertionError(f'changed missing source value {key} {name}')
      elif float(row[name]) != float(value) or float(expected[name]) != float(value):
        raise AssertionError(f'changed source field {key} {name}')
    for name in ('lpat', 'pooled_xb', 'pooled_residual', 'fe_xb', 'fe_xbu', 'fe_u', 'fe_residual', 're_xb', 're_residual'):
      close(row[name], expected[name], f'{name} {key}')
    count += 1
if source or predictions or count != pool['observation_count']:
  raise AssertionError('missing or extra source/prediction rows')
if len(sys.argv) == 5:
  # Compare selected author-log numbers at half of their final printed unit;
  # this is not a claim that all original output or float storage is reproduced.
  facts = [
    ('pooled slope', pool['coefficients'][1]['value'], .8340745, 5e-8),
    ('pooled slope SE', pool['coefficients'][1]['standard_error'], .0228655, 5e-8),
    ('pooled intercept', pool['coefficients'][0]['value'], .7954785, 5e-8),
    ('pooled intercept SE', pool['coefficients'][0]['standard_error'], .0579246, 5e-8),
    ('pooled F', pt['statistic'], 1330.60, .005),
    ('within slope', fe['coefficients'][0]['value'], .1067505, 5e-8),
    ('within slope SE', fe['coefficients'][0]['standard_error'], .0560364, 5e-8),
    ('within normalized intercept', fe['common_intercept'], 1.709116, 5e-7),
    ('within R squared', fe['within_r_squared'], .0026, .00005),
    ('within F', ft['statistic'], 3.63, .005),
    ('within effects F', fe['unit_effects_f_test']['f_statistic'], 16.96, .005),
    ('RE slope', re['coefficients'][1]['value'], .7202377, 5e-8),
    ('RE slope SE', re['coefficients'][1]['standard_error'], .0237986, 5e-8),
    ('RE intercept', re['coefficients'][0]['value'], .9384761, 5e-8),
    ('RE intercept SE', re['coefficients'][0]['standard_error'], .0599584, 5e-8),
    ('RE sigma_u', math.sqrt(re['sigma_u_squared']), .90057544, 5e-9),
    ('RE sigma_e', math.sqrt(re['sigma_e_squared']), .51119065, 5e-9),
    ('RE rho', re['rho'], .7563152, 5e-8),
    ('RE Wald', re['wald_statistic'], 915.90, .005),
  ]
  for name, actual, printed, half_unit in facts:
    if abs(actual - printed) > half_unit:
      raise AssertionError(f'author printed {name}: {actual} does not round to {printed}')
  print(f'PASS: {len(facts)} selected author-log printed facts')
print(f'PASS: {count} exact source/design and R prediction rows, {len(references)} independent fit/test quantities')
