#!/usr/bin/env python3
"""Audit original NLS sample, projections and model-variance sandwich against R."""
import csv
import hashlib
import json
import math
from pathlib import Path
import sys

if len(sys.argv) != 5:
  raise SystemExit('usage: check_model_variance.py original.asc result.jsonl nls_modelvar.csv reference-directory')
source, output, native_path, reference = map(Path, sys.argv[1:])
assert hashlib.sha256(source.read_bytes()).hexdigest() == '460f5d731f1848c8695d78a1fbf0db348d0daced4b8b5b0c2734bffa20b3e31c'
raw = [line.split() for line in source.read_text().splitlines()]
with (output.parent/'nls.csv').open(newline='') as stream:
  prepared = list(csv.DictReader(stream))
with (reference/'predictions.csv').open(newline='') as stream:
  projections = list(csv.DictReader(stream))
with (reference/'covariance.csv').open(newline='') as stream:
  expected, = list(csv.DictReader(stream))
with native_path.open(newline='') as stream:
  actual, = list(csv.DictReader(stream))
results = [json.loads(line) for line in output.read_text().splitlines()]
fit, = [r['data'] for r in results if r['result_type']=='NlsResult']
assert len(raw)==len(prepared)==len(projections)==10000
assert fit['sample']['retained_indices']==list(range(10000))
assert fit['sample']['dropped_observations']==0 and fit['observation_count']==10000
assert [c['name'] for c in fit['coefficients']]==['a','b']
for c, ref in zip(fit['coefficients'], [1.8875622977112239,-0.95746831129837129]):
  assert abs(c['value']-ref)<1e-11
assert len(fit['fitted_values'])==len(fit['residuals'])==10000

def near(value, target, context):
  assert math.isfinite(value) and math.isfinite(target), context
  assert abs(value-target)<=1e-11+1e-10*abs(target), (context,value,target)

for i,(tokens,row,proj,mu,resid) in enumerate(zip(raw,prepared,projections,fit['fitted_values'],fit['residuals']),1):
  assert len(tokens)==2 and int(row['source_row'])==int(proj['source_row'])==i
  assert float(row['y'])==float(tokens[0])==float(proj['y'])
  assert float(row['x'])==float(tokens[1])==float(proj['x'])
  near(mu,float(proj['mean']),f'row{i}.mean')
  near(resid,float(proj['residual']),f'row{i}.residual')
assert actual.keys()==expected.keys() and actual['n']==expected['n']=='10000'
assert actual['inference']==expected['inference']=='variance-model plug-in, no HC correction; parameter order a,b'
for key in actual.keys()-{'n','inference'}:
  near(float(actual[key]),float(expected[key]),key)
# Author prints eight decimals from a historical Stata fit; agreement at 4 decimals only.
assert abs(float(actual['se_a'])-.2110)<0.00005
assert abs(float(actual['se_b'])-.0880)<0.00005
print('validated 10000 ordered source rows and mean/residual pairs, matrices and plug-in covariance against R')
