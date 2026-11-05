#!/usr/bin/env python3
"""Audit CLI JSON/export against independent QR and pinned source records."""
import argparse
import csv
import hashlib
import json
import math
import re
from pathlib import Path

SHA={1:'a2a58a2548096fff062d9c7ff3cb820785a39c70f55a3e34abe1eb5c256d7283',
     2:'7f1662ba056298aafebcf72d0dbc54f49eac839df942de4cf033c527a7417bd2',
     3:'d68cd8b77d0a244092959f4819179e330c5ff40d3ae48a3c983fe884db43d1c6',
     4:'f727f69ade594b5d35a8f43769bad0566f5df1334e4fadad94298a5051ee924e'}

def rows(path):
  with path.open(newline='') as f: return list(csv.DictReader(f))

def close(actual,expected,label,p=False):
  a=float(actual);b=float(expected)
  if not math.isfinite(a) or not math.isfinite(b): raise AssertionError((label,a,b))
  tolerance=(abs(b)*1e-8 if b else 0) if p else 1e-9
  if abs(a-b)>tolerance: raise AssertionError((label,a,b,abs(a-b),tolerance))
  return abs(a-b)

def pointer(value,path):
  for part in path.strip('/').split('/'):
    value=value[int(part)] if isinstance(value,list) else value[part]
  return value

def main():
  ap=argparse.ArgumentParser();ap.add_argument('native',type=Path);ap.add_argument('reference',type=Path);ap.add_argument('source',type=Path)
  ap.add_argument('--author-log',type=Path,help='optional pinned original log for descriptive precision audit')
  mode=ap.add_mutually_exclusive_group(required=True);mode.add_argument('--case',type=int,choices=range(1,5));mode.add_argument('--synthetic',action='store_true');args=ap.parse_args()
  if args.author_log and not args.case: ap.error('--author-log requires --case')
  raw=args.source.read_bytes()
  if args.case and hashlib.sha256(raw).hexdigest()!=SHA[args.case]: raise AssertionError('source hash mismatch')
  records=[]
  for line in raw.decode('ascii').splitlines():
    fields=line.split()
    if len(fields)!=3 or fields[0]=='.': raise AssertionError('source shape or missing y')
    vals=[None if s=='.' else float(s) for s in fields]
    if any(v is not None and not math.isfinite(v) for v in vals): raise AssertionError('nonfinite source')
    records.append(vals)
  events=[json.loads(line) for line in (args.native/'out.jsonl').read_text().splitlines()]
  assert all(e['schema_version']==1 and not e.get('error') for e in events)
  fits=[e['data'] for e in events if e['result_type']=='RegressionResult']
  lincom=[e['data'] for e in events if e['result_type']=='LincomResult']
  tables=[e['data'] for e in events if e['result_type']=='TableResult']
  statuses=[e['data'] for e in events if e['result_type']=='StatusResult']
  assert len(fits)==2 and len(lincom)==6 and len(statuses)==1
  assert fits[0]['predictors']==['x1','x2'] and fits[1]['predictors']==['x1_filled','x2_filled']
  assert all(f['estimator']=='ols' and f['covariance']=='nonrobust' and f['include_intercept'] for f in fits)
  assert [c['label'] for c in lincom]==['intercept','x1','x2','intercept','x1_filled','x2_filled']
  assert all(c['ci_level']==95.0 for c in lincom)
  assert statuses[0]['active_table_name']=='missing_comparison_predictions' and statuses[0]['estimation_family']=='linear'
  assert statuses[0]['linear_model']['predictors']==['x1_filled','x2_filled']
  ref=rows(args.reference/'fit.csv');assert len(ref)==96
  maxerr=0.0
  for r in ref:
    i={'complete':0,'filled':1}[r['model']]
    obj=fits[i] if r['result']=='fit' else lincom[3*i+int(r['result'][-1])]
    maxerr=max(maxerr,close(pointer(obj,r['path']),r['value'],(r['model'],r['result'],r['path']),r['path'].endswith('p_value')))
  diag=rows(args.reference/'diagnostics.csv');assert len(diag)==1
  actual_diag=next(t for t in tables if t['headers'][0]=='rows_total');assert len(actual_diag['rows'])==1
  for key,v in zip(actual_diag['headers'],actual_diag['rows'][0]): close(v,diag[0][key],('diagnostic',key))
  assert 'not multiple imputation' in json.dumps(tables)
  predictions=rows(args.native/'missing_comparison_predictions.csv');expected=rows(args.reference/'predictions.csv')
  assert len(predictions)==len(expected)==len(records)
  expected_diag={'rows_total':len(records),'observed_x1':sum(v[1] is not None for v in records),
    'observed_x2':sum(v[2] is not None for v in records),'missing_x1':sum(v[1] is None for v in records),
    'missing_x2':sum(v[2] is None for v in records),'complete_cases':sum(v[1] is not None and v[2] is not None for v in records),
    'both_missing':sum(v[1] is None and v[2] is None for v in records)}
  for j,key in [(1,'mean_x1_observed'),(2,'mean_x2_observed')]:
    observed=[v[j] for v in records if v[j] is not None];expected_diag[key]=math.fsum(observed)/len(observed)
  for key,value in expected_diag.items(): close(diag[0][key],value,('source diagnostic',key))
  count=0;maxpred=0.0
  for i,(a,b,src) in enumerate(zip(predictions,expected,records),1):
    assert set(a)==set(b) and int(a['source_row'])==i
    for key,v in b.items():
      if v=='':
        if a[key]!='': raise AssertionError((i,key,'expected NULL',a[key]))
      else:
        maxpred=max(maxpred,close(a[key],v,(i,key)));count+=1
    for key,v in zip(('y','x1','x2'),src):
      if (v is None and a[key]!='') or (v is not None and (a[key]=='' or float(a[key])!=v)):
        raise AssertionError(('source mismatch',i,key,v,a[key]))
    assert int(a['x1_missing'])==int(src[1] is None) and int(a['x2_missing'])==int(src[2] is None)
    assert (a['complete_mean']=='')==(src[1] is None or src[2] is None)
    for k,v in [('x1_filled',src[1]),('x2_filled',src[2])]:
      close(a[k],float(diag[0]['mean_'+k[:2]+'_observed']) if v is None else v,(i,k,'fill'))
  complete=sum(v[1] is not None and v[2] is not None for v in records)
  assert fits[0]['sample']=={'total_observations':len(records),'dropped_observations':len(records)-complete}
  assert fits[1]['sample']=={'total_observations':len(records),'dropped_observations':0}
  if args.case:
    assert len(records)==1000 and complete==({1:812,2:564,3:812,4:564}[args.case])
  print(f'fit: {len(ref)} numerical fields, max abs {maxerr:.3g}; diagnostics {len(diag[0])} fields')
  print(f'predictions: {len(records)} ordered rows, {count} numerical fields, {3*len(records)} exact parsed-f64 source values/missing markers, max abs {maxpred:.3g}')
  if args.author_log:
    log=args.author_log.read_bytes()
    assert hashlib.sha256(log).hexdigest()=='5896a37748f219d62d9471a86b4a84dcb7d8bf65a67c2b423f2156de969d1186'
    blocks=re.split(r'\nColumn [23]: OLS with (?:listwise deletion of missing data|mean imputation of missing data)\n',log.decode('ascii').replace('\r\n','\n'))[1:]
    assert len(blocks)==8
    diffs=[];same=0
    for method in (0,1):
      seen=set()
      for line in blocks[2*(args.case-1)+method].splitlines():
        m=re.match(r'\s*(x1missing|x2missing|x1meanimpute|x2meanimpute|_cons)\s*\|\s*([-.\d]+)\s+([-.\d]+)\s+([-.\d]+)\s+([-.\d]+)\s+([-.\d]+)\s+([-.\d]+)\s*$',line)
        if not m or m[1] in seen: continue
        seen.add(m[1]);index=0 if m[1]=='_cons' else (1 if m[1].startswith('x1') else 2)
        for key,printed in [('value',float(m[2])),('standard_error',float(m[3]))]:
          actual=fits[method]['coefficients'][index][key];diffs.append(abs(actual-printed));same+=f'{actual:.4f}'==f'{printed:.4f}'
        if len(seen)==3: break
      assert len(seen)==3
    print(f'author log descriptive only: {len(diffs)} displayed coefficient/SE values, max absolute difference {max(diffs):.3g}, {same}/{len(diffs)} agree rounded to 4 decimals; log used pre-export data')

if __name__=='__main__': main()
