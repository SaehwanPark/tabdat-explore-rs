#!/usr/bin/env python3
"""Development-only source/mean-frame audit; defaults retain the B200 contract."""
import argparse
import csv
import hashlib
import json
import math
from pathlib import Path


def records(path):
  with path.open(newline='') as f: return list(csv.DictReader(f))


def close(a,e,label):
  a,e=float(a),float(e)
  assert math.isfinite(a) and math.isfinite(e) and abs(a-e)<=1e-9,(label,a,e)


def audit(native,reference,raw,b500=False,synthetic=False):
  assert not synthetic or b500
  if not synthetic:
    assert hashlib.sha256(raw.read_bytes()).hexdigest()=='2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77'
  fields=['lnhr','lnwg','kids','ageh','agesq','disab','id','year']
  input_rows=records(native/'MOM.csv'); tokens=raw.read_text().split()
  assert input_rows and len(tokens)==8*len(input_rows)
  if not synthetic:assert len(input_rows)==5320
  members={}
  for i,row in enumerate(input_rows):
    assert int(row['source_row'])==i+1
    for j,k in enumerate(fields):assert float(row[k])==float(tokens[8*i+j]),(i,k)
    key=float(row['id']);year=float(row['year'])
    assert key==int(key) and 1<=key<=532 and year==int(year) and 1979<=year<=1988
    assert year not in members.setdefault(key,set());members[key].add(year)
  g=len(members)
  if b500:assert 3<=g<=532 and all(years==set(range(1979,1989)) for years in members.values())
  suffix='_b500' if b500 else ''
  events=[json.loads(l) for l in (native/f'between_bootstrap{suffix}.jsonl').read_text().splitlines()]
  assert events and all('error' not in e and e['schema_version']==1 for e in events)
  fits=[e['data'] for e in events if e['result_type']=='BootstrapOlsResult'];assert len(fits)==1
  fit=fits[0]
  assert fit['seed']=='10001' and fit['rng']=='xoshiro256++/splitmix64/u64-rejection-v1'
  assert fit['sample']=={'total_observations':g,'dropped_observations':0}
  assert fit['replications']==(500 if b500 else 200)
  assert fit['interval_method']=='percentile_r_type7' and fit['confidence_level']==95
  assert [c['name'] for c in fit['coefficients']]==['intercept','mean_lnwg']
  assert len(fit['covariance_matrix'])==2 and all(len(row)==2 for row in fit['covariance_matrix'])
  scalars=records(reference/'reference.csv');assert len(scalars)==18
  assert len({r['path'] for r in scalars})==18
  for row in scalars:
    value=fit
    for part in row['path'].strip('/').split('/'):value=value[int(part)] if isinstance(value,list) else value[part]
    close(value,row['value'],row['path'])
  a=records(native/f'panel_between_bootstrap{suffix}_predictions.csv');e=records(reference/'predictions.csv')
  assert len(a)==len(e)==g and list(a[0])==list(e[0]);cells=0
  assert [float(row['id']) for row in a]==sorted(members)
  for actual,expected in zip(a,e):
    for key in expected:close(actual[key],expected[key],(key,expected['id']));cells+=1
  assert cells==8*g
  print(f'PASS:B{fit["replications"]};18 R bootstrap quantities,{cells} person-mean/design/prediction cells;{g} independent means;all{len(input_rows)} raw input rows pinned')


if __name__=='__main__':
  parser=argparse.ArgumentParser(description=__doc__)
  for name in ['native','reference','raw']:parser.add_argument(name,type=Path)
  parser.add_argument('--b500',action='store_true')
  parser.add_argument('--synthetic',action='store_true')
  args=parser.parse_args()
  if args.synthetic and not args.b500:parser.error('--synthetic requires --b500')
  audit(args.native,args.reference,args.raw,args.b500,args.synthetic)
