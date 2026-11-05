#!/usr/bin/env python3
"""Audit source-bound household FE estimates and full-dummy R predictions."""
import csv
from decimal import Decimal
import hashlib
import json
import math
from pathlib import Path
import shlex
import sys


def rows(path):
  with path.open(newline='') as f:
    return list(csv.DictReader(f))


def near(actual, expected, p=False):
  a,b=float(actual),float(expected)
  assert math.isfinite(a) and math.isfinite(b), (a,b)
  tol=abs(b)*1e-8 if p else 1e-9
  assert abs(a-b)<=tol, (a,b,tol)


def main():
  if len(sys.argv)!=4:
    raise SystemExit('usage: check_household_fe.py native-dir reference-dir vietnam_ex1.asc')
  native,ref,raw=map(Path,sys.argv[1:])
  assert hashlib.sha256(raw.read_bytes()).hexdigest()=='74cb37e8c14609e382d334e368a54f63bccd4e952d02bcbf535baf19115d4061'
  tokens=shlex.split(raw.read_text(encoding='ascii')); assert len(tokens)==5999*8
  source=rows(native/'vietnam.csv'); original=rows(ref/'source.csv')
  assert len(source)==len(original)==5999
  farms={'non farm':0,'farm':1}
  educ={'Never':0,'Nhatre':1,'<cap I':2,'Cap I':3,'Cap II':4,'Cap III':5,
        'Nghe SC':6,'THCN':7,'DHDC':8,'DHCD':9,'Thac sy':10,'PTS':11,'TS':12}
  for i,(a,b) in enumerate(zip(source,original)):
    assert set(a)==set(b) and int(a['source_row'])==int(b['source_row'])==i+1
    y,age,sex,size,farm,education,x,commune=tokens[8*i:8*i+8]
    values={'lnexp12m':'' if y=='.' else y,'age':age,'sex':sex,'hhsize':size,
            'farm':farms[farm],'educ':educ[education],'lnhhexp':x,'commune':commune}
    for key,value in values.items():
      if value=='': assert a[key]==b[key]==''
      else: assert Decimal(a[key])==Decimal(b[key])==Decimal(str(value)),(i,key)
  events=[json.loads(line) for line in (native/'result.jsonl').read_text().splitlines()]
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  assert [e['result_type'] for e in events]==['LoadResult','IsidResult','TableResult','SqlCreateResult',
    'FixedEffectsResult','LincomResult','LincomResult','TestResult','PredictionResult','PredictionResult',
    'PredictionResult','PredictionResult','SummarizeResult','TableResult','ExportResult']
  select=lambda kind:[e['data'] for e in events if e['result_type']==kind]
  fit=select('FixedEffectsResult')[0]; lincom=select('LincomResult'); joint=select('TestResult')[0]
  assert fit['observation_count']==5006 and fit['group_count']==194 and fit['singleton_groups']==1
  assert fit['entity_variable']=='commune' and fit['time_variable']=='source_row'
  assert fit['unit_effect_normalization']=='estimation_observation_weighted_zero_mean'
  assert fit['covariance']=='nonrobust' and fit['reference_distribution']=='student_t'
  assert fit['normalized_level']['parameter_names']==['_cons','lnhhexp','age','sex','hhsize','farm','educ']
  assert lincom[0]['label']=='_cons' and lincom[1]['label']=='lnhhexp'
  assert joint['df']==2 and joint['df_residual']==4806 and not joint['is_chi2']
  objects={'fit':fit,'common':lincom[0],'slope':lincom[1],'joint':joint}
  reference=rows(ref/'reference.csv')
  assert len(reference)==114 and len({(r['kind'],r['path']) for r in reference})==114
  for r in reference:
    v=objects[r['kind']]
    for key in r['path'].strip('/').split('/'):
      v=v[int(key)] if isinstance(v,list) else v[key]
    near(v,r['value'],r['path'].endswith('p_value'))
  actual=rows(native/'household_fe_predictions.csv'); expected=rows(ref/'predictions.csv')
  selected=[r for r in source if r['lnexp12m']!='']
  assert len(actual)==len(expected)==len(selected)==5006
  sum_u=0
  for a,b,s in zip(actual,expected,selected):
    assert set(a)==set(b)==set(s)|{'fe_xb','fe_xbu','fe_u','fe_residual'}
    for key in s: assert Decimal(a[key])==Decimal(b[key])==Decimal(s[key]),(s['source_row'],key)
    for key in ('fe_xb','fe_xbu','fe_u','fe_residual'): near(a[key],b[key])
    near(float(a['fe_xbu']),float(a['fe_xb'])+float(a['fe_u']))
    near(float(a['lnexp12m']),float(a['fe_xbu'])+float(a['fe_residual']))
    sum_u+=float(a['fe_u'])
  assert abs(sum_u/len(actual))<=1e-12
  assert select('ExportResult')[0]['dataset']['row_count']==5006
  print('validated 114 R full-dummy quantities, 5999 source rows and 5006 FE prediction rows')


if __name__=='__main__':
  main()
