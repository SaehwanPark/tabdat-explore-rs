#!/usr/bin/env python3
"""Compare source, all row effects, fit sample, information and summaries."""
import csv
import hashlib
import json
import math
import sys
from pathlib import Path

if len(sys.argv) != 5:
  raise SystemExit('usage: check.py ORIGINAL.asc results.jsonl exponential_effects.csv reference.csv')
source, output, native, reference = map(Path, sys.argv[1:])
assert hashlib.sha256(source.read_bytes()).hexdigest() == '460f5d731f1848c8695d78a1fbf0db348d0daced4b8b5b0c2734bffa20b3e31c'
with native.open(newline='') as f:
  actual = list(csv.DictReader(f))
with reference.open(newline='') as f:
  expected = list(csv.DictReader(f))
rows = [json.loads(line) for line in output.read_text().splitlines()]
fit, = [r['data'] for r in rows if r['result_type'] == 'PoissonResult']
assert len(actual) == len(expected) == 10000
assert fit['sample']['retained_indices'] == list(range(10000))
assert fit['sample']['dropped_observations'] == 0
# Tolerances are absolute + relative; finite differencing is cancellation-sensitive.
for i, (got, want) in enumerate(zip(actual, expected), 1):
  assert got.keys() == want.keys()
  assert int(got['source_row']) == int(want['source_row']) == i
  for key in got.keys() - {'source_row'}:
    a, b = float(got[key]), float(want[key])
    tol = 2e-10 if key == 'numerical_derivative' else 2e-11
    assert math.isfinite(a) and abs(a-b) <= tol*(1+abs(b)), (i, key, a, b)
assert [c['name'] for c in fit['coefficients']] == ['intercept', 'x']
for got, want in zip([c['value'] for c in fit['coefficients']], [1.9829208730721173, -0.9896275707485616]):
  assert abs(got-want) < 1e-11
for row, target in zip(fit['covariance_matrix'], [[0.00020021026010574468, -0.00009879617578629009], [-0.00009879617578629009, 0.00009740204585534243]]):
  assert all(abs(a-b) < 1e-13 for a, b in zip(row, target))
summary, = [r['data'] for r in rows if r['result_type'] == 'TableResult' and r['data']['headers'][0] == 'observations' and len(r['data']['headers']) == 5]
for value, key in zip(summary['rows'][0][1:], ['analytical_derivative', 'numerical_derivative', 'finite_difference', 'derivative_at_mean']):
  target = sum(float(row[key]) for row in expected)/len(expected)
  assert abs(value-target) < 2e-10*(1+abs(target)), (key, value, target)
print('validated 10000 ordered records, 90000 numeric fields, fit sample, coefficient/covariance and four summaries')
