#!/usr/bin/env python3
"""Audit normal-RE source/design, joint inference and population predictions."""
import csv
import hashlib
import json
import math
from pathlib import Path
import sys

if len(sys.argv) not in (4, 5):
  raise SystemExit('usage: check_normal_re_poisson.py work-dir reference-dir result.jsonl [original-source.asc]')
work, reference_dir, output = map(Path, sys.argv[1:4])
if len(sys.argv) == 5 and hashlib.sha256(Path(sys.argv[4]).read_bytes()).hexdigest() != '87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8':
  raise AssertionError('original-source SHA-256 mismatch')
with (reference_dir / 'reference.csv').open(newline='') as f:
  reference = {row['field']: float(row['value']) for row in csv.DictReader(f)}
with (reference_dir / 'predictions.csv').open(newline='') as f:
  predictions = {(int(row['id']), int(row['year'])): row for row in csv.DictReader(f)}
with (work / 'patents.csv').open(newline='') as f:
  source = {(int(row['id']), int(row['year'])): row for row in csv.DictReader(f)}

def close(actual, expected, label):
  actual, expected = float(actual), float(expected)
  tol = 1e-9 + 1e-10 * abs(expected)
  if not math.isfinite(actual) or abs(actual - expected) > tol:
    raise AssertionError(f'{label}: {actual} != {expected} (tolerance {tol})')

report = None
with output.open() as f:
  for line in f:
    event = json.loads(line)
    if event.get('result_type') == 'TableResult' and 'covariance_tt' in event['data']['headers']:
      data = event['data']
      if len(data['rows']) != 1:
        raise AssertionError('expected one report row')
      report = dict(zip(data['headers'], data['rows'][0]))
if report is None:
  raise AssertionError('normal RE report missing')
for key, value in reference.items():
  if key.startswith('p_') and value > 0:
    actual = float(report[key])
    if not math.isfinite(actual) or abs(actual - value) > 1e-9 * value:
      raise AssertionError(f'report {key}: {actual} != {value} (relative tolerance 1e-9)')
  else:
    close(report[key], value, 'report ' + key)
assert report['inference'] == 'normal_asymptotic'
assert report['lr_reference'] == 'chi_bar_square_0_1_equal_mixture'
if not 1 <= int(report['iterations']) <= 100 or not 0 <= float(report['last_step']) <= 1e-12 or not 0 <= float(report['quad_diff']) <= 1e-9:
  raise AssertionError('invalid convergence or quadrature metadata')
count = 0
previous = None
with (work / 'normal_re_poisson_predictions.csv').open(newline='') as f:
  for row in csv.DictReader(f):
    key = (int(row['id']), int(row['year']))
    if previous is not None and key <= previous:
      raise AssertionError('prediction rows not strictly ordered')
    previous = key
    actual_source = source.pop(key)
    expected = predictions.pop(key)
    for name, value in actual_source.items():
      if value == '':
        if row[name] != '':
          raise AssertionError(f'changed missing source value {key} {name}')
      elif float(row[name]) != float(value):
        raise AssertionError(f'changed source field {key} {name}')
    for name in ('id', 'year', 'logr', 'pat'):
      if float(row[name]) != float(expected[name]):
        raise AssertionError(f'source/reference mismatch {key} {name}')
    for name in ('marginal_mu', 'marginal_residual'):
      close(row[name], expected[name], f'{name} {key}')
    for name in ('estimate_intercept', 'estimate_slope', 'estimate_lnvariance', 'sigma', 'covariance_aa', 'covariance_ab', 'covariance_at', 'covariance_bb', 'covariance_bt', 'covariance_tt', 'log_likelihood', 'poisson_log_likelihood', 'lr_variance_zero', 'candidate_rows', 'firms', 'zero_total_firms'):
      close(row[name], reference[name], f'{name} {key}')
    if int(row['iterations']) != report['iterations'] or float(row['last_step']) != report['last_step'] or float(row['quad_diff']) != report['quad_diff']:
      raise AssertionError(f'inconsistent diagnostics {key}')
    count += 1
if source or predictions or count != int(reference['candidate_rows']):
  raise AssertionError('missing or extra source/prediction rows')
print(f'PASS: {count} exact source/design and R prediction rows, {len(reference)} independent R report quantities, full joint covariance')
