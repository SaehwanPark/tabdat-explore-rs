#!/usr/bin/env python3
"""Audit source, fit, conditional-X effects and delta intervals against R."""
import csv
import hashlib
import json
import math
from pathlib import Path
import sys

if len(sys.argv) != 5:
  raise SystemExit('usage: check_uncertainty.py original.asc result.jsonl effects_uncertainty.csv reference.csv')
source, output, native, reference = map(Path, sys.argv[1:])
assert hashlib.sha256(source.read_bytes()).hexdigest() == '460f5d731f1848c8695d78a1fbf0db348d0daced4b8b5b0c2734bffa20b3e31c'
with native.open(newline='') as f:
  actual = list(csv.DictReader(f))
with reference.open(newline='') as f:
  expected = list(csv.DictReader(f))
results = [json.loads(line) for line in output.read_text().splitlines()]
fit, = [v['data'] for v in results if v['result_type'] == 'PoissonResult']
assert fit['sample']['retained_indices'] == list(range(10000))
assert fit['sample']['dropped_observations'] == 0
assert fit['covariance'] == 'model_based_poisson_information'
assert [c['name'] for c in fit['coefficients']] == ['intercept', 'x']
assert len(actual) == len(expected) == 3
for row, ref in zip(actual, expected):
  assert row.keys() == ref.keys() and row['effect'] == ref['effect'] and row['n'] == ref['n'] == '10000'
  for key in ('estimate', 'standard_error', 'ci_lower', 'ci_upper'):
    a, b = float(row[key]), float(ref[key])
    # R gradients use a verified central difference, so uncertainty is less precise than point estimates.
    tol = 1e-9 if key != 'estimate' else 2e-11
    assert math.isfinite(a) and abs(a-b) <= tol*(1+abs(b)), (row['effect'], key, a, b)
print('validated ordered 10000-row fit sample and 12 effect/SE/interval values against R')
