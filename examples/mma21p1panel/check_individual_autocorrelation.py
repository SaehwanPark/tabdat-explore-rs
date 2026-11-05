#!/usr/bin/env python3
"""Audit individual lag-one lnhr correlations against raw MOM, R and author log."""
import argparse
import csv
import hashlib
import json
import math
from pathlib import Path
import re

SOURCE='lnhr lnwg kids ageh agesq disab id year'.split()
RAW=['source_row']+SOURCE


def records(path):
  with path.open(newline='') as f:return list(csv.DictReader(f))


def author_values(path):
  assert hashlib.sha256(path.read_bytes()).hexdigest()=='925aff11176cb78bbacf71abc69c2bef4876bdcc0e5544c001c2df4f5f8f56b5'
  text=path.read_text();found={}
  for id,end in [(1,'. corr lnhr2 L.lnhr2'),(2,'. * And so on')]:
    block=text[text.index(f'. corr lnhr{id} L.lnhr{id}'):text.index(end)]
    assert '(obs=9)' in block
    match=re.search(r'^\s*L1\s*\|\s*([-+0-9.]+)\s+1\.0000',block,re.MULTILINE);assert match
    found[id]=float(match.group(1))
  return found


def audit(native,reference,raw,author_log=None,synthetic=False):
  if not synthetic:
    assert hashlib.sha256(raw.read_bytes()).hexdigest()=='2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77'
    assert author_log is not None
  values=[float(v) for v in raw.read_text().split()];assert len(values)%8==0 and all(math.isfinite(v) for v in values)
  source=[dict(source_row=i//8+1,**dict(zip(SOURCE,values[i:i+8]))) for i in range(0,len(values),8)]
  source.sort(key=lambda x:(x['id'],x['year']));ids=sorted({r['id'] for r in source});g=len(ids);n=len(source)-g;assert 3<=g<=532 and n==9*g
  annual=[(a,b) for a,b in zip(source[1:],source[:-1]) if a['id']==b['id']];assert len(annual)==n
  events=[json.loads(l) for l in (native/'individual_autocorrelation.jsonl').read_text().splitlines()]
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  select=lambda kind:[e['data'] for e in events if e['result_type']==kind]
  assert 'nine adjacent years within each person, not people across years' in json.dumps(events)
  assert 'descriptive, not a test or covariance correction' in json.dumps(events)
  assert [e['result_type'] for e in events][-4:]==['ExportResult','SqlCreateResult','TableResult','ExportResult']
  exports=select('ExportResult');assert len(exports)==2
  for event,name,count in zip(exports,['panel_individual_lnhr_pairs.csv','panel_individual_lnhr_correlations.csv'],[n,g]):
    assert event['path']==event['dataset']['path']==name and event['dataset']['row_count']==count
  actual=records(native/'panel_individual_lnhr_pairs.csv');expected=records(reference/'pairs.csv')
  assert len(actual)==len(expected)==n and list(actual[0])==list(expected[0]) and len(actual[0])==12
  for (a,b),(original,previous) in zip(zip(actual,expected),annual):
    assert original['year']==previous['year']+1
    for key in RAW:assert float(a[key])==float(b[key])==original[key],key
    for key in ['source_row','year','lnhr']:assert float(a['previous_'+key])==float(b['previous_'+key])==previous[key],key
  actual=records(native/'panel_individual_lnhr_correlations.csv');expected=records(reference/'correlations.csv')
  table=select('TableResult')[-1];assert table['headers']==['id','pairs','correlation'] and len(table['rows'])==g
  assert len(actual)==len(expected)==g and list(actual[0])==list(expected[0])==table['headers']
  result={};max_r=0
  for id,a,b,row in zip(ids,actual,expected,table['rows']):
    assert float(a['id'])==float(b['id'])==row[0]==id and float(a['pairs'])==float(b['pairs'])==row[1]==9
    value=float(a['correlation']);ref=float(b['correlation']);max_r=max(max_r,abs(value-ref))
    assert math.isfinite(value) and math.isfinite(ref) and abs(value-ref)<=1e-9 and abs(row[2]-value)<=1e-9
    result[int(id)]=value
  max_log=0
  if author_log is not None:
    for id,value in author_values(author_log).items():
      difference=abs(result[id]-value);max_log=max(max_log,difference);assert difference<=0.00005,(id,result[id],value)
  print(f'PASS {"synthetic" if synthetic else "original MOM"}: {n}x12 annual/source cells, {g} R/table correlations (max {max_r:.12g})'+(f', 2 author-log correlations (max {max_log:.12g})' if author_log else ''))


if __name__=='__main__':
  parser=argparse.ArgumentParser(description=__doc__)
  for name in ['native','reference','raw']:parser.add_argument(name,type=Path)
  parser.add_argument('--author-log',type=Path);parser.add_argument('--synthetic',action='store_true')
  args=parser.parse_args();audit(args.native,args.reference,args.raw,args.author_log,args.synthetic)
