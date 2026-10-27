#!/usr/bin/env python3
"""Verify paired CLI results against the pinned source and independent R."""
import csv
import hashlib
import json
import math
from pathlib import Path
import sys

if len(sys.argv)!=5:
  raise SystemExit('usage: check_paired.py nswpsid.da1 paired_result.jsonl paired_predictions.csv reference-directory')
source, result, export, reference = map(Path,sys.argv[1:])
assert hashlib.sha256(source.read_bytes()).hexdigest()=='810f6dbfbe0ffbcc52676946c330f49d4d8bf5cafc06980e912ca945ac7f4e44'
def rows(path):
  with path.open(newline='') as stream:
    return list(csv.DictReader(stream))
def near(a,b,label):
  a,b=float(a),float(b)
  assert math.isfinite(a) and math.isfinite(b) and abs(a-b)<=1e-8+1e-9*abs(b),(label,a,b)
  if label.endswith('p_value') and 0<abs(b)<1e-20:
    assert abs(a/b-1)<1e-7,(label,a,b)
raw=[line.split() for line in source.read_text().splitlines() if line.strip()]
prepared=rows(result.parent/'nswpsid.csv'); native=rows(export); ref=rows(reference/'predictions.csv')
assert len(raw)==len(prepared)==2675 and len(native)==len(ref)==5350
fields=['treat','age','educ','black','hisp','marr','re74','re75','re78','u74_source','u75_source']
for i,(source_row,prepared_row) in enumerate(zip(raw,prepared),1):
  assert prepared_row['source_row']==str(i)
  for field,value in zip(fields,source_row): assert float(prepared_row[field])==float(value)
  for post in (0,1):
    actual,expected=native[2*(i-1)+post],ref[2*(i-1)+post]
    assert actual['source_row']==expected['source_row']==str(i)
    assert actual['post']==expected['post']==str(post)
    assert float(actual['treat'])==float(expected['treat'])==float(source_row[0])
    assert float(actual['earnings'])==float(expected['earnings'])==float(source_row[7+post])
    assert float(actual['interaction'])==float(expected['interaction'])==float(source_row[0])*post
    near(actual['fitted_earnings'],expected['fitted_earnings'],f'{i}.{post}.fitted')
leaves=[json.loads(line) for line in result.read_text().splitlines()]
fits=[v['data'] for v in leaves if v['result_type']=='RegressionResult']
lincoms=[v['data'] for v in leaves if v['result_type']=='LincomResult']
assert len(fits)==len(lincoms)==2
coefficients=rows(reference/'coefficients.csv'); covariances=rows(reference/'covariances.csv')
for fit,lincom,model,target,n,g in zip(fits,lincoms,['before_after','diff_in_diff'],['post','interaction'],[370,5350],[185,2675]):
  assert fit['covariance']=='cluster(source_row)' and fit['observation_count']==n
  reference_coefficients=[r for r in coefficients if r['model']==model]
  assert len(reference_coefficients)==len(fit['coefficients'])
  for c,r in zip(fit['coefficients'],reference_coefficients):
    assert c['name']==('intercept' if r['parameter']=='(Intercept)' else r['parameter'])
    assert int(r['n'])==n and int(r['groups'])==g
    for key,refkey in [('value','value'),('standard_error','se'),('statistic','statistic'),('p_value','p_value')]:
      near(c[key],r[refkey],model+'.'+key)
  names=[c['name'] for c in fit['coefficients']]
  for r in (r for r in covariances if r['model']==model):
    idx=lambda s:names.index('intercept' if s=='(Intercept)' else s)
    near(fit['covariance_matrix'][idx(r['row'])][idx(r['column'])],r['value'],model+'.cov')
  r=next(r for r in reference_coefficients if r['parameter']==target)
  assert lincom['label']==target and lincom['df_residual']==g-1
  for key,refkey in [('estimate','value'),('standard_error','se'),('statistic','statistic'),('p_value','p_value'),('ci_lower','lower'),('ci_upper','upper')]:
    near(lincom[key],r[refkey],model+'.'+key)
table,=[v['data'] for v in leaves if v['result_type']=='EstimateTableResult']
assert [m['name'] for m in table['models']]==['before_after','diff_in_diff']
for model,fit,g in zip(table['models'],fits,[185,2675]):
  indices=[j for j,r in enumerate(native) if model['name']=='diff_in_diff' or r['treat']=='1']
  assert model['sample']['retained_indices']==indices and model['cluster_count']==g
  assert model['inference_degrees_of_freedom']==g-1 and model['covariance_matrix']==fit['covariance_matrix']
  for a,b in zip(model['coefficients'],fit['coefficients']):
    for key in ('name','value','standard_error','statistic','p_value'): assert a[key]==b[key]
summary,=[v['data'] for v in leaves if v['result_type']=='TableResult' and 'mean_earnings' in v['data']['headers']]
assert len(summary['rows'])==4
for row,(group,post) in zip(summary['rows'],[(0,0),(0,1),(1,0),(1,1)]):
  earnings=[float(r[7+post]) for r in raw if int(r[0])==group]
  assert row[:3]==[group,post,len(earnings)]
  near(row[3],math.fsum(earnings)/len(earnings),'group mean')
print('PASS: all 2675 source people/5350 ordered records, clustered coefficients, covariance, inference and predictions')
