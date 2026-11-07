#!/usr/bin/env python3
"""Audit variable-wise source samples, DOUBLE-generated levels and descriptive artifacts."""
import hashlib
import json
import math
from pathlib import Path
import statistics
import sys

from check_household_bp_lm import audit_source, rows, near
from check_household_re_ml_author import rounded_match

VARIABLES = ['lnexp12m','age','sex','hhsize','farm','educ','hhexp','lnhhexp','commune']
FIELDS = ['count','mean','std_dev','minimum','maximum']
KEYS = ['source_row','lnexp12m','age','sex','hhsize','farm','educ','lnhhexp','commune']


def nullable(actual, expected):
  if expected is None or expected == '':
    assert actual is None or actual == '', (actual,expected)
  else:
    near(actual,expected)


def author_gates(log, summary, reference):
  assert hashlib.sha256(log.read_bytes()).hexdigest()=='5e74c8de58eaf833b0070d68b289d61fad43831f3eb8b57c06818a6bf549837e'
  section = log.read_text().split('. sum LNEXP12M AGE SEX HHSIZE FARM EDUC HHEXP LNHHEXP COMMUNE',1)[1].split('. outfile',1)[0]
  printed = {}
  for line in section.splitlines():
    if '|' not in line: continue
    name,values = line.split('|',1)
    name = name.strip().lower()
    if name in VARIABLES:
      assert name not in printed
      printed[name] = values.split()
      assert len(printed[name])==5
  assert list(printed)==VARIABLES
  for a,b in zip(summary,reference):
    for field,text in zip(FIELDS,printed[a['variable']]):
      assert rounded_match(a[field],text),(a['variable'],field,text,a[field])
      assert rounded_match(b[field],text),(b['variable'],field,text,b[field])
  return 45


def audit(native, reference, raw, mode='--ascii', author=None):
  assert mode in ('--ascii','--dta','--csv')
  source = audit_source(native,reference/'source.csv',raw,mode,minimum_rows=1)
  assert 1<=len(source)<=40000
  ids = [int(r['source_row']) for r in source]
  assert len(set(ids))==len(ids)
  assert all(0<int(r[k])<=2**53-1 for r in source for k in ('source_row','commune'))
  source = sorted(source,key=lambda r:int(r['source_row']))
  actual = rows(native/'household_descriptive_source.csv')
  expected = rows(reference/'prepared_source.csv')
  assert len(actual)==len(expected)==len(source)
  for a,b,s in zip(actual,expected,source):
    assert list(a)==list(b)==KEYS+['hhexp']
    for key in KEYS:
      if s[key]=='': assert a[key]==b[key]==''
      else: assert float(a[key])==float(b[key])==float(s[key]),(s['source_row'],key)
    level = None if s['lnhhexp']=='' else math.exp(float(s['lnhhexp']))
    nullable(a['hhexp'],b['hhexp']);nullable(a['hhexp'],level)
    if level is not None: assert math.isfinite(level) and level>0
  members = [{'variable':name,'source_row':str(int(r['source_row']))} for name in VARIABLES for r in source if
             (r['lnhhexp'] if name=='hhexp' else r[name])!='']
  assert rows(reference/'members.csv')==members
  events = [json.loads(line) for line in (native/'result.jsonl').read_text().splitlines()]
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  assert [e['result_type'] for e in events]==['LoadResult','IsidResult','TableResult','SqlCreateResult',
    'TableResult','SummarizeResult','ExportResult','SqlCreateResult','ExportResult','ActivateResult']
  assert events[0]['data']['dataset']['row_count']==len(source)
  assert events[1]['data']=={'missing_key_rows':0,'missok':False,'total_rows':len(source),'unique_groups':len(source),'variables':['source_row']}
  assert events[2]['data']=={'headers':['source_rows','missing_outcomes','source_communes'],
    'rows':[[len(source),sum(r['lnexp12m']=='' for r in source),len({int(r['commune']) for r in source})]]}
  assert events[4]['data']=={'headers':['prepared_rows'],'rows':[[len(source)]]}
  columns = [{'name':k,'data_type':'BIGINT' if k in ('source_row','commune') else 'DOUBLE'} for k in KEYS+['hhexp']]
  for i in (3,6,9):
    d = events[i]['data']['dataset']
    assert d['row_count']==len(source) and d['columns']==columns
  assert events[3]['data']==events[9]['data']
  assert events[9]['data']['table_name']=='household_descriptive_source'
  assert events[6]['data']['path']=='household_descriptive_source.csv'
  assert events[8]['data']['path']=='household_descriptives.csv'
  assert events[7]['data']['dataset']['row_count']==events[8]['data']['dataset']['row_count']==9
  report_columns = [{'name':k,'data_type':'VARCHAR' if k=='variable' else 'BIGINT' if k=='count' else 'DOUBLE'} for k in ['variable']+FIELDS]
  assert events[7]['data']['dataset']['columns']==events[8]['data']['dataset']['columns']==report_columns
  summary = events[5]['data']['rows']
  expected = rows(reference/'summary.csv')
  report = rows(native/'household_descriptives.csv')
  assert len(summary)==len(expected)==len(report)==9
  assert [r['variable'] for r in summary]==[r['variable'] for r in expected]==[r['variable'] for r in report]==VARIABLES
  for a,b,c in zip(summary,expected,report):
    assert set(a)==set(b)==set(c)==set(['variable']+FIELDS)
    values = [float(r[a['variable']]) for r in actual if r[a['variable']]!='']
    assert all(math.isfinite(v) for v in values)
    check = {'count':len(values),'mean':statistics.fmean(values) if values else None,
      'std_dev':statistics.stdev(values) if len(values)>1 else None,
      'minimum':min(values) if values else None,'maximum':max(values) if values else None}
    assert a['count']==int(b['count'])==int(c['count'])==len(values)
    for field in FIELDS[1:]:
      nullable(a[field],b[field]);nullable(c[field],b[field]);nullable(a[field],check[field])
  facts = 0
  if author is not None:
    assert mode=='--dta'
    facts = author_gates(author,summary,expected)
  return len(source),45,facts


def main():
  if len(sys.argv) not in (4,5,6):
    raise SystemExit('usage: check_household_descriptives.py native-dir reference-dir raw [--ascii|--dta|--csv] [author-log]')
  n,gates,facts = audit(*map(Path,sys.argv[1:4]),sys.argv[4] if len(sys.argv)>4 else '--ascii',Path(sys.argv[5]) if len(sys.argv)>5 else None)
  print('validated',gates,'independent R variable-wise summary fields, all',n,'source/prepared rows and memberships;',facts,'author facts')


if __name__=='__main__':
  main()
