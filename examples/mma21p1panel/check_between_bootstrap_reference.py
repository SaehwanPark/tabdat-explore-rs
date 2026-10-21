#!/usr/bin/env python3
"""Development-only original-source, mean-frame and bootstrap inference audit."""
import csv
import hashlib
import json
import math
from pathlib import Path
import sys
native,reference,raw=map(Path,sys.argv[1:])
assert hashlib.sha256(raw.read_bytes()).hexdigest()=='2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77'
def records(path):
  with path.open(newline='') as f: return list(csv.DictReader(f))
def close(a,e,label):
  a,e=float(a),float(e)
  assert math.isfinite(a) and math.isfinite(e) and abs(a-e)<=1e-9,(label,a,e)
# Validate the actual loaded source, before judging aggregate/statistical matches.
input_rows=records(native/'MOM.csv'); tokens=raw.read_text().split()
assert len(input_rows)==5320 and len(tokens)==8*5320
for i,row in enumerate(input_rows):
  assert int(row['source_row'])==i+1
  for j,k in enumerate(['lnhr','lnwg','kids','ageh','agesq','disab','id','year']): assert float(row[k])==float(tokens[8*i+j])
events=[json.loads(l) for l in (native/'between_bootstrap.jsonl').read_text().splitlines()]
assert all('error' not in e and e['schema_version']==1 for e in events)
fit=[e['data'] for e in events if e['result_type']=='BootstrapOlsResult']; assert len(fit)==1
fit=fit[0]
assert fit['seed']=='10001' and fit['rng']=='xoshiro256++/splitmix64/u64-rejection-v1'
assert fit['sample']=={'total_observations':532,'dropped_observations':0}
assert fit['interval_method']=='percentile_r_type7' and fit['confidence_level']==95
scalars=records(reference/'reference.csv'); assert len(scalars)==18
for row in scalars:
  value=fit
  for part in row['path'].strip('/').split('/'): value=value[int(part)] if isinstance(value,list) else value[part]
  close(value,row['value'],row['path'])
a=records(native/'panel_between_bootstrap_predictions.csv'); e=records(reference/'predictions.csv')
assert len(a)==len(e)==532 and list(a[0])==list(e[0]); cells=0
for actual,expected in zip(a,e):
  for k in expected:
    close(actual[k],expected[k],(k,expected['id'])); cells+=1
assert cells==4256
print(f'PASS:18 R bootstrap quantities,4,256 person-mean/design/prediction cells;532 independent means;all5,320 original loaded rows pinned')
