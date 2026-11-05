#!/usr/bin/env python3
"""Audit the ordinary auxiliary restriction; its F tail is not chi-square."""
import argparse
import csv
import hashlib
import json
import math
from pathlib import Path

SOURCE = "lnhr lnwg kids ageh agesq disab id year".split()


def audit(native, reference, raw, synthetic=False):
  if not synthetic:
    assert hashlib.sha256(raw.read_bytes()).hexdigest() == "2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77"
  values = [float(v) for v in raw.read_text().split()]
  assert len(values) % 8 == 0
  source = [dict(source_row=i//8+1, **dict(zip(SOURCE, values[i:i+8]))) for i in range(0, len(values), 8)]
  source = [r for r in source if math.isfinite(r['lnhr']) and math.isfinite(r['lnwg'])]
  n = len(source)
  events = [json.loads(l) for l in (native/'iid_hausman.jsonl').read_text().splitlines()]
  data = {}
  for kind in ['RegressionResult', 'LincomResult', 'TestResult', 'ExportResult']:
    matches = [e['data'] for e in events if e['result_type'] == kind]
    assert len(matches) == 1
    data[kind] = matches[0]
  rows = list(csv.DictReader((reference/'reference.csv').open()))
  assert len(rows) == len({(r['result_type'], r['path']) for r in rows}) == 39
  for row in rows:
    x = data[row['result_type']]
    for key in row['path'].split('/')[1:]:
      x = x[int(key)] if isinstance(x, list) else x[key]
    y = float(row['value']); tolerance = abs(y)*1e-8 if row['path'].endswith('p_value') else 1e-9
    assert x is not None and math.isfinite(x) and abs(x-y) <= tolerance, (row, x)
  fit, combination, test = (data[k] for k in ['RegressionResult', 'LincomResult', 'TestResult'])
  assert fit['covariance'] == 'nonrobust' and fit['predictors'] == ['quasi_lnwg', 'within_lnwg']
  assert fit['covariance_parameter_names'] == ['intercept', 'quasi_lnwg', 'within_lnwg']
  assert fit['sample'] == dict(total_observations=n, dropped_observations=0)
  assert fit['df_residual'] == combination['df_residual'] == test['df_residual'] == n-3
  assert test['is_chi2'] is False and test['df'] == 1
  assert abs(test['statistic']-combination['statistic']**2) <= 1e-9
  assert 'within-minus-Between' in json.dumps(events) and 'not asymptotic chi-square' in json.dumps(events)
  assert [e['result_type'] for e in events][-4:] == ['SummarizeResult', 'LincomResult', 'TestResult', 'ExportResult']
  export = data['ExportResult']
  assert export['path'] == export['dataset']['path'] == 'panel_iid_hausman_predictions.csv'
  assert export['dataset']['row_count'] == n
  actual = list(csv.DictReader((native/'panel_iid_hausman_predictions.csv').open()))
  expected = list(csv.DictReader((reference/'predictions.csv').open()))
  assert len(actual) == len(expected) == n and list(actual[0]) == list(expected[0]) and len(actual[0]) == 18
  for a, b, original in zip(actual, expected, source):
    for key in original:
      assert float(a[key]) == float(b[key]) == original[key], key
    for key in a.keys()-original.keys():
      assert abs(float(a[key])-float(b[key])) <= 1e-9, key
  print(f'PASS {"synthetic" if synthetic else "original MOM"}:39 QR fields,{n}x18 source/design/prediction cells;H={test["statistic"]};working F p={test["p_value"]};DF={n-3}')


if __name__ == '__main__':
  p = argparse.ArgumentParser(description=__doc__)
  for name in ['native', 'reference', 'raw']:
    p.add_argument(name, type=Path)
  p.add_argument('--synthetic', action='store_true')
  a = p.parse_args(); audit(a.native, a.reference, a.raw, a.synthetic)
