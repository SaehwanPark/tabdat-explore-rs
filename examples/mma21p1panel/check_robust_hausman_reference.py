#!/usr/bin/env python3
"""Development-only original MOM source, auxiliary-design and inference audit."""
import csv
import hashlib
import json
import math
from pathlib import Path
import sys

native, reference, raw = map(Path, sys.argv[1:])
assert hashlib.sha256(raw.read_bytes()).hexdigest() == '2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77'

def records(path):
  with path.open(newline='') as stream:
    return list(csv.DictReader(stream))

def close(actual, expected, label, tail=False):
  a,e=float(actual),float(expected)
  tolerance=abs(e)*1e-8 if tail else 1e-9
  assert math.isfinite(a) and math.isfinite(e) and abs(a-e)<=tolerance,(label,a,e,tolerance)

events=[json.loads(line) for line in (native/'robust_hausman.jsonl').read_text().splitlines()]
assert all(e['schema_version']==1 and 'error' not in e for e in events)
results={e['result_type']:e['data'] for e in events}
fit=results['RegressionResult']; test=results['TestResult']
assert fit['sample']=={'total_observations':5320,'dropped_observations':0}
assert fit['covariance']=='cluster(id)' and fit['predictors']==['quasi_lnwg','within_lnwg']
assert fit['covariance_parameter_names']==['intercept','quasi_lnwg','within_lnwg']
assert test['is_chi2'] is False and test['df']==1 and test['df_residual']==531
assert test['constraints']==['within_lnwg = 0']
scalars=records(reference/'reference.csv'); assert len(scalars)==39
for row in scalars:
  value=results[row['result_type']]
  for part in row['path'].strip('/').split('/'):
    value=value[int(part)] if isinstance(value,list) else value[part]
  close(value,row['value'],row,tail=row['path'].endswith('p_value'))
actual=records(native/'panel_robust_hausman_predictions.csv'); expected=records(reference/'predictions.csv')
assert len(actual)==len(expected)==5320 and list(actual[0])==list(expected[0])
cells=0
for a,e in zip(actual,expected):
  for key in e:
    if key in ('source_row','id','year'): assert float(a[key])==float(e[key]),(key,a,e)
    else: close(a[key],e[key],(key,e['source_row']))
    cells+=1
assert cells==95760
print(f'PASS: {len(scalars)} independent R/plm/sandwich quantities, {cells} source/design/prediction cells across5320 rows')
