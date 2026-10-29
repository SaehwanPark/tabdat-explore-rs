#!/usr/bin/env python3
"""Audit NLS predictions, firm scores and both covariance conventions against R."""
import collections
import csv
import hashlib
import itertools
import json
import math
from pathlib import Path
import sys

if len(sys.argv) not in (3,4):
  raise SystemExit('usage: check_exponential_nls.py work-dir reference-dir [original-source.asc]')
work,refdir=map(Path,sys.argv[1:3])
if len(sys.argv)==4:
  digest=hashlib.sha256(Path(sys.argv[3]).read_bytes()).hexdigest()
  if digest!='87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8':
    raise AssertionError('original-source SHA-256 mismatch')
with (refdir/'reference.csv').open(newline='') as f:
  ref={row['field']:float(row['value']) for row in csv.DictReader(f)}

def close(value,expected,label):
  value=float(value); expected=float(expected)
  tolerance=1e-9+1e-10*abs(expected)
  if not math.isfinite(value) or abs(value-expected)>tolerance:
    raise AssertionError(f'{label}: {value} != {expected}, tolerance {tolerance}')

report=nls=None
with (work/'result.jsonl').open() as f:
  for line in f:
    event=json.loads(line)
    if event['result_type']=='NlsResult': nls=event['data']
    if event['result_type']=='TableResult' and 'cluster_aa' in event['data']['headers']:
      data=event['data']; assert len(data['rows'])==1
      report=dict(zip(data['headers'],data['rows'][0]))
if nls is None or report is None: raise AssertionError('native NLS or SQL covariance report missing')
for field,value in ref.items():
  if not field.startswith(('native_','score_scale_')): close(report[field],value,'report '+field)
assert report['working_covariance']=='rss_over_n_working_gauss_newton'
assert report['cluster_covariance']=='firm_score_gauss_newton_G_over_G_minus_1'
assert report['inference']=='asymptotic_normal; p-values not computed'
for key in ('a','b'):
  coefficient=next(c for c in nls['coefficients'] if c['name']==key)
  suffix='intercept' if key=='a' else 'slope'
  close(coefficient['value'],ref[suffix],'native '+key)
  close(coefficient['standard_error'],ref['native_se_'+suffix],'native '+key+' se')
assert nls['observation_count']==int(ref['observations'])
assert nls['sample']['dropped_observations']==0
assert nls['covariance']=='working_homoskedastic_gauss_newton'
close(nls['residual_sum_of_squares'],ref['rss'],'native rss')
for key in ('a','b'):
  if abs(report['score_'+key])>1e-11*(1+ref['score_scale_'+key]):
    raise AssertionError('unconverged score '+key)

scores=collections.defaultdict(lambda:[0.,0.,0.,0.])
rows=0
with (work/'patents.csv').open(newline='') as f,(work/'exponential_nls_predictions.csv').open(newline='') as g,(refdir/'predictions.csv').open(newline='') as h:
  for src,out,expected in itertools.zip_longest(csv.DictReader(f),csv.DictReader(g),csv.DictReader(h)):
    if src is None or out is None or expected is None: raise AssertionError('source/export/reference row count mismatch')
    for key,value in src.items():
      if value=='':
        if out[key]!='': raise AssertionError('missing source field changed: '+key)
      elif float(value)!=float(out[key]): raise AssertionError('source field changed: '+key)
    for key in ('id','year','logr','pat'):
      if float(out[key])!=float(expected[key]): raise AssertionError('R source design mismatch: '+key)
    for key in ('nls_mu','nls_residual'): close(out[key],expected[key],'prediction '+key)
    mu=float(out['nls_mu']); residual=float(out['nls_residual']); x=float(out['logr'])
    reference_mu=float(expected['nls_mu']); reference_res=float(expected['nls_residual'])
    emu=1e-9+1e-10*abs(reference_mu); eres=1e-9+1e-10*abs(reference_res)
    bound=abs(reference_res)*emu+abs(reference_mu)*eres+emu*eres
    unit=scores[int(out['id'])]; unit[0]+=mu*residual; unit[1]+=mu*residual*x
    unit[2]+=bound; unit[3]+=bound*abs(x); rows+=1
with (refdir/'scores.csv').open(newline='') as f:
  score_rows=list(csv.DictReader(f))
for expected in score_rows:
  unit=scores[int(expected['id'])]
  for i,key in enumerate(('score_a','score_b')):
    if abs(unit[i]-float(expected[key]))>1e-9+unit[i+2]:
      raise AssertionError(f'firm {expected["id"]} {key} outside propagated prediction tolerance')
if rows!=int(ref['observations']) or len(scores)!=int(ref['firms']) or len(score_rows)!=len(scores):
  raise AssertionError('sample or firm count mismatch')
if len(sys.argv)==4:
  for field,value,tol in [('intercept',2.528729,5e-7),('slope',.5084673,5e-8),
    ('rss',3565052.8,.05),('working_se_intercept',.0544558,5e-8),
    ('working_se_slope',.0105636,5e-8),('cluster_se_intercept',.2176674,5e-8),
    ('cluster_se_slope',.0550460,5e-8)]:
    if abs(report[field]-value)>tol: raise AssertionError('author log '+field)
print(f'PASS: {rows} source/prediction rows, {len(scores)} firm scores, native NLS and 27 R summary fields')
