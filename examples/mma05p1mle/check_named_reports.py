#!/usr/bin/env python3
"""Check named source reports against their fits and the existing R reference."""
import csv
import hashlib
import json
import math
from pathlib import Path
import sys

if len(sys.argv) != 4:
  raise SystemExit('usage: check_named_reports.py original.asc result.jsonl reference-directory')
source, output, reference = map(Path,sys.argv[1:])
assert hashlib.sha256(source.read_bytes()).hexdigest() == '460f5d731f1848c8695d78a1fbf0db348d0daced4b8b5b0c2734bffa20b3e31c'
raw = [line.split() for line in source.read_text().splitlines()]
with (output.parent/'nls.csv').open(newline='') as stream:
  prepared = list(csv.DictReader(stream))
assert len(raw)==len(prepared)==10000
# Independent profile of the intercept-only exponential null, plus the fixed offset constant.
y_values = [float(row[0]) for row in raw]
null_count_ll = len(y_values)*(math.log(len(y_values)/sum(y_values))-1)+sum(map(math.log,y_values))
for i,(tokens,row) in enumerate(zip(raw,prepared),1):
  assert len(tokens)==2 and int(row['source_row'])==i
  assert float(row['y'])==float(tokens[0]) and float(row['x'])==float(tokens[1])
with (reference/'reference.csv').open(newline='') as stream:
  ref = {row['key']:float(row['value']) for row in csv.DictReader(stream)}
rows = [json.loads(line) for line in output.read_text().splitlines()]
fits = [r['data'] for r in rows if r['result_type'] in ('RegressionResult','PoissonResult')]
table, = [r['data'] for r in rows if r['result_type']=='EstimateTableResult']
assert len(fits)==len(table['models'])==4

def near(actual, target, key):
  assert math.isfinite(actual) and math.isfinite(target),key
  if key.endswith('.p') and 0<abs(target)<1e-20:
    assert abs(actual-target)<=1e-8*abs(target),(key,actual,target)
  else:
    assert abs(actual-target)<=2e-11*(1+abs(target)),(key,actual,target)

for fit,copy,name in zip(fits,table['models'],('ols_classical','ols_robust','exp_classical','exp_robust')):
  assert copy['sample']['retained_indices']==list(range(10000))
  assert copy['sample']['dropped_observations']==0
  assert copy['covariance_matrix']==fit['covariance_matrix']
  assert len(fit['coefficients'])==len(copy['coefficients'])==2
  for i,(a,b) in enumerate(zip(fit['coefficients'],copy['coefficients']),1):
    for key,suffix in [('name',None),('value','value'),('standard_error','se'),('statistic','stat'),('p_value','p')]:
      assert a[key]==b[key]
      if suffix: near(b[key],ref[f'{name}.{i}.{suffix}'],f'{name}.{i}.{suffix}')
  assert len(copy['covariance_matrix'])==2
  for i,row in enumerate(copy['covariance_matrix'],1):
    assert len(row)==2
    for j,value in enumerate(row,1): near(value,ref[f'{name}.cov.{i}.{j}'],f'{name}.cov.{i}.{j}')
  if name.startswith('exp_'):
    assert copy['poisson_offset_variable']=='log_exposure'
    assert copy['poisson_offset_coefficient']==1.0
    assert copy['poisson_null_model']=='intercept_only_with_same_fixed_offset'
    assert copy['poisson_null_log_likelihood']==fit['null_log_likelihood']
    near(copy['poisson_null_log_likelihood'],null_count_ll,name+'.null_ll')
    near(copy['poisson_mcfadden_r_squared'],1-ref['poisson_log_likelihood']/null_count_ll,name+'.mcfadden')
    assert copy['log_likelihood_convention']=='poisson_count_pmf'
    near(copy['log_likelihood'],ref['poisson_log_likelihood'],name+'.ll')
    assert copy['poisson_covariance_correction']==(1.0 if name=='exp_classical' else 10000/9999)
  else:
    assert 'poisson_offset_variable' not in copy
    near(copy['log_likelihood'],ref['ols_log_likelihood'],name+'.ll')
print('validated four named source reports, R coefficient inference/covariance and 10000-row samples')
