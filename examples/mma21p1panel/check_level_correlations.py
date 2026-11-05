#!/usr/bin/env python3
"""Audit observed panel-level correlations against raw MOM, R and author log."""
import argparse
import csv
import hashlib
import json
import math
from pathlib import Path
import re

SOURCE = 'lnhr lnwg kids ageh agesq disab id year'.split()
RAW = ['source_row'] + SOURCE


def records(path):
  with path.open(newline='') as f:
    return list(csv.DictReader(f))


def author_values(path):
  assert hashlib.sha256(path.read_bytes()).hexdigest() == '925aff11176cb78bbacf71abc69c2bef4876bdcc0e5544c001c2df4f5f8f56b5'
  text = path.read_text(); found = {}
  for variable,end in [('lnhr','. corr lnwg1979'),('lnwg','. corr upols1979')]:
    block = text[text.index('. corr '+variable+'1979'):text.index(end)]
    seen = set()
    for line in block.splitlines():
      match = re.match(r'\s*'+variable+r'(19\d\d)\s*\|\s*(.*)',line)
      if not match:
        continue
      year = int(match.group(1))
      if year in seen:
        continue
      seen.add(year)
      for i,value in enumerate(match.group(2).split()):
        other=1979+i
        if other < year:
          found[(variable,other,year)] = float(value)
    assert seen == set(range(1979,1989))
  assert len(found) == 90
  return found


def audit(native,reference,raw,author_log=None,synthetic=False):
  if not synthetic:
    assert hashlib.sha256(raw.read_bytes()).hexdigest() == '2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77'
    assert author_log is not None
  values=[float(v) for v in raw.read_text().split()]
  assert len(values)%8==0 and all(math.isfinite(v) for v in values)
  source=[dict(zip(SOURCE,values[i:i+8])) for i in range(0,len(values),8)]
  n,g=len(source),len({r['id'] for r in source});assert 3<=g<=532 and n==10*g
  events=[json.loads(l) for l in (native/'level_correlations.jsonl').read_text().splitlines()]
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  select=lambda kind:[e['data'] for e in events if e['result_type']==kind]
  assert 'descriptive, not a test or covariance correction' in json.dumps(events)
  kinds=[e['result_type'] for e in events]
  assert kinds[-4:]==['ExportResult','SqlCreateResult','TableResult','ExportResult']
  exports=select('ExportResult');assert len(exports)==2
  for event,name,count in zip(exports,['panel_level_source_rows.csv','panel_level_correlations.csv'],[n,200]):
    assert event['path']==event['dataset']['path']==name and event['dataset']['row_count']==count
  actual=records(native/'panel_level_source_rows.csv');expected=records(reference/'rows.csv')
  assert len(actual)==len(expected)==n and list(actual[0])==list(expected[0])==RAW
  for i,(a,b,original) in enumerate(zip(actual,expected,source),1):
    for key in RAW:
      assert float(a[key])==float(b[key])==(i if key=='source_row' else original[key]),(i,key)
  actual=records(native/'panel_level_correlations.csv');expected=records(reference/'correlations.csv')
  table=select('TableResult')[-1];assert table['headers']==['variable','year_a','year_b','people','correlation'] and len(table['rows'])==200
  assert len(actual)==len(expected)==200 and list(actual[0])==list(expected[0])==table['headers']
  matrix={};maximum=0
  for index,(a,b,row) in enumerate(zip(actual,expected,table['rows'])):
    variable=['lnhr','lnwg'][index//100];x,y=1979+(index%100)//10,1979+index%10
    assert a['variable']==b['variable']==row[0]==variable
    assert float(a['year_a'])==float(b['year_a'])==row[1]==x
    assert float(a['year_b'])==float(b['year_b'])==row[2]==y
    assert float(a['people'])==float(b['people'])==row[3]==g
    value=float(a['correlation']);reference_value=float(b['correlation']);maximum=max(maximum,abs(value-reference_value))
    assert math.isfinite(value) and abs(value-reference_value)<=1e-9 and abs(row[4]-value)<=1e-9
    matrix[(variable,x,y)]=value
  for (variable,x,y),value in matrix.items():
    assert abs(value-matrix[(variable,y,x)])<=1e-9
    if x==y:assert abs(value-1)<=1e-9
  max_log=0
  if author_log is not None:
    for key,value in author_values(author_log).items():
      difference=abs(matrix[key]-value);max_log=max(max_log,difference);assert difference<=0.00005,(key,matrix[key],value)
  print(f'PASS {"synthetic" if synthetic else "original MOM"}: {n}x9 source cells, 200 R/table cells (max {maximum:.12g})'+(f', 90 author-log cells (max {max_log:.12g})' if author_log else ''))


if __name__=='__main__':
  parser=argparse.ArgumentParser(description=__doc__)
  for name in ['native','reference','raw']:parser.add_argument(name,type=Path)
  parser.add_argument('--author-log',type=Path);parser.add_argument('--synthetic',action='store_true')
  args=parser.parse_args();audit(args.native,args.reference,args.raw,args.author_log,args.synthetic)
