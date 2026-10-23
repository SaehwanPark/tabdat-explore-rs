#!/usr/bin/env python3
"""Independent development audit of original source, common-draw R inference/state."""
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
  assert math.isfinite(a) and math.isfinite(e) and abs(a-e) <= 1e-8, (label, a, e)
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
events = [json.loads(l) for l in (native/'joint_bootstrap.jsonl').read_text().splitlines()]
assert all('error' not in e and e['schema_version'] == 1 for e in events)
fits = [e['data'] for e in events if e['result_type'] == 'QuantileBootstrapResult']
assert len(fits) == 1
fit = fits[0]
assert fit['observation_count'] == 5006 and fit['replications'] == 200
assert fit['quantiles'] == [i/20 for i in range(1,20)]
assert fit['seed'] == '10101' and fit['rng'] == 'xoshiro256++/splitmix64/u64-rejection-v1'
assert fit['covariance_convention'] == 'common_iid_pairs_b_minus_1'
assert fit['interval_method'] == 'percentile_r_type7' and fit['confidence_level'] == 95
assert fit['simultaneous_coverage'] is False
assert fit['sample'] == {'total_observations':5006,'dropped_observations':0}
names = [f'q{q:g}:{p}' for q in fit['quantiles'] for p in ['intercept','lntotal']]
assert fit['covariance_parameter_names'] == names
assert [r['name'] for r in fit['coefficients']] == names
scalars = records(reference/'reference.csv'); assert len(scalars) == 1672
for row in scalars:
  value = fit
  for part in row['path'].strip('/').split('/'):
    value = value[int(part)] if isinstance(value, list) else value[part]
  close(value, row['value'], row['path'])
actual = records(native/'quantile_bootstrap_cohort.csv'); expected = records(reference/'cohort.csv')
assert len(actual) == len(expected) == 5006
kept = [r for r in loaded if r['lhhexp1'] != '' and r['lhhex12m'] != '']
assert [r['source_row'] for r in actual] == [r['source_row'] for r in kept]
assert any(float(r['lnmed']) == 0 for r in actual)
cells = 0
for a, e in zip(actual, expected):
  for k in e:
    close(a[k], e[k], k); cells += 1
assert cells == 25030
print('PASS:1,672 R common-draw quantreg quantities;25,030 cohort/retained-median prediction cells;5,999 original records pinned;5,006 complete cases including log zeros')
