#!/usr/bin/env python3
"""Compare the combined CLI workflow with independent base-R reference output."""
import csv
import hashlib
import json
import math
from pathlib import Path
import sys

if len(sys.argv) != 6:
  raise SystemExit('usage: check.py original.asc result.jsonl predictions.csv ols-exp-reference nls-reference-parent')
source, result, predictions, linear_ref, nls_parent = map(Path, sys.argv[1:])
assert hashlib.sha256(source.read_bytes()).hexdigest() == '460f5d731f1848c8695d78a1fbf0db348d0daced4b8b5b0c2734bffa20b3e31c'
def rows(path):
  with path.open(newline='') as stream:
    return list(csv.DictReader(stream))
def near(actual, expected, label):
  actual, expected = float(actual), float(expected)
  assert math.isfinite(actual) and math.isfinite(expected), (label, actual, expected)
  tolerance = abs(expected)*1e-8 if label.endswith('.p') and 0 < abs(expected) < 1e-20 else 1e-11 + 1e-10*abs(expected)
  assert abs(actual-expected) <= tolerance, (label, actual, expected)

raw = [list(map(float, line.split())) for line in source.read_text().splitlines()]
prepared, native = rows(result.parent/'nls.csv'), rows(predictions)
linear = rows(linear_ref/'projections.csv')
weighted = rows(nls_parent/'weighted-reference'/'projections.csv')
weighted = {method: [r for r in weighted if r['case']=='original' and r['method']==method] for method in ('wnls','fgnls')}
assert len(raw)==len(prepared)==len(native)==len(linear)==10000
for i, (original, inp, actual, ref) in enumerate(zip(raw, prepared, native, linear), 1):
  y,x = original
  assert inp['source_row']==actual['source_row']==ref['source_row']==str(i)
  assert float(inp['y'])==float(actual['y'])==y and float(inp['x'])==float(actual['x'])==x
  for field, reference in [('ols_mean','ols_classical'),('exp_integrated','integrated_classical'),('exp_mean','exponential_mean')]:
    near(actual[field],ref[reference],f'{i}.{field}')
  for method in ('wnls','fgnls'):
    w = weighted[method][i-1]
    assert w['row']==str(i) and float(w['y'])==y and float(w['x'])==x
    for field, reference in [('nls_mean','preliminary_mean'),(method+'_scale','scale'),(method+'_weight','weight'),(method+'_mean','level_mean')]:
      near(actual[field],w[reference],f'{i}.{field}')

leaves = [json.loads(line) for line in result.read_text().splitlines()]
fits = [v['data'] for v in leaves if v['result_type'] in ('RegressionResult','PoissonResult','NlsResult')]
assert len(fits)==10
ordered = [fits[i] for i in (0,2,4,6,8,1,3,5,7,9)]
tables = [v['data'] for v in leaves if v['result_type']=='EstimateTableResult']
assert len(tables)==2
snapshots = tables[0]['models']+tables[1]['models']
assert [m['name'] for m in snapshots]==[f'{method}_{mode}' for mode in ('working','robust') for method in ('ols','exp','nls','wnls','fgnls')]
for snapshot, fit in zip(snapshots,ordered):
  assert snapshot['sample']['retained_indices']==list(range(10000))
  assert snapshot['sample']['dropped_observations']==0
  assert snapshot['covariance']==fit['covariance'] and snapshot['covariance_matrix']==fit['covariance_matrix']
  for actual, original in zip(snapshot['coefficients'],fit['coefficients']):
    for key in ('name','value','standard_error','statistic','p_value'):
      assert actual[key]==original[key]
  if snapshot['estimator']=='nls':
    assert snapshot['nls_model']==fit['model'] and snapshot['log_likelihood'] is None
  if snapshot['estimator']=='poisson':
    assert snapshot['poisson_offset_variable']=='log_exposure'
    assert snapshot['log_likelihood_convention']=='poisson_count_pmf'

ref = {r['key']:float(r['value']) for r in rows(linear_ref/'reference.csv')}
for fit, name in zip(fits[:4],('ols_classical','ols_robust','exp_classical','exp_robust')):
  for i, coefficient in enumerate(fit['coefficients'],1):
    for field, rfield in [('value','value'),('standard_error','se'),('statistic','stat'),('p_value','p')]:
      key=f'{name}.{i}.{rfield}'; near(coefficient[field],ref[key],key)
    for j, value in enumerate(fit['covariance_matrix'][i-1],1):
      key=f'{name}.cov.{i}.{j}'; near(value,ref[key],key)

nls_fits = {('nls','classical'):fits[4],('nls','hc1'):fits[5],('wnls','classical'):fits[6],('wnls','hc1'):fits[7],('fgnls','classical'):fits[8],('fgnls','hc1'):fits[9]}
for method, directory in [('nls',nls_parent/'nls-reference'),('wnls',nls_parent/'weighted-reference'),('fgnls',nls_parent/'weighted-reference')]:
  for row in rows(directory/'parameters.csv'):
    if row['case']!='original' or (method!='nls' and row['method']!=method): continue
    fit=nls_fits[method,row['mode']]; c=fit['coefficients'][('a','b').index(row['parameter'])]
    for field,rfield in [('value','estimate'),('standard_error','standard_error'),('statistic','t'),('p_value','p_value'),('confidence_lower','ci_lower'),('confidence_upper','ci_upper')]:
      near(c[field],row[rfield],method+'.'+rfield.replace('p_value','p'))
  for row in rows(directory/'covariance.csv'):
    if row['case']!='original' or (method!='nls' and row['method']!=method): continue
    fit=nls_fits[method,row['mode']]
    near(fit['covariance_matrix'][('a','b').index(row['row_parameter'])][('a','b').index(row['column_parameter'])],row['value'],method+'.cov')

summary, = [v['data'] for v in leaves if v['result_type']=='TableResult' and 'exp_density_log_likelihood' in v['data']['headers']]
actual = dict(zip(summary['headers'],summary['rows'][0])); assert actual['observations']==10000
y = [r[0] for r in raw]; mean = math.fsum(y)/len(y); tss=math.fsum((v-mean)**2 for v in y)
for method in ('ols','exp','nls','wnls','fgnls'):
  means = [float(r[method+'_mean']) for r in native]
  near(actual[method+'_level_r_squared'],1-math.fsum((v-m)**2 for v,m in zip(y,means))/tss,method+'.r2')
  if method!='ols':
    near(actual[method+'_density_log_likelihood'],math.fsum(-math.log(m)-v/m for v,m in zip(y,means)),method+'.density')
print('PASS: 10000 ordered source rows, all projections, ten fits and stored reports, original-scale diagnostics')
