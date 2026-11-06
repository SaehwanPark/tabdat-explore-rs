#!/usr/bin/env python3
"""Audit every original source/prediction row and ML inference against R/nlme."""
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
  if len(sys.argv) not in (4,5) or (len(sys.argv)==5 and sys.argv[4] not in ('--dta','--synthetic')):
    raise SystemExit('usage: check_household_re_ml.py native-dir reference-dir source [--dta|--synthetic]')
  mode=sys.argv[4] if len(sys.argv)==5 else '--ascii'
  native,ref,raw=map(Path,sys.argv[1:4])
  hashes={'--ascii':'74cb37e8c14609e382d334e368a54f63bccd4e952d02bcbf535baf19115d4061',
          '--dta':'0ea6639f93a5bc3f53bcbd26b6865ad6908a04f41ab04d90e7c9aa4b7e9787bf'}
  if mode!='--synthetic': assert hashlib.sha256(raw.read_bytes()).hexdigest()==hashes[mode]
  source=rows(native/'vietnam.csv'); original=rows(ref/'source.csv')
  assert len(source)==len(original)>0
  if mode!='--synthetic': assert len(source)==5999
  tokens=shlex.split(raw.read_text(encoding='ascii')) if mode=='--ascii' else None
  if tokens is not None: assert len(tokens)==5999*8
  farms={'non farm':0,'farm':1}
  educ={'Never':0,'Nhatre':1,'<cap I':2,'Cap I':3,'Cap II':4,'Cap III':5,'Nghe SC':6,'THCN':7,'DHCD':9,'DHDC':8,'Thac sy':10,'PTS':11,'TS':12}
  keys=['source_row','lnexp12m','age','sex','hhsize','farm','educ','lnhhexp','commune']
  for i,(a,b) in enumerate(zip(source,original)):
    assert list(a)==keys and list(b)==keys
    for key in keys:
      if a[key]=='': assert b[key]==''
      else: assert float(a[key])==float(b[key]),(i,key,a[key],b[key])
    if tokens is not None:
      assert int(a['source_row'])==i+1
      y,age,sex,size,farm,education,x,commune=tokens[8*i:8*i+8]
      values={'lnexp12m':'' if y=='.' else y,'age':age,'sex':sex,'hhsize':size,'farm':farms[farm],'educ':educ[education],'lnhhexp':x,'commune':commune}
      for key,value in values.items():
        if value=='': assert a[key]==''
        else: assert Decimal(a[key])==Decimal(str(value)),(i,key)
  events=[json.loads(line) for line in (native/'result.jsonl').read_text().splitlines()]
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  assert [e['result_type'] for e in events]==['LoadResult','IsidResult','TableResult','SqlCreateResult','TableResult','UnbalancedRandomEffectsMlResult','PredictionResult','PredictionResult','SummarizeResult','TableResult','ExportResult']
  fit=next(e['data'] for e in events if e['result_type']=='UnbalancedRandomEffectsMlResult')
  selected=sorted((r for r in source if r['lnexp12m']!=''),key=lambda r:int(r['source_row'])); G=len({r['commune'] for r in selected})
  assert fit['observation_count']==len(selected) and fit['group_count']==G
  if mode!='--synthetic': assert (len(selected),G,fit['minimum_group_size'],fit['maximum_group_size'])==(5006,194,1,39)
  assert fit['entity_variable']=='commune' and fit['time_variable']=='source_row'
  assert fit['estimator']=='unbalanced_gaussian_random_effects_ml' and fit['variance_component_method']=='unbalanced_gaussian_marginal_ml'
  assert fit['covariance']=='observed_information'
  assert fit['covariance_convention']==('constrained_observed_information' if fit['ml']['unit_variance_boundary'] else 'joint_observed_information')
  assert fit['reference_distribution']=='normal' and fit['wald_reference']=='chi_square' and fit['balanced'] is False
  assert fit['sample']=={'total_observations':len(selected),'dropped_observations':0}
  names=['intercept','lnhhexp','age','sex','hhsize','farm','educ']
  assert fit['covariance_parameter_names']==names and [c['name'] for c in fit['coefficients']]==names
  assert all(c['ci_level']==95 for c in fit['coefficients'])
  boundary=fit['ml']['unit_variance_boundary'];null_boundary=fit['ml']['null_sigma_u_squared']==0
  assert fit['ml']['joint_parameter_names']==names+['log_sigma_e_squared']+([] if boundary else ['log_sigma_u_squared'])
  if boundary or null_boundary:
    assert fit['ml']['likelihood_ratio_p_value'] is None and fit['ml']['likelihood_ratio_unavailable_reason']
  else: assert fit['ml']['likelihood_ratio_unavailable_reason'] is None
  if boundary:
    assert fit['sigma_u_squared']==0 and fit['ml']['variance_component_inference']['sigma_u_squared'] is None
    assert fit['ml']['variance_component_inference']['sigma_u_squared_unavailable_reason']
  reference=rows(ref/'reference.csv');count=210-(22 if boundary else 0)-(1 if boundary or null_boundary else 0)
  assert len(reference)==count and len({r['path'] for r in reference})==count
  for r in reference:
    v=fit
    for key in r['path'].strip('/').split('/'):
      v=v[int(key)] if isinstance(v,list) else v[key]
    if r['path']=='/ml/profile_score' and not boundary:
      # Both numerical roots must independently satisfy a tight absolute score
      # condition; bisection's eta tolerance does not make their scores equal.
      assert abs(float(v))<1e-8 and abs(float(r['value']))<1e-8
    else: near(v,r['value'],r['path'].endswith('p_value'))
  actual=rows(native/'household_re_ml_predictions.csv'); expected=rows(ref/'predictions.csv')
  assert len(actual)==len(expected)==len(selected)
  for a,b,s in zip(actual,expected,selected):
    assert set(a)==set(b)==set(s)|{'ml_xb','ml_residual'}
    for key in s: assert float(a[key])==float(b[key])==float(s[key]),(s['source_row'],key)
    for key in ('ml_xb','ml_residual'): near(a[key],b[key])
    near(float(a['lnexp12m']),float(a['ml_xb'])+float(a['ml_residual']))
  assert events[-1]['data']['dataset']['row_count']==len(selected)
  print('validated',count,'independent R/nlme quantities,',len(source),'source rows and',len(selected),'population-prediction rows')


if __name__=='__main__':
  main()
