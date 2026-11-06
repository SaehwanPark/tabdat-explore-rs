#!/usr/bin/env python3
"""Strict original-household GEE, sample and marginal-prediction audit."""
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


def near(a,b,p=False):
  a,b=float(a),float(b)
  assert math.isfinite(a) and math.isfinite(b),(a,b)
  tolerance=abs(b)*1e-8 if p else 1e-9
  assert abs(a-b)<=tolerance,(a,b,tolerance)


def main():
  if len(sys.argv) not in (4,5) or (len(sys.argv)==5 and sys.argv[4]!='--dta'):
    raise SystemExit('usage: check_household_gee.py native-dir reference-dir vietnam_ex1.asc|dta [--dta]')
  native,ref,raw=map(Path,sys.argv[1:4]);dta=len(sys.argv)==5
  expected_hash=('0ea6639f93a5bc3f53bcbd26b6865ad6908a04f41ab04d90e7c9aa4b7e9787bf' if dta else
                 '74cb37e8c14609e382d334e368a54f63bccd4e952d02bcbf535baf19115d4061')
  assert hashlib.sha256(raw.read_bytes()).hexdigest()==expected_hash
  source=rows(native/'vietnam.csv');original=rows(ref/'input.csv');pred=rows(ref/'predictions.csv')
  actual=rows(native/'household_gee_predictions.csv')
  assert len(source)==len(original)==len(pred)==len(actual)==5999
  tokens=None if dta else shlex.split(raw.read_text(encoding='ascii'))
  if tokens is not None: assert len(tokens)==5999*8
  farms={'non farm':0,'farm':1}
  education={'Never':0,'Nhatre':1,'<cap I':2,'Cap I':3,'Cap II':4,'Cap III':5,
             'Nghe SC':6,'THCN':7,'DHDC':8,'DHCD':9,'Thac sy':10,'PTS':11,'TS':12}
  for i,(s,o,p,a) in enumerate(zip(source,original,pred,actual)):
    assert set(s)==set(o)=={'source_row','lnexp12m','lnhhexp','age','sex','hhsize','farm','educ','commune'}
    assert set(a)==set(s)|{'gee_mean','gee_xb','gee_residual','included_in_gee'}
    assert set(p)=={'source_row','selected','mean','xb','residual'}
    assert int(s['source_row'])==int(o['source_row'])==int(p['source_row'])==int(a['source_row'])==i+1
    for key in s:
      if not s[key]: assert o[key]==a[key]==''
      else:
        # DTA and CSV writers may use different shortest decimal representations.
        assert float(s[key])==float(o[key])==float(a[key]),(i,key,s[key],o[key],a[key])
    if tokens is not None:
      y,age,sex,size,farm,educ,x,group=tokens[8*i:8*i+8]
      values={'lnexp12m':'' if y=='.' else y,'age':age,'sex':sex,'hhsize':size,
              'farm':farms[farm],'educ':education[educ],'lnhhexp':x,'commune':group}
      for key,value in values.items():
        if value=='': assert s[key]==''
        else: assert Decimal(s[key])==Decimal(str(value)),(i,key)
    assert p['selected'] in ('0','1')
    assert a['included_in_gee']==('true' if p['selected']=='1' else 'false')
    for key,reference in [('gee_mean','mean'),('gee_xb','xb'),('gee_residual','residual')]:
      if p[reference]=='': assert a[key]==''
      else: near(a[key],p[reference])
  assert sum(p['selected']=='1' for p in pred)==5006
  assert sum(bool(p['mean']) for p in pred)==5999
  assert sum(bool(p['residual']) for p in pred)==5006
  events=[json.loads(line) for line in (native/'result.jsonl').read_text().splitlines()]
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  assert [e['result_type'] for e in events]==['LoadResult','IsidResult','SqlCreateResult',
    'GaussianGeeResult','PredictionResult','PredictionResult','PredictionResult',
    'SummarizeResult','SqlCreateResult','TableResult','ExportResult']
  fit=next(e['data'] for e in events if e['result_type']=='GaussianGeeResult')
  assert fit['estimator']=='gaussian_identity_exchangeable_gee' and fit['include_intercept'] is True
  assert fit['scale']=='pearson_rss_over_n' and fit['correlation']=='ordered_pair_weighted_pearson'
  assert fit['working_covariance']=='inverse_gee_bread' and fit['sandwich_covariance']=='uncorrected_unit_sandwich'
  assert fit['covariance_correction']==1 and fit['reference_distribution']=='asymptotic_standard_normal'
  assert fit['prediction_scope']=='population_marginal_mean_identity_link'
  assert fit['sample']=={'total_observations':5999,'dropped_observations':993}
  assert fit['predictors']==['lnhhexp','age','sex','hhsize','farm','educ'] and fit['entity_variable']=='commune'
  assert 0<fit['iterations']<100 and fit['max_abs_scaled_score']<=1e-10
  names=['_cons']+fit['predictors']
  actual_fields={key:fit[key] for key in ['observation_count','group_count','minimum_group_size',
    'maximum_group_size','singleton_group_count','ordered_pair_count','phi','rho']}
  actual_fields.update(source_count=fit['sample']['total_observations'],parameter_count=len(names))
  for kind in ('working','sandwich'):
    inf=fit[kind];cs=inf['coefficients'];matrix=inf['covariance']
    assert [c['name'] for c in cs]==names and len(matrix)==len(names) and all(len(row)==len(names) for row in matrix)
    for i,c in enumerate(cs):
      for field,key in [('estimate','value'),('standard_error','standard_error'),('statistic','statistic'),
                        ('p_value','p_value'),('ci_lower','confidence_lower'),('ci_upper','confidence_upper')]:
        actual_fields[f'{kind}.{names[i]}.{field}']=c[key]
      for j,name in enumerate(names): actual_fields[f'{kind}.covariance.{names[i]}.{name}']=matrix[i][j]
    for key in ('wald_statistic','wald_df','wald_p_value'): actual_fields[f'{kind}.{key}']=inf[key]
  expected=rows(ref/'reference.csv')
  assert len(expected)==len(actual_fields)==198 and len({r['path'] for r in expected})==198
  assert set(actual_fields)=={r['path'] for r in expected}
  max_abs=0.0;max_relative_p=0.0
  for r in expected:
    path=r['path'];value=float(r['value']);got=float(actual_fields[path])
    near(got,value,path.endswith('p_value'))
    max_abs=max(max_abs,abs(got-value))
    if path.endswith('p_value') and value!=0: max_relative_p=max(max_relative_p,abs(got-value)/abs(value))
  assert events[-1]['data']['dataset']['row_count']==5999
  print('validated 198 R/geeM quantities, 5999 source/prediction rows and 5006 selected cases;',
        'DTA' if dta else 'ASCII','maximum absolute discrepancy',max_abs,'maximum nonzero-p relative discrepancy',max_relative_p)


if __name__=='__main__':
  main()
