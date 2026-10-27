#!/usr/bin/env python3
"""Audit the source stratification against base R and the historical printed output."""
import csv
import hashlib
import json
import math
from pathlib import Path
import re
import sys

if len(sys.argv)!=6:
  raise SystemExit('usage: check_stratified.py nswpsid.da1 author-output.txt stratified_result.jsonl stratified_records.csv reference-directory')
source,author,result,output,reference=map(Path,sys.argv[1:])
assert hashlib.sha256(source.read_bytes()).hexdigest()=='810f6dbfbe0ffbcc52676946c330f49d4d8bf5cafc06980e912ca945ac7f4e44'
assert hashlib.sha256(author.read_bytes()).hexdigest()=='f06c09769fd8812061955d67dca78665fbdae68592186efd41003723f9985bdd'
def rows(path):
  with path.open(newline='') as stream: return list(csv.DictReader(stream))
def near(a,b,label):
  a,b=float(a),float(b)
  assert math.isfinite(a) and math.isfinite(b) and abs(a-b)<=1e-8+1e-9*abs(b),(label,a,b)
  if label.endswith('pscore') and 0<abs(b)<1e-8: assert abs(a/b-1)<1e-7,(label,a,b)
raw=[line.split() for line in source.read_text().splitlines() if line.strip()]
prepared=rows(result.parent/'nswpsid.csv'); actual=rows(output); expected=rows(reference/'rows.csv'); cells=rows(reference/'cells.csv')
assert len(raw)==len(prepared)==len(actual)==len(expected)==2675 and len(cells)==10
fields=['treat','age','educ','black','hisp','marr','re74','re75','re78','u74_source','u75_source']
for i,(r,p,a,e) in enumerate(zip(raw,prepared,actual,expected),1):
  assert p['source_row']==a['source_row']==e['source_row']==str(i)
  for field,value in zip(fields,r): assert float(p[field])==float(value)
  assert float(a['treat'])==float(e['treat'])==float(r[0]) and float(a['re78'])==float(e['re78'])==float(r[8])
  for field in ('pscore','lower_bound','upper_bound'): near(a[field],e[field],f'{i}.{field}')
  assert a['in_overlap'].lower()==e['in_overlap'].lower() and a['stratum']==e['stratum']
  if not a['stratum']:
    assert all(a[k]=='' for k in ('treated_n','control_n','treated_mean','control_mean','difference','status'))
  else:
    c=cells[int(a['stratum'])-1]
    for key in ('treated_n','control_n','status'): assert a[key]==c[key]
    for key in ('treated_mean','control_mean','difference'):
      if c[key]: near(a[key],c[key],f'{i}.{key}')
      else: assert a[key]==''
leaves=[json.loads(line) for line in result.read_text().splitlines()]
fit,=[v['data'] for v in leaves if v['result_type']=='BinaryLogitResult']
assert fit['converged'] and fit['observation_count']==2675 and fit['sample']['retained_indices']==list(range(2675))
summary,=[v['data'] for v in leaves if v['result_type']=='TableResult' and 'treated_weighted_difference' in v['data']['headers']]
summary_row=dict(zip(summary['headers'],summary['rows'][0])); r,=rows(reference/'summary.csv')
for key in r: near(summary_row[key],r[key],key)
assert summary_row['standard_error'] is None
assert [int(r[k]) for k in ('common_range_treated','supported_treated','unsupported_treated','supported_strata')]==[179,166,13,9]
table,=[v['data'] for v in leaves if v['result_type']=='TableResult' and 'difference' in v['data']['headers'] and 'status' in v['data']['headers']]
assert len(table['rows'])==10
for values,c in zip(table['rows'],cells):
  for key,value in zip(table['headers'],values):
    if key=='status': assert value==c[key]
    elif c[key]: near(value,c[key],f'cell{c["stratum"]}.{key}')
    else: assert value is None
# The printed Stata float results are rounded here to half of one thousandth.
printed={int(i):float(value) for i,value in re.findall(r'^\s*(\d+) estimate =\s*([-+0-9.eE]+)',author.read_text(),re.MULTILINE)}
assert len(printed)==10
for c in cells:
  i=int(c['stratum'])
  if c['status']=='supported': assert abs(float(c['difference'])-printed[i])<0.0005,(i,c['difference'],printed[i])
  else: assert i==9 and printed[i]==0
for r in rows(reference/'adjusted.csv'):
  i=int(r['stratum']); assert abs(float(r['adjusted_treatment'])-printed[i])>100
assert abs(summary_row['treated_weighted_difference']-1562.7274)<0.0005
print('PASS: all 2675 source rows, score assignments, ten cells, missing inference and nine historical contrasts')
