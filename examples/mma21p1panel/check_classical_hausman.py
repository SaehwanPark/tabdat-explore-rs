#!/usr/bin/env python3
"""Development-only original source and retained-sample Hausman audit."""
import csv
import hashlib
import json
import math
from pathlib import Path
import sys

if len(sys.argv)!=5:
  raise SystemExit('usage: check_classical_hausman.py native-dir hausman-reference re-reference MOM.dat')
native,reference,re_reference,raw=map(Path,sys.argv[1:])
assert hashlib.sha256(raw.read_bytes()).hexdigest()=='2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77'
def records(path):
  with path.open(newline='') as stream: return list(csv.DictReader(stream))
def near(a,b,label):
  a,b=float(a),float(b)
  assert math.isfinite(a) and math.isfinite(b) and abs(a-b)<=1e-9+1e-9*abs(b),(label,a,b)
source=records(native/'MOM.csv'); tokens=raw.read_text().split()
assert len(source)==5320 and len(tokens)==5320*8
for i,row in enumerate(source):
  assert int(row['source_row'])==i+1
  for j,key in enumerate(['lnhr','lnwg','kids','ageh','agesq','disab','id','year']): assert float(row[key])==float(tokens[i*8+j])
events=[json.loads(line) for line in (native/'result.jsonl').read_text().splitlines()]
assert all(e['schema_version']==1 and 'error' not in e for e in events)
results=[e['data'] for e in events if e['result_type']=='ClassicalPanelHausmanResult']
assert len(results)==1; result=results[0]
assert result['test']=='classical_panel_fe_re_covariance_difference'
assert result['reference_distribution']=='asymptotic_chi_square' and result['covariance']=='model_based_nonrobust'
assert result['outcome']=='lnhr' and result['predictor']=='lnwg' and result['entity_variable']=='id'
refs=records(reference/'reference.csv'); assert len(refs)==12
for r in refs: near(result[r['field']],r['value'],r['field'])
assert abs(result['statistic']-13.73)<.005 and abs(result['p_value']-.0002)<.00005
actual=records(native/'panel_classical_hausman_predictions.csv'); expected=records(re_reference/'predictions.csv')
assert len(actual)==len(expected)==5320 and list(actual[0])==list(expected[0])
for a,e in zip(actual,expected):
  for key in e: near(a[key],e[key],(e['source_row'],key))
print('PASS: 12 plm Hausman quantities, all original rows and 58,520 prediction/design cells')
