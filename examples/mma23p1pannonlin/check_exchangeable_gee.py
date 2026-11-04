#!/usr/bin/env python3
"""Audit source, GEE report, firm scores and predictions against independent R."""
import collections
import csv
import hashlib
import itertools
import json
import math
from pathlib import Path
import sys

if len(sys.argv) not in (3,4):
  raise SystemExit('usage: check_exchangeable_gee.py work-dir reference-dir [original-source.asc]')
work,refdir=map(Path,sys.argv[1:3])
if len(sys.argv)==4:
  if hashlib.sha256(Path(sys.argv[3]).read_bytes()).hexdigest()!='87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8':
    raise AssertionError('original-source SHA-256 mismatch')
with (refdir/'reference.csv').open(newline='') as f:
  ref={r['field']:float(r['value']) for r in csv.DictReader(f)}
with (refdir/'scores.csv').open(newline='') as f:
  refscores={int(r['id']):(float(r['score_a']),float(r['score_b'])) for r in csv.DictReader(f)}

def close(actual,expected,label):
  actual=float(actual); expected=float(expected)
  tolerance=1e-9+1e-10*abs(expected)
  if not math.isfinite(actual) or abs(actual-expected)>tolerance:
    raise AssertionError(f'{label}: {actual} != {expected}, tolerance {tolerance}')

report=poisson=None
with (work/'result.jsonl').open() as f:
  for line in f:
    event=json.loads(line)
    if event.get('result_type')=='PoissonResult': poisson=event['data']
    if event.get('result_type')=='TableResult' and 'working_aa' in event['data']['headers']:
      data=event['data']; assert len(data['rows'])==1
      report=dict(zip(data['headers'],data['rows'][0]))
if report is None or poisson is None: raise AssertionError('native start or GEE report missing')
for key,value in ref.items(): close(report[key],value,'report '+key)
assert report['correlation']=='exchangeable_pearson_legacy_moment_fixed_scale_1'
assert report['working_covariance']=='gee_working_scale_1'
assert report['cluster_covariance']=='firm_score_G_over_G_minus_1'
assert report['inference']=='asymptotic_normal; p-values not computed'
assert 1<=report['iterations']<=100 and 0<=report['last_step']<=1e-12
assert poisson['converged'] and poisson['observation_count']==int(ref['observations'])
assert poisson['sample']['dropped_observations']==0
with (work/'patents.csv').open(newline='') as f:
  source={(int(r['id']),int(r['year'])):r for r in csv.DictReader(f)}
groups=collections.defaultdict(list); rows=0; prev=None
with (work/'exchangeable_gee_predictions.csv').open(newline='') as f,(refdir/'predictions.csv').open(newline='') as g:
  for out,expected in itertools.zip_longest(csv.DictReader(f),csv.DictReader(g)):
    if out is None or expected is None: raise AssertionError('prediction/reference row count mismatch')
    key=(int(out['id']),int(out['year']))
    if prev is not None and key<=prev: raise AssertionError('export not ordered by id/year')
    prev=key; src=source.pop(key)
    for name,value in src.items():
      if value=='':
        if out[name]!='': raise AssertionError('missing source field changed: '+name)
      elif float(value)!=float(out[name]): raise AssertionError('source field changed: '+name)
    for name in ('id','year','logr','pat'):
      if float(out[name])!=float(expected[name]): raise AssertionError('R source design mismatch: '+name)
    for name in ('gee_mu','gee_residual'): close(out[name],expected[name],'prediction '+name)
    close(out['rho'],ref['rho'],'row rho')
    groups[key[0]].append(out);rows+=1
if source or rows!=int(ref['observations']) or len(groups)!=int(ref['firms']):
  raise AssertionError('sample identity/count mismatch')
rho=float(report['rho']);k=1/(1-rho);c=rho/(1+4*rho); total=[0.,0.]; scales=[0.,0.]
for firm,unit in groups.items():
  if len(unit)!=5: raise AssertionError('firm is not balanced')
  h=[0.,0.];d=[0.,0.];e=0.
  for row in unit:
    x=float(row['logr']);mu=float(row['gee_mu']);residual=float(row['gee_residual'])
    root=math.sqrt(mu);e+=residual/root;h[0]+=root;h[1]+=root*x;d[0]+=residual;d[1]+=x*residual
  for j in range(2):
    score=k*(d[j]-c*h[j]*e); tolerance=1e-8+1e-10*abs(refscores[firm][j])
    if not math.isfinite(score) or abs(score-refscores[firm][j])>tolerance:
      raise AssertionError(f'firm {firm} score {j}: {score} != {refscores[firm][j]}')
    total[j]+=score;scales[j]+=k*(abs(d[j])+abs(c*h[j]*e))
for j,key in enumerate(('score_intercept','score_slope')):
  if abs(report[key])>1e-10*(1+scales[j]): raise AssertionError('GEE score did not converge: '+key)
  if abs(report[key]-total[j])>1e-8+1e-10*scales[j]: raise AssertionError('reported score mismatch: '+key)
if len(sys.argv)==4:
  for key,value,tol in [('intercept',2.067515,5e-7),('working_se_intercept',.0185166,5e-8),
    ('working_se_slope',.0043803,5e-8),('cluster_se_intercept',.1113256,5e-8),('cluster_se_slope',.0326436,5e-8)]:
    if abs(report[key]-value)>tol: raise AssertionError('author log '+key)
  # The seven-decimal historical slope differs: retain and disclose it, not a false parity gate.
  print('historical slope .5595302 minus converged float64:',.5595302-report['slope'])
print(f'PASS: {rows} source/design/prediction rows, {len(groups)} firm scores and {len(ref)} R GEE report fields')
