#!/usr/bin/env python3
"""Development-only independent raw-source/sample, inference and prediction audit."""
import csv
import hashlib
import json
import math
from pathlib import Path
import shlex
import sys

native, reference, raw = map(Path, sys.argv[1:])
assert hashlib.sha256(raw.read_bytes()).hexdigest() == 'eb1b422fa29a096e608ff5f8912ee1b9fcecc749d227fbfd37b19d3b18db8b0b'
def records(path):
  with path.open(newline='') as f:
    return list(csv.DictReader(f))
def close(a, e, label):
  a, e = float(a), float(e)
  # Fixed absolute error for coefficients/covariance/prediction; normal tails relative.
  tol = abs(e)*1e-7+1e-300 if str(label).endswith('p_value') else 1e-8
  assert math.isfinite(a) and math.isfinite(e) and abs(a-e) <= tol, (label, a, e)
loaded = records(native/'qreg.csv'); tokens = shlex.split(raw.read_text())
assert len(loaded) == 5999 and len(tokens) == 5999*9
columns = ['sex','age','educyr98','farm','urban98','hhsize','lhhexp1','lhhex12m','lnrlfood']
for i, row in enumerate(loaded):
  assert int(row['source_row']) == i+1
  for j, k in enumerate(columns):
    token = tokens[9*i+j]
    if token == '.': assert row[k] == ''
    elif k in ['farm','urban98']: assert row[k] == token
    else: assert float(row[k]) == float(token)
events = [json.loads(l) for l in (native/'quantiles.jsonl').read_text().splitlines()]
assert all('error' not in e and e['schema_version'] == 1 for e in events)
fits = [e['data'] for e in events if e['result_type'] == 'QuantileRegressionResult']
assert len(fits) == 3
by_case = dict(zip(['q10','q50','q90'], fits))
scalars = records(reference/'reference.csv'); assert len(scalars) == 66
for row in scalars:
  value = by_case[row['case']]
  for part in row['path'].strip('/').split('/'):
    value = value[int(part)] if isinstance(value, list) else value[part]
  close(value, row['value'], row['path'])
for fit in fits:
  assert fit['sample'] == {'total_observations':5006,'dropped_observations':0}
  assert fit['covariance'] == 'iid_location_hall_sheather_residual_spacing'
  assert fit['reference_distribution'] == 'asymptotic_standard_normal'
  assert fit['confidence_level'] == 95
  assert fit['relative_duality_gap'] <= 1e-8 and fit['max_score_violation'] <= 1e-7
actual = records(native/'quantile_predictions.csv'); expected = records(reference/'predictions.csv')
assert len(actual) == len(expected) == 5006
kept = [r for r in loaded if r['lhhexp1'] != '' and r['lhhex12m'] != '']
assert [r['source_row'] for r in actual] == [r['source_row'] for r in kept]
assert any(float(r['lnmed']) == 0 for r in actual)  # Valid log zero must remain.
cells = 0
for a, e in zip(actual, expected):
  for k in e:
    close(a[k], e[k], k); cells += 1
assert cells == 35042
print('PASS:66 R quantreg quantities;35,042 identity/design/prediction cells;all5,999 raw source records pinned;5,006 complete cases including log zeros')
