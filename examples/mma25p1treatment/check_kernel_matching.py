#!/usr/bin/env python3
"""Audit fixed-bandwidth kernel matching and estimated-score sandwich against R."""
import csv
import hashlib
import json
import math
from pathlib import Path
import sys

if len(sys.argv)!=5:
  raise SystemExit('usage: check_kernel_matching.py nswpsid.da1 kernel_matching_result.jsonl kernel_matching_rows.csv reference-directory')
source,result,output,reference=map(Path,sys.argv[1:])
assert hashlib.sha256(source.read_bytes()).hexdigest()=='810f6dbfbe0ffbcc52676946c330f49d4d8bf5cafc06980e912ca945ac7f4e44'
def rows(path):
  with path.open(newline='') as stream: return list(csv.DictReader(stream))
def near(a,b,label):
  a,b=float(a),float(b)
  assert math.isfinite(a) and math.isfinite(b) and abs(a-b)<=1e-8+1e-9*abs(b),(label,a,b)
  if label.endswith('pscore') and 0<abs(b)<1e-8: assert abs(a/b-1)<1e-7,(label,a,b)
raw=[line.split() for line in source.read_text().splitlines() if line.strip()]
prepared=rows(result.parent/'nswpsid.csv'); actual=rows(output); expected=rows(reference/'rows.csv')
assert len(raw)==len(prepared)==len(actual)==len(expected)==2675
fields=['treat','age','educ','black','hisp','marr','re74','re75','re78','u74_source','u75_source']
for i,(r,p,a,e) in enumerate(zip(raw,prepared,actual,expected),1):
  assert p['source_row']==a['source_row']==e['source_row']==str(i)
  for field,value in zip(fields,r): assert float(p[field])==float(value)
  assert float(a['treat'])==float(e['treat'])==float(r[0]) and float(a['re78'])==float(e['re78'])==float(r[8])
  for field in ('pscore','matched_mean','nearest_distance','match_ess','reuse_weight','direct_influence','score_numerator','nuisance_multiplier','influence'):
    if e[field]: near(a[field],e[field],f'{i}.{field}')
    else: assert a[field]==''
leaves=[json.loads(line) for line in result.read_text().splitlines()]
fit,=[v['data'] for v in leaves if v['result_type']=='BinaryLogitResult']
assert fit['converged'] and fit['observation_count']==2675 and fit['sample']['retained_indices']==list(range(2675))
solve,=[v['data'] for v in leaves if v['result_type']=='RegressionResult']
assert solve['observation_count']==2675 and len(solve['coefficients'])==14
for c,r in zip(solve['coefficients'],rows(reference/'gradient.csv')):
  assert c['name']==('intercept' if r['parameter']=='(Intercept)' else r['parameter'])
  near(c['value'],r['multiplier'],c['name']+'.multiplier')
summary,=[v['data'] for v in leaves if v['result_type']=='TableResult' and 'kernel_contrast' in v['data']['headers']]
row=dict(zip(summary['headers'],summary['rows'][0])); ref,=rows(reference/'summary.csv')
assert row['observations']==2675 and row['treated_people']==185
for key in ref: near(row[key],ref[key],key)
print('PASS: 2675 source rows, kernel matches, reused controls, score derivatives and sandwich inference')
