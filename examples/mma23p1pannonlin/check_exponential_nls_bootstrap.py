#!/usr/bin/env python3
"""Audit source, native NLS and whole-firm resamples against independent R/Python."""
import csv
import hashlib
import itertools
import json
import math
from pathlib import Path
import statistics
import sys

if len(sys.argv) not in (5,6):
  raise SystemExit('usage: check_exponential_nls_bootstrap.py work-dir oracle.csv reference-dir result.jsonl [original-source.asc]')
work,oracle,refdir,output=map(Path,sys.argv[1:5])
if len(sys.argv)==6:
  if hashlib.sha256(Path(sys.argv[5]).read_bytes()).hexdigest()!='87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8':
    raise AssertionError('original-source SHA-256 mismatch')
with (refdir/'reference.csv').open(newline='') as f:
  ref={row['field']:float(row['value']) for row in csv.DictReader(f)}
with (refdir/'fits.csv').open(newline='') as f:
  fits={int(row['replicate']):(float(row['intercept']),float(row['slope'])) for row in csv.DictReader(f)}
with (refdir/'predictions.csv').open(newline='') as f:
  expected={(int(row['id']),int(row['year'])):row for row in csv.DictReader(f)}

def close(actual,expected,label):
  actual=float(actual); expected=float(expected); tol=1e-9+1e-10*abs(expected)
  if not math.isfinite(actual) or abs(actual-expected)>tol:
    raise AssertionError(f'{label}: {actual} != {expected} (tolerance {tol})')

with (work/'patents.csv').open(newline='') as f:
  source=list(csv.DictReader(f))
if len(source)!=int(ref['candidate_rows']) or len(expected)!=len(source): raise AssertionError('source count mismatch')
ordered=[];firms=sorted({int(row['id']) for row in source})
for row in source:
  key=(int(row['id']),int(row['year'])); reference=expected.pop(key)
  for name in ('id','year','logr','pat'):
    if float(row[name])!=float(reference[name]): raise AssertionError('source/design mismatch: '+name)
  ordered.append(reference)
if expected or len(firms)!=int(ref['firms_per_draw']): raise AssertionError('source design/firm mismatch')
checked=0
with oracle.open(newline='') as f,(work/'exponential_nls_bootstrap_draws.csv').open(newline='') as g:
  for exp,row in itertools.zip_longest(csv.DictReader(f),csv.DictReader(g)):
    if exp is None or row is None: raise AssertionError('draw length mismatch')
    rep=int(exp['replicate']);position=int(exp['position'])
    for key in ('replicate','position','step'):
      if int(row[key])!=int(exp[key]): raise AssertionError(f'draw {rep}:{position} {key} mismatch')
    if int(row['firm_id'])!=firms[int(exp['index0'])]: raise AssertionError(f'draw {rep}:{position} firm mismatch')
    close(row['replicate_intercept'],fits[rep][0],f'replicate {rep} intercept')
    close(row['replicate_slope'],fits[rep][1],f'replicate {rep} slope')
    close(row['estimate_a'],ref['estimate_intercept'],'original intercept')
    close(row['estimate_b'],ref['estimate_slope'],'original slope')
    for key in ('candidate_rows','firms_per_draw','replications'): close(row[key],ref[key],key)
    checked+=1
if checked!=int(ref['firms_per_draw']*ref['replications']) or len(fits)!=500: raise AssertionError('wrong draw/refit count')
report=native=summary=None
with output.open() as f:
  for line in f:
    event=json.loads(line)
    if event.get('result_type')=='NlsResult': native=event['data']
    if event.get('result_type')=='SummarizeResult': summary=event['data']['rows']
    if event.get('result_type')=='TableResult' and 'covariance_aa' in event['data']['headers']:
      data=event['data']; assert len(data['rows'])==1
      report=dict(zip(data['headers'],data['rows'][0]))
if report is None or native is None or summary is None: raise AssertionError('native NLS/summary/bootstrap report missing')
for key,value in ref.items(): close(report[key],value,'report '+key)
assert report['base_seed']==10001 and report['method']=='whole_firm_pairs'
assert report['interval_method']=='type7_percentile_95'
assert report['rng']=='xoshiro256++/splitmix64/per_replicate_seed_plus_index/u64_rejection_v1'
assert native['observation_count']==len(source) and native['sample']['dropped_observations']==0
assert native['sample']['retained_indices']==list(range(len(source)))
for name,key in [('a','estimate_intercept'),('b','estimate_slope')]:
  coefficient=next(c for c in native['coefficients'] if c['name']==name)
  close(coefficient['value'],ref[key],'native '+name)
for prediction,kind in [('fitted_values','mu'),('residuals','residual')]:
  actual=native[prediction]
  if len(actual)!=len(ordered): raise AssertionError('native prediction length')
  for row,value in zip(ordered,actual): close(value,row[kind],'native '+kind)
for row in summary:
  kind={'nls_mu':'mu','nls_residual':'residual'}[row['variable']]
  values=[float(r[kind]) for r in ordered]
  for key,expect in [('count',len(values)),('minimum',min(values)),('maximum',max(values)),
                     ('mean',statistics.mean(values)),('std_dev',statistics.stdev(values))]:
    close(row[key],expect,'native summary '+kind+' '+key)
if len(summary)!=2: raise AssertionError('missing prediction summary')
if len(sys.argv)==6:
  for key,value,tol in [('estimate_slope',.5084673,5e-8),('estimate_intercept',2.528729,5e-7)]:
    if abs(report[key]-value)>tol: raise AssertionError('author log '+key)
print(f'PASS: {len(source)} exact source/design and native mean/residual rows, {checked} exact ordered draws with both beta cells, 500 independent R refits, {len(ref)} summaries')
