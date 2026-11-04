#!/usr/bin/env python3
"""Original-source, independent R dummy-GLM and conditional FE audit."""
import csv
from decimal import Decimal
import hashlib
import json
import math
from pathlib import Path
import sys

if len(sys.argv)!=4:
  raise SystemExit('usage: check_conditional_fe.py native-dir reference-dir patr7079.asc')
native,reference,raw=map(Path,sys.argv[1:])
assert hashlib.sha256(raw.read_bytes()).hexdigest()=='87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8'
def records(path):
  with path.open(newline='') as stream: return list(csv.DictReader(stream))
def near(actual,expected,label):
  a,e=float(actual),float(expected); tol=1e-9+1e-10*abs(e)
  assert math.isfinite(a) and math.isfinite(e) and abs(a-e)<=tol,(label,a,e,tol)
tokens=raw.read_text().split(); assert len(tokens)==346*25
source=[tokens[25*i:25*(i+1)] for i in range(346)]
events=[json.loads(line) for line in (native/'result.jsonl').read_text().splitlines()]
assert all(e['schema_version']==1 and 'error' not in e for e in events)
assert [e['result_type'] for e in events]==['LoadResult','IsidResult','TableResult','TableResult',
  'SqlCreateResult','TableResult','SummarizeResult','ExportResult']
table=events[5]['data']; assert len(table['rows'])==1
report=dict(zip(table['headers'],table['rows'][0]))
assert report['estimator']=='conditional_poisson_fixed_effects_one_slope'
assert report['covariance']=='model_based_conditional_information_nonrobust'
assert report['p_reference']=='asymptotic_standard_normal'
ref=records(reference/'reference.csv'); assert len(ref)==14
refs={r['field']:float(r['value']) for r in ref}; assert len(refs)==14
for key,value in refs.items(): near(report[key],value,key)
assert abs(report['score'])<1e-8
assert abs(report['beta']-(-.0377642))<.00000005
assert abs(report['se']-.0325518)<.00000005
assert abs(report['p_value']-.246)<.0005
assert abs(report['conditional_ll']-(-3659.5926))<.00005
actual=records(native/'conditional_fe_predictions.csv'); expected=records(reference/'predictions.csv')
assert len(actual)==len(expected)==1730
fields=['id','year','cusip','ardssic','scisect','logk','sumpat','logr','pat']
labels=['estimator','covariance','p_reference']
fitfields=['beta','retained_firms','conditional_ll','information','se','z','p_value','ci_lower','ci_upper',
  'candidate_rows','candidate_firms','retained_rows']
assert len(actual[0])==28
counts={}; means={}; excluded=set(); checked=0
for i,(a,e) in enumerate(zip(actual,expected)):
  r=source[i//5]; t=i%5
  truth=[str(i//5+1),str(1975+t),r[0],'' if r[1]=='.' else r[1],r[2],r[3],r[4],r[10+t],r[20+t]]
  for key,v in zip(fields,truth):
    if v=='': assert a[key]==e[key]==''
    else: assert Decimal(a[key])==Decimal(e[key])==Decimal(v),(i,key,a[key],e[key],v)
    checked+=1
  included=a['included_in_fe']=='true'
  assert a['included_in_fe'] in ('true','false') and included==(e['included_in_fe']=='TRUE')
  id=int(a['id']); counts[id]=counts.get(id,0)+float(a['pat'])
  if included:
    near(a['fe_mean'],e['fe_mean'],'fe_mean'); near(a['fe_residual'],e['fe_residual'],'fe_residual')
    near(float(a['pat'])-float(a['fe_mean']),a['fe_residual'],'residual_identity')
    means[id]=means.get(id,0)+float(a['fe_mean'])
  else:
    excluded.add(id); assert a['fe_mean']==a['fe_residual']==e['fe_mean']==e['fe_residual']==''
  checked+=3
  for key in labels: assert a[key]==report[key]; checked+=1
  for key in fitfields: near(a[key],refs[key],key); checked+=1
  assert abs(float(a['score']))<1e-8; checked+=1
assert len(excluded)==22 and all(counts[i]==0 for i in excluded)
assert len(means)==324
for id,total in means.items(): near(total,counts[id],'firm_total')
assert checked==1730*28
print('PASS: 14 R dummy-GLM/conditional quantities and 48,440 source/design/prediction/report cells across 1,730 rows; 22 zero-total firms excluded')
