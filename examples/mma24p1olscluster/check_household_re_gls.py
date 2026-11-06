#!/usr/bin/env python3
"""Audit unbalanced RE output and all original rows against R/plm."""
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
  assert math.isfinite(a) and math.isfinite(b),(a,b)
  tolerance=abs(b)*1e-8 if p else 1e-9
  assert abs(a-b)<=tolerance,(a,b,tolerance)


def main():
  if len(sys.argv)!=4:
    raise SystemExit('usage: check_household_re_gls.py native-dir reference-dir vietnam_ex1.asc')
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
    'TableResult','UnbalancedRandomEffectsGlsResult','PredictionResult','PredictionResult',
    'SummarizeResult','TableResult','ExportResult']
  select=lambda kind:[e['data'] for e in events if e['result_type']==kind]
  fit=select('UnbalancedRandomEffectsGlsResult')[0]
  assert fit['observation_count']==5006 and fit['group_count']==194
  assert fit['entity_variable']=='commune' and fit['time_variable']=='source_row'
  assert fit['variance_component_method']=='unbalanced_swamy_arora'
  assert fit['covariance']=='nonrobust' and fit['reference_distribution']=='normal'
  assert fit['covariance_convention']=='quasi_demeaned_rss_over_n_minus_k'
  assert fit['wald_reference']=='chi_square' and fit['balanced'] is False
  assert fit['sample']=={'total_observations':5006,'dropped_observations':0}
  names=['intercept','lnhhexp','age','sex','hhsize','farm','educ']
  assert fit['covariance_parameter_names']==names
  assert [c['name'] for c in fit['coefficients']]==names
  assert all(c['ci_level']==95 for c in fit['coefficients'])
  reference=rows(ref/'reference.csv')
  assert len(reference)==108 and len({r['path'] for r in reference})==108
  for r in reference:
    v=fit
    for key in r['path'].strip('/').split('/'):
      v=v[int(key)] if isinstance(v,list) else v[key]
    near(v,r['value'],r['path'].endswith('p_value'))
  actual=rows(native/'household_re_gls_predictions.csv'); expected=rows(ref/'predictions.csv')
  selected=[r for r in source if r['lnexp12m']!='']
  assert len(actual)==len(expected)==len(selected)==5006
  for a,b,s in zip(actual,expected,selected):
    assert set(a)==set(b)==set(s)|{'re_xb','re_residual'}
    for key in s: assert Decimal(a[key])==Decimal(b[key])==Decimal(s[key]),(s['source_row'],key)
    for key in ('re_xb','re_residual'): near(a[key],b[key])
    near(float(a['lnexp12m']),float(a['re_xb'])+float(a['re_residual']))
  assert select('ExportResult')[0]['dataset']['row_count']==5006
  print('validated 108 R/plm quantities, 5999 source rows and 5006 population-prediction rows')


if __name__=='__main__':
  main()
