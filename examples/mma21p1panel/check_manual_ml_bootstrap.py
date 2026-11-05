#!/usr/bin/env python3
"""Audit original fixed ML theta, whole-person bootstrap and retained predictions."""
import argparse
import csv
import hashlib
import json
import math
from pathlib import Path

SOURCE = 'lnhr lnwg kids ageh agesq disab id year'.split()
RAW = ['source_row'] + SOURCE


def close(actual, expected, label):
  assert actual is not None and not isinstance(actual, bool)
  assert math.isfinite(actual) and math.isfinite(expected) and abs(actual-expected) <= 1e-9, (label,actual,expected)


def pointer(data, path):
  for key in path.split('/')[1:]:
    data = data[int(key)] if isinstance(data, list) else data[key]
  return data


def audit(native, reference, raw, synthetic=False):
  if not synthetic:
    assert hashlib.sha256(raw.read_bytes()).hexdigest() == '2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77'
  values = [float(v) for v in raw.read_text().split()]
  assert len(values)%8 == 0 and all(math.isfinite(v) for v in values)
  source = [dict(zip(SOURCE, values[i:i+8])) for i in range(0,len(values),8)]
  n,g = len(source),len({r['id'] for r in source})
  assert 3 <= g <= 532 and n == 10*g
  events = [json.loads(line) for line in (native/'manual_ml_bootstrap.jsonl').read_text().splitlines()]
  assert all(e['schema_version'] == 1 and 'error' not in e for e in events)
  select = lambda kind: [e['data'] for e in events if e['result_type'] == kind]
  canonical, = select('RandomEffectsResult')
  bootstrap, = select('ClusterBootstrapOlsResult')
  fit, = select('RegressionResult')
  assert canonical['estimator'] == 'balanced_gaussian_random_effects_ml'
  assert canonical['sample'] == bootstrap['sample'] == fit['sample'] == dict(total_observations=n,dropped_observations=0)
  assert canonical['sigma_e_squared']>0 and canonical['sigma_u_squared']>=0 and 0<=canonical['theta']<1
  if canonical['sigma_u_squared'] == 0:
    assert canonical['theta'] == 0
  assert bootstrap['estimator'] == 'cluster_pairs_bootstrap_ols' and bootstrap['seed'] == '10001'
  assert bootstrap['rng'] == 'xoshiro256++/splitmix64/u64-rejection-v1'
  assert bootstrap['confidence_level'] == 95 and bootstrap['interval_method'] == 'percentile_r_type7'
  assert bootstrap['outcome'] == 'quasi_lnhr' and bootstrap['predictors'] == ['quasi_lnwg'] and bootstrap['include_intercept']
  assert bootstrap['covariance_parameter_names'] == ['intercept','quasi_lnwg']
  assert bootstrap['cluster_count'] == g and bootstrap['cluster_sizes'] == [10]*g
  assert bootstrap['cluster_order'] == 'first_seen_complete_case' and bootstrap['replicate_observation_counts'] == [n]*500
  for i in range(2):
    close(bootstrap['coefficients'][i]['estimate'],fit['coefficients'][i]['value'],'retained quasi coefficient')
    close(bootstrap['coefficients'][i]['estimate'],canonical['coefficients'][i]['value']*((1-canonical['theta']) if i==0 else 1),'population scale')
  refs = list(csv.DictReader((reference/'reference.csv').open()))
  assert len(refs) == len({(r['model'],r['path']) for r in refs}) == 22
  for row in refs:
    close(pointer({'canonical':canonical,'bootstrap':bootstrap}[row['model']],row['path']),float(row['value']),row['path'])
  kinds = [e['result_type'] for e in events]
  assert kinds[-5:] == ['TableResult','PredictionResult','PredictionResult','SummarizeResult','ExportResult']
  assert kinds.count('PredictionResult') == 4
  notes = json.dumps(events)
  assert all(s in notes for s in ['frozen original ML theta','not the historical Stata stream','not any bootstrap replicate','without retries'])
  export, = select('ExportResult')
  assert export['path'] == export['dataset']['path'] == 'panel_manual_ml_bootstrap_predictions.csv' and export['dataset']['row_count'] == n
  actual = list(csv.DictReader((native/'panel_manual_ml_bootstrap_predictions.csv').open()))
  expected = list(csv.DictReader((reference/'predictions.csv').open()))
  assert len(actual)==len(expected)==n and list(actual[0])==list(expected[0]) and len(actual[0])==22
  for i,(a,b,original) in enumerate(zip(actual,expected,source),1):
    for key in RAW:
      assert float(a[key]) == float(b[key]) == (i if key == 'source_row' else original[key]), (i,key)
    for key in a.keys()-set(RAW):
      close(float(a[key]),float(b[key]),key)
    for key in ['theta','sigma_e_squared','sigma_u_squared']:
      close(float(a[key]),canonical[key],key)
  draws = list(csv.DictReader((reference/'draws.csv').open()))
  refits = list(csv.DictReader((reference/'refits.csv').open()))
  assert len(draws) == len(refits) == 500
  for i,row in enumerate(draws,1):
    indices = [int(x) for x in row['indices'].split(':')]
    assert int(row['replication']) == i and len(indices) == g and all(0<=v<g for v in indices)
  print(f'PASS {"synthetic" if synthetic else "original MOM"}:22 independent ML/QR/bootstrap fields,500 QR refits,{n}x22 source/design/prediction cells,theta={canonical["theta"]}')


if __name__ == '__main__':
  parser = argparse.ArgumentParser(description=__doc__)
  for name in ['native','reference','raw']:
    parser.add_argument(name,type=Path)
  parser.add_argument('--synthetic',action='store_true')
  args = parser.parse_args(); audit(args.native,args.reference,args.raw,args.synthetic)
