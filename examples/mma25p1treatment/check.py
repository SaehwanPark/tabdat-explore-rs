#!/usr/bin/env python3
"""Audit source rows, fits, robust inference, and predictions against base R."""
import csv
import hashlib
import json
import math
from pathlib import Path
import sys

if len(sys.argv)!=5:
  raise SystemExit('usage: check.py nswpsid.da1 result.jsonl treatment_predictions.csv reference-directory')
source,result,export,reference=map(Path,sys.argv[1:])
assert hashlib.sha256(source.read_bytes()).hexdigest()=='810f6dbfbe0ffbcc52676946c330f49d4d8bf5cafc06980e912ca945ac7f4e44'
def read(path):
  with path.open(newline='') as stream:
    return list(csv.DictReader(stream))
def near(actual, expected, name):
  actual,expected=float(actual),float(expected)
  assert math.isfinite(actual) and math.isfinite(expected), (name,actual,expected)
  assert abs(actual-expected)<=1e-8+1e-9*abs(expected), (name,actual,expected)
  if name.endswith('.p_value') and 0 < abs(expected) < 1e-20:
    assert abs(actual/expected-1)<1e-7, (name,actual,expected)
raw=[line.split() for line in source.read_text().splitlines() if line.strip()]
prepared=read(result.parent/'nswpsid.csv'); native=read(export); expected=read(reference/'predictions.csv')
assert len(raw)==len(prepared)==len(native)==len(expected)==2675
fields=['treat','age','educ','black','hisp','marr','re74','re75','re78','u74_source','u75_source']
for i,(r,p,n,e) in enumerate(zip(raw,prepared,native,expected),1):
  assert p['source_row']==n['source_row']==e['source_row']==str(i)
  for field,value in zip(fields,r):
    assert float(p[field])==float(value)
    if field in n: assert float(n[field])==float(value)
    rfield = {'u74_source':'u74','u75_source':'u75'}.get(field,field)
    assert float(e[rfield])==float(value)
  for field in ('age_sq','nodegree','unadjusted_mean','adjusted_mean'):
    near(n[field],e[field],f'{i}.{field}')
leaves=[json.loads(line) for line in result.read_text().splitlines()]
fits=[v['data'] for v in leaves if v['result_type']=='RegressionResult']
assert len(fits)==2
modelnames=['unadjusted','adjusted']
coefs=read(reference/'coefficients.csv'); covs=read(reference/'covariances.csv')
for fit,name in zip(fits,modelnames):
  assert fit['covariance']=='robust' and fit['observation_count']==2675
  assert fit['sample']['dropped_observations']==0
  refs=[r for r in coefs if r['method']==name]
  assert len(refs)==len(fit['coefficients'])
  for coefficient,r in zip(fit['coefficients'],refs):
    assert coefficient['name']==('intercept' if r['parameter']=='(Intercept)' else r['parameter'])
    for field,reference_field in [('value','value'),('standard_error','se'),('statistic','statistic'),('p_value','p_value')]:
      near(coefficient[field],r[reference_field],name+'.'+field)
  for r in (r for r in covs if r['method']==name):
    names=[c['name'] for c in fit['coefficients']]
    index=lambda s: names.index('intercept' if s=='(Intercept)' else s)
    near(fit['covariance_matrix'][index(r['row'])][index(r['column'])],r['value'],name+'.covariance')
summary,=[v['data'] for v in leaves if v['result_type']=='TableResult' and 'earnings_1978' in v['data']['headers']]
assert [r[1] for r in summary['rows']]==[2490,185]
for group in (0,1):
  row=summary['rows'][group]; subset=[r for r in raw if int(r[0])==group]
  near(row[4],sum(float(r[2])<12 for r in subset)/len(subset),f'group{group}.nodegree')
  for j,index in [(2,1),(3,2),(5,3),(6,4),(7,5),(8,6),(9,7),(10,8)]:
    near(row[j],math.fsum(float(r[index]) for r in subset)/len(subset),f'group{group}.{j}')
lincoms=[v['data'] for v in leaves if v['result_type']=='LincomResult']
assert len(lincoms)==2
for item,name in zip(lincoms,modelnames):
  ref=next(r for r in coefs if r['method']==name and r['parameter']=='treat')
  assert item['df_residual']==(2673 if name=='unadjusted' else 2665)
  for field,rfield in [('estimate','value'),('standard_error','se'),('statistic','statistic'),('p_value','p_value'),('ci_lower','lower'),('ci_upper','upper')]:
    near(item[field],ref[rfield],name+'.'+field)
table,=[v['data'] for v in leaves if v['result_type']=='EstimateTableResult']
assert [m['name'] for m in table['models']]==modelnames
for model,fit in zip(table['models'],fits):
  assert model['sample']['retained_indices']==list(range(2675)) and model['covariance_matrix']==fit['covariance_matrix']
  for a,b in zip(model['coefficients'],fit['coefficients']):
    for key in ('name','value','standard_error','statistic','p_value'): assert a[key]==b[key]
print('PASS: 2675 original rows, two HC1 fits and full covariance, stored reports, group means and all projections')
