#!/usr/bin/env python3
"""Audit nearest-score assignments against an independent base-R search."""
import csv
import hashlib
import json
import math
from pathlib import Path
import sys

if len(sys.argv)!=5:
  raise SystemExit('usage: check_nearest_matching.py nswpsid.da1 nearest_result.jsonl nearest_pairs.csv reference-directory')
source,result,output,reference=map(Path,sys.argv[1:])
assert hashlib.sha256(source.read_bytes()).hexdigest()=='810f6dbfbe0ffbcc52676946c330f49d4d8bf5cafc06980e912ca945ac7f4e44'
def rows(path):
  with path.open(newline='') as stream: return list(csv.DictReader(stream))
def near(a,b,label):
  a,b=float(a),float(b)
  assert math.isfinite(a) and math.isfinite(b) and abs(a-b)<=1e-8+1e-9*abs(b),(label,a,b)
  if label.endswith('score') and 0<abs(b)<1e-8: assert abs(a/b-1)<1e-7,(label,a,b)
raw=[line.split() for line in source.read_text().splitlines() if line.strip()]
prepared=rows(result.parent/'nswpsid.csv'); actual=rows(output); expected=rows(reference/'pairs.csv')
assert len(raw)==len(prepared)==2675 and len(actual)==len(expected)>0
fields=['treat','age','educ','black','hisp','marr','re74','re75','re78','u74_source','u75_source']
for i,(r,p) in enumerate(zip(raw,prepared),1):
  assert p['source_row']==str(i)
  for field,value in zip(fields,r): assert float(p[field])==float(value)
for a,e in zip(actual,expected):
  for key in ('treated_id','control_id','tied_controls'): assert a[key]==e[key]
  ti,ci=int(a['treated_id'])-1,int(a['control_id'])-1
  assert int(raw[ti][0])==1 and int(raw[ci][0])==0
  assert float(a['treated_outcome'])==float(raw[ti][8]) and float(a['control_outcome'])==float(raw[ci][8])
  for key in ('treated_score','control_score','distance','nearest_distance','matched_outcome'):
    near(a[key],e[key],f'pair{ti+1}.{key}')
leaves=[json.loads(line) for line in result.read_text().splitlines()]
fit,=[v['data'] for v in leaves if v['result_type']=='BinaryLogitResult']
assert fit['converged'] and fit['observation_count']==2675 and fit['sample']['retained_indices']==list(range(2675))
summary,=[v['data'] for v in leaves if v['result_type']=='TableResult' and 'matched_treated' in v['data']['headers']]
row=dict(zip(summary['headers'],summary['rows'][0])); ref,=rows(reference/'summary.csv')
for key in ref: near(row[key],ref[key],key)
assert row['standard_error'] is None
assert row['matched_treated']==179 and row['distinct_controls']==57 and row['tied_treated']==3
print('PASS: all original source rows and nearest pairs, exact ties, matched outcomes, reuse and point comparison')
