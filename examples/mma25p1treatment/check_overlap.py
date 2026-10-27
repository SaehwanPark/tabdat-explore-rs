#!/usr/bin/env python3
"""Audit original score model, range, strata, and balance against independent R."""
import csv
import hashlib
import json
import math
from pathlib import Path
import sys

if len(sys.argv)!=5:
  raise SystemExit('usage: check_overlap.py nswpsid.da1 overlap_result.jsonl propensity_scores.csv reference-directory')
source, result, output, reference=map(Path,sys.argv[1:])
assert hashlib.sha256(source.read_bytes()).hexdigest()=='810f6dbfbe0ffbcc52676946c330f49d4d8bf5cafc06980e912ca945ac7f4e44'
def rows(path):
  with path.open(newline='') as stream:
    return list(csv.DictReader(stream))
def near(a,b,label):
  a,b=float(a),float(b)
  assert math.isfinite(a) and math.isfinite(b) and abs(a-b)<=1e-10+1e-9*abs(b),(label,a,b)
  if (label.endswith('p_value') and 0<abs(b)<1e-20) or (label.endswith('pscore') and 0<abs(b)<1e-8):
    assert abs(a/b-1)<1e-7,(label,a,b)
raw=[line.split() for line in source.read_text().splitlines() if line.strip()]
prepared=rows(result.parent/'nswpsid.csv'); actual=rows(output); expected=rows(reference/'scores.csv')
assert len(raw)==len(prepared)==len(actual)==len(expected)==2675
fields=['treat','age','educ','black','hisp','marr','re74','re75','re78','u74_source','u75_source']
for i,(raw_row,inp,n,r) in enumerate(zip(raw,prepared,actual,expected),1):
  assert inp['source_row']==n['source_row']==r['source_row']==str(i)
  for field,value in zip(fields,raw_row):
    assert float(inp[field])==float(value)
    ref_field={'u74_source':'u74','u75_source':'u75'}.get(field,field)
    assert float(r[ref_field])==float(value)
    if field in n: assert float(n[field])==float(value)
  assert n['nodegree']==r['nodegree'] and n['u74black']==r['u74black']
  for field in ('pscore','lower_bound','upper_bound'): near(n[field],r[field],f'{i}.{field}')
  assert n['in_overlap'].lower()==r['in_overlap'].lower()
  assert n['stratum']==r['stratum']
leaves=[json.loads(line) for line in result.read_text().splitlines()]
fit,=[v['data'] for v in leaves if v['result_type']=='BinaryLogitResult']
assert fit['converged'] and fit['observation_count']==2675 and fit['event_count']==185
assert fit['sample']['retained_indices']==list(range(2675)) and fit['reference_distribution']=='asymptotic_standard_normal'
coeff=rows(reference/'coefficients.csv'); cov=rows(reference/'covariances.csv')
assert len(coeff)==len(fit['coefficients'])==14
for c,r in zip(fit['coefficients'],coeff):
  assert c['name']==('intercept' if r['parameter']=='(Intercept)' else r['parameter'])
  for field,ref_field in [('value','value'),('standard_error','se'),('statistic','statistic'),('p_value','p_value'),('confidence_lower','lower'),('confidence_upper','upper')]:
    near(c[field],r[ref_field],c['name']+'.'+field)
names=[c['name'] for c in fit['coefficients']]
for r in cov:
  idx=lambda s:names.index('intercept' if s=='(Intercept)' else s)
  near(fit['covariance_matrix'][idx(r['row'])][idx(r['column'])],r['value'],'covariance')
summary,=[v['data'] for v in leaves if v['result_type']=='TableResult' and 'common_lower' in v['data']['headers']]
for group,row in enumerate(summary['rows']):
  subset=[r for r in expected if int(r['treat'])==group]
  assert row[:4]==[group,len(subset),sum(r['in_overlap']=='TRUE' for r in subset),sum(r['in_overlap']=='FALSE' for r in subset)]
  for index,value in [(4,min(float(r['pscore']) for r in subset)),(5,max(float(r['pscore']) for r in subset)),(6,float(expected[0]['lower_bound'])),(7,float(expected[0]['upper_bound']))]:
    near(row[index],value,'summary')
assert [r[2] for r in summary['rows']]==[1146,179]
balance,=[v['data'] for v in leaves if v['result_type']=='TableResult' and 'u74black' in v['data']['headers']]
assert len(balance['rows'])==20
for row in balance['rows']:
  stratum,group,count=row[:3]
  subset=[r for r in expected if r['stratum']==str(stratum) and int(r['treat'])==group]
  assert count==len(subset)
  for j,field in enumerate(['age','educ','nodegree','black','hisp','marr','re74','re75','u74black'],3):
    if subset: near(row[j],math.fsum(float(r[field]) for r in subset)/len(subset),f'{stratum}.{group}.{field}')
    else: assert row[j] is None
print('PASS: 2675 ordered source scores, full logit covariance, range exclusions and all 20 balance cells')
