#!/usr/bin/env python3
"""Development-only original MOM unit-score ML audit; Python stdlib only."""
import csv
from decimal import Decimal
import hashlib
import json
import math
from pathlib import Path
import sys

native, reference, raw = map(Path, sys.argv[1:])
assert hashlib.sha256(raw.read_bytes()).hexdigest() == "2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77"


def records(path):
  with path.open(newline="") as stream:
    return list(csv.DictReader(stream))


def close(actual, expected, label, tail=False):
  a, e = float(actual), float(expected)
  tolerance = 1e-7 * max(abs(e), 1e-300) if tail else 1e-8 * max(abs(e), 1)
  assert math.isfinite(a) and math.isfinite(e) and abs(a-e) <= tolerance, (label, a, e, tolerance)


def reported_path(path):
  if path.startswith('/gls/coefficients/'):
    return path.replace('/gls/coefficients/', '/coefficients/', 1)
  if path.startswith('/gls/covariance/matrix/'):
    return path.replace('/gls/covariance/matrix/', '/covariance_matrix/', 1)
  if path.startswith('/confidence_intervals/'):
    _, _, index, bound = path.split('/')
    field = 'ci_lower' if bound == '0' else 'ci_upper'
    return f'/coefficients/{index}/{field}'
  return path


events = [json.loads(line) for line in (native / 'random_effects_ml_cluster.jsonl').read_text().splitlines()]
assert all(e['schema_version'] == 1 and 'error' not in e for e in events)
fits = [e['data'] for e in events if e['result_type'] == 'RandomEffectsResult']
assert len(fits) == 1
fit = fits[0]
assert fit['estimator'] == 'balanced_gaussian_random_effects_ml'
assert fit['covariance'] == 'cluster(id)'
assert fit['covariance_convention'] == 'unit_grouped_joint_ml_scores_observed_bread'
assert fit['reference_distribution'] == 'normal' and fit['wald_reference'] == 'chi_square'
assert fit['sample'] == {'total_observations': 5320, 'dropped_observations': 0}
assert fit['group_count'] == 532 and fit['group_size'] == 10
assert fit['covariance_parameter_names'] == ['intercept', 'lnwg']
assert fit['ml']['joint_parameter_names'] == ['intercept', 'lnwg', 'log_sigma_e_squared', 'log_sigma_e_squared_plus_t_sigma_u_squared']
assert fit['ml']['cluster']['group_count'] == 532
assert fit['cluster']['correction_convention'] == 'gaussian_marginal_ml_full_joint_scores_g_over_g_minus_1'
assert fit['ml']['likelihood_ratio_p_value'] is None
assert 'unit_variance_test' not in fit['ml']
assert 'unit-cluster' in fit['ml']['unit_variance_test_unavailable_reason']
assert fit['ml']['variance_component_inference']['method'] == 'unit_cluster_joint_score_log_wald'
scalars = records(reference / 'reference.csv')
assert len(scalars) == 53
for row in scalars:
  value = fit
  for part in reported_path(row['path']).strip('/').split('/'):
    value = value[int(part)] if isinstance(value, list) else value[part]
  close(value, row['value'], row, tail=row['path'].endswith('p_value'))
actual = records(native / 'panel_re_ml_cluster_predictions.csv')
expected = records(reference / 'mom_predictions.csv')
assert len(actual) == len(expected) == 5320
assert list(actual[0]) == [key for key in expected[0] if key != 'theta']
columns = ['lnhr', 'lnwg', 'kids', 'ageh', 'agesq', 'disab', 'id', 'year']
source = raw.read_text().split()
assert len(source) == 8*5320
cells = 0
for i, (a, e) in enumerate(zip(actual, expected)):
  for j, key in enumerate(columns):
    assert Decimal(a[key]) == Decimal(source[8*i+j]), (i, key, a[key], source[8*i+j])
  assert int(a['source_row']) == i+1
  for key in a:
    close(a[key], e[key], (key, i+1))
    cells += 1
  close(fit['theta'], e['theta'], ('theta', i+1))
print(f'PASS: {len(scalars)} independent R quantities, {cells} source/prediction cells; all 5320 records/532 people')
