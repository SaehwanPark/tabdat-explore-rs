#!/usr/bin/env python3
"""Audit native Poisson and whole-firm bootstrap against independent R/Python."""
import csv
import hashlib
import itertools
import json
import math
from pathlib import Path
import sys

if len(sys.argv) not in (5,6):
  raise SystemExit('usage: check_pooled_bootstrap.py work-dir oracle.csv reference-dir result.jsonl [original-source.asc]')
work,oracle,refdir,output=map(Path,sys.argv[1:5])
if len(sys.argv)==6:
  if hashlib.sha256(Path(sys.argv[5]).read_bytes()).hexdigest()!='87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8':
    raise AssertionError('original-source SHA-256 mismatch')
with (refdir/'reference.csv').open(newline='') as f:
  ref={row['field']:float(row['value']) for row in csv.DictReader(f)}
with (refdir/'replicates.csv').open(newline='') as f:
  reps={int(row['replicate']):(float(row['intercept']),float(row['slope'])) for row in csv.DictReader(f)}
with (refdir/'groups.csv').open(newline='') as f:
  groups={int(row['index0']):int(row['firm_id']) for row in csv.DictReader(f)}
with (refdir/'prediction_summary.csv').open(newline='') as f:
  predictions={row['variable']:row for row in csv.DictReader(f)}

def close(actual,expected,label):
  actual=float(actual); expected=float(expected)
  if not math.isfinite(actual) or abs(actual-expected)>1e-10:
    raise AssertionError(f'{label}: {actual} != {expected} (absolute tolerance 1e-10)')

rows=0
with (work/'patents.csv').open(newline='') as f,(refdir/'design.csv').open(newline='') as g:
  for source,expected in itertools.zip_longest(csv.DictReader(f),csv.DictReader(g)):
    if source is None or expected is None: raise AssertionError('source/design length mismatch')
    for key in ('id','year','logr','pat'):
      if float(source[key])!=float(expected[key]): raise AssertionError('source mismatch: '+key)
    rows+=1
if rows!=int(ref['candidate_rows']): raise AssertionError('wrong source row count')
checked=0
with oracle.open(newline='') as f,(work/'pooled_poisson_bootstrap_draws.csv').open(newline='') as g:
  expected_rows=csv.DictReader(f); actual_rows=csv.DictReader(g)
  for expected in expected_rows:
    row=next(actual_rows,None)
    if row is None: raise AssertionError('missing draw row')
    r=int(expected['replicate']); pos=int(expected['position'])
    for key in ('replicate','position','step'):
      if int(row[key])!=int(expected[key]): raise AssertionError(f'draw {r}:{pos} {key} mismatch')
    if int(row['firm_id'])!=groups[int(expected['index0'])]: raise AssertionError(f'draw {r}:{pos} firm mismatch')
    close(row['replicate_intercept'],reps[r][0],f'replicate {r} intercept')
    close(row['replicate_slope'],reps[r][1],f'replicate {r} slope')
    close(row['estimate_a'],ref['estimate_intercept'],'original intercept')
    close(row['estimate_b'],ref['estimate_slope'],'original slope')
    for key in ('candidate_rows','firms_per_draw','replications'): close(row[key],ref[key],key)
    checked+=1
  if next(actual_rows,None) is not None: raise AssertionError('extra draw rows')
if checked!=int(ref['firms_per_draw']*ref['replications']): raise AssertionError('wrong draw count')
report=native=summary=None
with output.open() as f:
  for line in f:
    event=json.loads(line)
    if event.get('result_type')=='PoissonResult': native=event['data']
    if event.get('result_type')=='SummarizeResult': summary=event['data']['rows']
    if event.get('result_type')=='TableResult' and 'covariance_aa' in event['data']['headers']:
      data=event['data']; assert len(data['rows'])==1
      report=dict(zip(data['headers'],data['rows'][0]))
if report is None or native is None or summary is None: raise AssertionError('Poisson/summary/bootstrap report missing')
for key,value in ref.items(): close(report[key],value,'report '+key)
assert report['base_seed']==10001 and report['method']=='whole_firm_pairs'
assert report['interval_method']=='type7_percentile_95'
assert report['rng']=='xoshiro256++/splitmix64/per_replicate_seed_plus_index/u64_rejection_v1'
assert native['observation_count']==int(ref['candidate_rows']) and native['converged']
assert native['sample']['dropped_observations']==0
for name,key in [('intercept','estimate_intercept'),('logr','estimate_slope')]:
  coefficient=next(c for c in native['coefficients'] if c['name']==name)
  close(coefficient['value'],ref[key],'native '+name)
for row in summary:
  expected=predictions.pop(row['variable'])
  for key in ('count','minimum','maximum','mean','std_dev'): close(row[key],expected[key],'prediction '+row['variable']+' '+key)
if predictions: raise AssertionError('missing native prediction summary')
if len(sys.argv)==6:
  for key,value,tol in [('estimate_slope',.6929337,5e-8),('estimate_intercept',1.711528,5e-7)]:
    if abs(report[key]-value)>tol: raise AssertionError('author log '+key)
print(f'PASS: {rows} source/design rows, {checked} exact ordered draws and both beta cells, 500 R refits, {len(ref)} summaries and native fit/prediction summaries')
