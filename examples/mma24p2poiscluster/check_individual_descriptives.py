#!/usr/bin/env python3
"""Audit retained individual source records and independent variable-wise descriptive samples."""
from decimal import Decimal
import hashlib
import json
import math
from pathlib import Path
import shlex
import statistics
import sys

from check_reference import rows, near

VARIABLES = ['pharvis','lnhhexp','age','sex','married','educ','illness','injury','illdays','actdays','insurance','commune']
KEYS = ['source_row']+VARIABLES
FIELDS = ['count','mean','std_dev','minimum','maximum']
HASHES = {'--ascii':'e876f8ee17791881c6378de2761436e1e72e1eda84347089a9dc365ed290c85d',
          '--dta':'284a5dd732926fe6ee02157012b17d8751523b89f1dd2f652a6f4dd021f3dac3'}


def nullable(actual, expected):
  if expected is None or expected=='': assert actual is None or actual==''
  else: near(actual,expected,'descriptive')


def audit_source(native, reference, raw, mode):
  assert mode in ('--ascii','--dta','--csv')
  if mode in HASHES: assert hashlib.sha256(raw.read_bytes()).hexdigest()==HASHES[mode]
  source = rows(native/'vietnam_individual.csv')
  expected = rows(reference/'source.csv')
  assert 1<=len(source)<=40000 and len(source)==len(expected)
  if mode in HASHES: assert len(source)==27766
  if mode=='--csv': assert rows(raw)==source
  tokens=shlex.split(raw.read_text(encoding='ascii')) if mode=='--ascii' else None
  if tokens is not None: assert len(tokens)==len(source)*12
  for i,(a,b) in enumerate(zip(source,expected)):
    assert list(a)==list(b)==KEYS
    for key in KEYS:
      if a[key]=='': assert b[key]==''
      else: assert float(a[key])==float(b[key]),(i,key)
    if mode in HASHES: assert int(a['source_row'])==i+1
    if tokens is not None:
      for key,token in zip(VARIABLES,tokens[i*12:i*12+12]):
        if token=='.': assert a[key]==''
        else: assert Decimal(a[key])==Decimal(token),(i,key)
  ids=[int(r['source_row']) for r in source]
  assert len(set(ids))==len(ids) and all(0<x<=2**53-1 for x in ids)
  for r in source:
    for key in ('pharvis','commune'):
      if r[key]!='': assert (0 if key=='pharvis' else 1)<=int(r[key])<=2**53-1
    for key in VARIABLES:
      if r[key]!='': assert math.isfinite(float(r[key]))
  if mode in HASHES:
    assert [int(r['source_row']) for r in source if all(r[k]=='' for k in VARIABLES)]==[27734]
  return sorted(source,key=lambda r:int(r['source_row']))


def author_gates(log, summary, reference):
  assert hashlib.sha256(log.read_bytes()).hexdigest()=='b86983173c9c712d10df522a0d3ce566fd7c03b77d9f9739b9e776c44223d33e'
  section=log.read_text().split('. sum PHARVIS LNHHEXP AGE SEX MARRIED EDUC ILLNESS INJURY ILLDAYS ACTDAYS INSURANCE COMMUNE',1)[1].split('. sum LNHHEXP, detail',1)[0]
  printed={}
  for line in section.splitlines():
    if '|' not in line: continue
    name,values=line.split('|',1);name=name.strip().lower()
    if name in VARIABLES:
      assert name not in printed
      printed[name]=values.split();assert len(printed[name])==5
  assert list(printed)==VARIABLES
  for a,b in zip(summary,reference):
    for field,text in zip(FIELDS,printed[a['variable']]):
      value=Decimal(text);half=Decimal('.5')*Decimal(10)**value.as_tuple().exponent
      for actual in (a[field],b[field]): assert abs(Decimal(str(actual))-value)<=half,(a['variable'],field,text,actual)
  return 60


def audit(native, reference, raw, mode='--ascii', author=None):
  source=audit_source(native,reference,raw,mode)
  actual=rows(native/'individual_descriptive_source.csv');expected=rows(reference/'prepared_source.csv')
  assert len(actual)==len(expected)==len(source)
  for a,b,s in zip(actual,expected,source):
    assert list(a)==list(b)==KEYS
    for key in KEYS:
      if s[key]=='': assert a[key]==b[key]==''
      else: assert float(a[key])==float(b[key])==float(s[key]),(s['source_row'],key)
  members=[{'variable':name,'source_row':str(int(r['source_row']))} for name in VARIABLES for r in source if r[name]!='']
  assert rows(reference/'members.csv')==members
  events=[json.loads(line) for line in (native/'result.jsonl').read_text().splitlines()]
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  assert [e['result_type'] for e in events]==['LoadResult','IsidResult','TableResult','SqlCreateResult','SummarizeResult','ExportResult','SqlCreateResult','ExportResult','ActivateResult']
  assert events[0]['data']['dataset']['row_count']==len(source)
  assert events[1]['data']=={'missing_key_rows':0,'missok':False,'total_rows':len(source),'unique_groups':len(source),'variables':['source_row']}
  assert events[2]['data']=={'headers':['source_rows','wholly_missing_records','source_communes'],
    'rows':[[len(source),sum(all(r[k]=='' for k in VARIABLES) for r in source),len({int(r['commune']) for r in source if r['commune']!=''})]]}
  columns=[{'name':k,'data_type':'BIGINT' if k in ('source_row','pharvis','commune') else 'DOUBLE'} for k in KEYS]
  for i in (3,5,8):
    assert events[i]['data']['dataset']['row_count']==len(source)
    assert events[i]['data']['dataset']['columns']==columns
  assert events[3]['data']==events[8]['data']
  assert events[8]['data']['table_name']=='individual_descriptive_source'
  assert events[5]['data']['path']=='individual_descriptive_source.csv'
  assert events[7]['data']['path']=='individual_descriptives.csv'
  report_columns=[{'name':k,'data_type':'VARCHAR' if k=='variable' else 'BIGINT' if k=='count' else 'DOUBLE'} for k in ['variable']+FIELDS]
  for i in (6,7):
    assert events[i]['data']['dataset']['row_count']==12
    assert events[i]['data']['dataset']['columns']==report_columns
  summary=events[4]['data']['rows'];expected=rows(reference/'summary.csv');report=rows(native/'individual_descriptives.csv')
  assert len(summary)==len(expected)==len(report)==12
  assert [r['variable'] for r in summary]==[r['variable'] for r in expected]==[r['variable'] for r in report]==VARIABLES
  for a,b,c in zip(summary,expected,report):
    assert set(a)==set(b)==set(c)==set(['variable']+FIELDS)
    values=[float(r[a['variable']]) for r in actual if r[a['variable']]!='']
    check={'count':len(values),'mean':statistics.fmean(values) if values else None,
      'std_dev':statistics.stdev(values) if len(values)>1 else None,
      'minimum':min(values) if values else None,'maximum':max(values) if values else None}
    assert a['count']==int(b['count'])==int(c['count'])==len(values)
    for field in FIELDS[1:]:
      nullable(a[field],b[field]);nullable(c[field],b[field]);nullable(a[field],check[field])
  facts=0
  if author is not None:
    assert mode=='--dta';facts=author_gates(author,summary,expected)
  return len(source),60,facts


def main():
  if len(sys.argv) not in (4,5,6):
    raise SystemExit('usage: check_individual_descriptives.py native-dir reference-dir raw [--ascii|--dta|--csv] [author-log]')
  n,gates,facts=audit(*map(Path,sys.argv[1:4]),sys.argv[4] if len(sys.argv)>4 else '--ascii',Path(sys.argv[5]) if len(sys.argv)>5 else None)
  print('validated',gates,'independent R variable-wise fields, all',n,'source/prepared rows and memberships;',facts,'author facts')


if __name__=='__main__':
  main()
