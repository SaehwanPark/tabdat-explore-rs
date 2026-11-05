#!/usr/bin/env python3
"""Audit supplied-imputation marginal pooling against R and, optionally, SAS log."""
import argparse
import csv
import hashlib
import json
import math
from pathlib import Path
import re

LOG_SHA='0d0633bf3758205423cd909fb20a6b05c6c19ed729eca35e05ad7ce379ef7074'

def rows(path):
  with path.open(newline='') as f:return list(csv.DictReader(f))

def check(a,b,label,p=False):
  a=float(a);b=float(b)
  if not math.isfinite(a) or not math.isfinite(b):raise AssertionError((label,a,b))
  bound=(abs(b)*1e-8 if b else 0) if p else 1e-9
  if abs(a-b)>bound:raise AssertionError((label,a,b,abs(a-b),bound))
  return abs(a-b)

def source_rows(raw):
  blocks=re.split(r'Imputation Number=\s*(\d+)',raw.decode('ascii'))
  assert len(blocks)==21
  result=[]
  for j in range(1,len(blocks),2):
    imp=int(blocks[j]);seen=set()
    for line in blocks[j+1].splitlines():
      m=re.match(r'\s*(Intercept|x1missing|x2missing)\s+1\s+([-.\d]+)\s+([-.\d]+)\s+[-.\d]+\s+\S+\s*$',line)
      if not m or m[1] in seen:continue
      seen.add(m[1]);result.append((imp,'intercept' if m[1]=='Intercept' else m[1],float(m[2]),float(m[3])))
      if len(seen)==3:break
    assert len(seen)==3
  return result

def main():
  ap=argparse.ArgumentParser();ap.add_argument('native',type=Path);ap.add_argument('reference',type=Path)
  mode=ap.add_mutually_exclusive_group(required=True);mode.add_argument('--author-log',type=Path);mode.add_argument('--synthetic',action='store_true');args=ap.parse_args()
  inputs=rows(args.native/'estimates.csv');assert inputs
  if args.author_log:
    raw=args.author_log.read_bytes()
    if hashlib.sha256(raw).hexdigest()!=LOG_SHA:raise AssertionError('author log hash mismatch')
    source=source_rows(raw)
    actual=[(int(r['imputation']),r['parameter'],float(r['estimate']),float(r['standard_error'])) for r in inputs]
    if source!=actual:raise AssertionError('author-log estimates/order mismatch')
    assert len(source)==30
  events=[json.loads(line) for line in (args.native/'out.jsonl').read_text().splitlines()]
  assert all(e['schema_version']==1 and not e.get('error') for e in events)
  tables=[e['data'] for e in events if e['result_type']=='TableResult']
  assert 'no imputation' in json.dumps(tables)
  status=[e['data'] for e in events if e['result_type']=='StatusResult'];assert len(status)==1
  assert status[0]['active_table_name']=='rubin_marginal_report' and status[0]['estimation_family'] is None
  actual=rows(args.native/'rubin_marginal_report.csv');reference=rows(args.reference/'rubin_marginal_report.csv')
  assert len(actual)==len(reference)==3
  groups={}
  for row in inputs:
    groups.setdefault(row['parameter'],[]).append((int(row['imputation']),float(row['estimate']),float(row['standard_error'])))
  assert set(groups)=={'intercept','x1missing','x2missing'}
  for row in actual:
    entries=sorted(groups[row['parameter']]);m=len(entries);assert 2<=m<=100 and [e[0] for e in entries]==list(range(1,m+1))
    q=math.fsum(e[1] for e in entries)/m;w=math.fsum(e[2]**2 for e in entries)/m
    between=math.fsum((e[1]-q)**2 for e in entries)/(m-1);total=w+(1+1/m)*between
    for field,value in [('imputations',m),('estimate',q),('within_variance',w),('between_variance',between),('total_variance',total),('standard_error',math.sqrt(total)),('minimum',min(e[1] for e in entries)),('maximum',max(e[1] for e in entries))]:
      check(row[field],value,('source',row['parameter'],field))
  report=next(t for t in tables if t['headers'][0]=='parameter')
  assert len(report['rows'])==3
  maxdiff=0;count=0
  for index,(a,b,table_row) in enumerate(zip(actual,reference,report['rows'])):
    assert set(a)==set(b)==set(report['headers'])
    for key,value in b.items():
      rendered=table_row[report['headers'].index(key)]
      if key in ('parameter','reference'):
        assert a[key]==value==rendered
      elif value=='':
        assert a[key]=='' and rendered is None
      else:
        maxdiff=max(maxdiff,check(a[key],value,(index,key),key=='p_value'))
        check(rendered,value,('json',index,key),key=='p_value');count+=1
  print(f'R: {len(actual)} marginal rows, {count} numeric fields, max abs {maxdiff:.3g}; JSON and CSV match')
  if args.author_log:
    # The final published MIANALYZE rows use unrounded estimates unavailable in the log.
    target={'intercept':(.850332,.101155,33662.0),'x1missing':(1.135289,.149916,172.37),'x2missing':(.873834,.155737,124.82)}
    for r in actual:
      q,se,df=target[r['parameter']]
      print(f"author log {r['parameter']}: rounded-input minus printed unrounded fit: estimate {float(r['estimate'])-q:+.3g}, SE {float(r['standard_error'])-se:+.3g}, df {float(r['degrees_freedom'])-df:+.3g}; descriptive, not a strict oracle")
    print('author log: 30 ordered displayed per-imputation estimates/SEs bound exactly')

if __name__=='__main__':main()
