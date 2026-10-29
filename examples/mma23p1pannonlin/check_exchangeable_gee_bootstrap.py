#!/usr/bin/env python3
"""Audit the prescribed GEE pairs draws and refits against Python and R."""
import csv
import hashlib
import itertools
import json
import math
from pathlib import Path
import sys

if len(sys.argv) not in (5, 6):
  raise SystemExit('usage: check_exchangeable_gee_bootstrap.py work-dir oracle.csv reference-dir result.jsonl [original-source.asc]')
work, oracle, refdir, output = map(Path, sys.argv[1:5])
if len(sys.argv) == 6 and hashlib.sha256(Path(sys.argv[5]).read_bytes()).hexdigest() != '87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8':
  raise AssertionError('original-source SHA-256 mismatch')
with (refdir / 'reference.csv').open(newline='') as f:
  reference = {row['field']: float(row['value']) for row in csv.DictReader(f)}
with (refdir / 'fits.csv').open(newline='') as f:
  fits = {int(row['replicate']): row for row in csv.DictReader(f)}
with (refdir / 'predictions.csv').open(newline='') as f:
  predictions = {(int(row['id']), int(row['year'])): row for row in csv.DictReader(f)}

def close(actual, expected, label):
  actual, expected = float(actual), float(expected)
  tol = 1e-9 + 1e-10 * abs(expected)
  if not math.isfinite(actual) or abs(actual - expected) > tol:
    raise AssertionError(f'{label}: {actual} != {expected} (tolerance {tol})')

with (work / 'patents.csv').open(newline='') as f:
  source = list(csv.DictReader(f))
if len(source) != int(reference['candidate_rows']) or len(predictions) != len(source):
  raise AssertionError('source row count mismatch')
firms = sorted({int(row['id']) for row in source})
if len(firms) != int(reference['firms_per_draw']):
  raise AssertionError('firm count mismatch')
for row in source:
  key = (int(row['id']), int(row['year']))
  p = predictions.pop(key)
  for name in ('id', 'year', 'logr', 'pat'):
    if float(row[name]) != float(p[name]):
      raise AssertionError(f'source/design mismatch {key} {name}')
  mu = math.exp(reference['estimate_intercept'] + reference['estimate_slope'] * float(row['logr']))
  close(mu, p['gee_mu'], f'original mean {key}')
  close(float(row['pat']) - mu, p['gee_residual'], f'original residual {key}')
if predictions:
  raise AssertionError('unmatched reference predictions')
checked = 0
with oracle.open(newline='') as f, (work / 'exchangeable_gee_bootstrap_draws.csv').open(newline='') as g:
  for expected, row in itertools.zip_longest(csv.DictReader(f), csv.DictReader(g)):
    if expected is None or row is None:
      raise AssertionError('draw length mismatch')
    rep, position = int(expected['replicate']), int(expected['position'])
    for key in ('replicate', 'position', 'step'):
      if int(row[key]) != int(expected[key]):
        raise AssertionError(f'draw {rep}:{position} {key} mismatch')
    if int(row['firm_id']) != firms[int(expected['index0'])]:
      raise AssertionError(f'draw {rep}:{position} firm mismatch')
    for actual, expected_key in [('replicate_intercept', 'intercept'), ('replicate_slope', 'slope'), ('replicate_rho', 'rho')]:
      close(row[actual], fits[rep][expected_key], f'replicate {rep} {expected_key}')
    for actual, expected_key in [('estimate_a', 'estimate_intercept'), ('estimate_b', 'estimate_slope'), ('estimate_rho', 'estimate_rho')]:
      close(row[actual], reference[expected_key], f'original {actual}')
    for key in ('candidate_rows', 'firms_per_draw', 'replications', 'original_mu_mean', 'original_mu_min', 'original_mu_max', 'original_residual_mean'):
      close(row[key], reference[key], key)
    if not 1 <= int(row['replicate_iterations']) <= 100 or not 0 <= float(row['replicate_last_step']) <= 1e-12:
      raise AssertionError(f'replicate {rep} convergence metadata')
    checked += 1
if checked != int(reference['firms_per_draw'] * reference['replications']) or set(fits) != set(range(1, 501)):
  raise AssertionError('wrong draw/refit count')
report = None
with output.open() as f:
  for line in f:
    event = json.loads(line)
    if event.get('result_type') == 'TableResult' and 'covariance_aa' in event['data']['headers']:
      data = event['data']
      if len(data['rows']) != 1:
        raise AssertionError('expected one report row')
      report = dict(zip(data['headers'], data['rows'][0]))
if report is None:
  raise AssertionError('bootstrap report missing')
for key, value in reference.items():
  close(report[key], value, 'report ' + key)
assert report['base_seed'] == 10001
assert report['method'] == 'whole_firm_pairs_distinct_copies'
assert report['interval_method'] == 'type7_percentile_95'
assert report['rng'] == 'xoshiro256++/splitmix64/per_replicate_seed_plus_index/u64_rejection_v1'
print(f'PASS: {len(source)} source/design and reconstructed mean/residual rows, {checked} exact ordered draws, 500 independent R refits, {len(reference)} report fields')
