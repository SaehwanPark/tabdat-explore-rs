#!/usr/bin/env python3
"""Audit source, immutable mixed-fit inference and final ML-owned population predictions."""
import json
import math
from pathlib import Path
import sys

from check_household_bp_lm import audit_source, rows, near


def reference_file(root, kind, name):
  flat = root / (f'{kind}_reference.csv' if name == 'reference.csv' else
                 'ml_predictions.csv' if name == 'predictions.csv' else 'input.csv')
  return flat if flat.exists() else root/kind/name


def pointer(value, path):
  for key in path.strip('/').split('/'):
    value = value[int(key)] if isinstance(value, list) else value[key]
  return value


def reference_gates(model, kind, reference):
  assert len(reference)==len({(r.get('kind'),r['path']) for r in reference})
  count = 0
  for row in reference:
    path, target = row['path'], None
    if kind == 'gee':
      parts = path.split('.')
      index = lambda name: model['covariance_parameter_names'].index('intercept' if name == '_cons' else name)
      if parts[0] == 'working':
        if parts[1] == 'covariance':
          target = f'/covariance_matrix/{index(parts[2])}/{index(parts[3])}'
        elif not parts[1].startswith('wald_') and not parts[2].startswith('ci_'):
          target = f"/coefficients/{index(parts[1])}/{'value' if parts[2]=='estimate' else parts[2]}"
      elif path == 'observation_count': target = '/observation_count'
      elif path == 'group_count': target = '/panel/group_count'
    elif kind == 'fe':
      if row['kind'] == 'common' and path in ('/estimate','/standard_error','/statistic','/p_value'):
        target = f"/coefficients/0/{'value' if path=='/estimate' else path[1:]}"
      elif row['kind'] == 'fit':
        if path.startswith('/normalized_level/covariance_matrix/'):
          target = path.replace('/normalized_level','')
        elif path.startswith('/normalized_level/parameters/'):
          target = f"/coefficients/{path.rsplit('/',1)[1]}/value"
        elif path.startswith('/coefficients/'):
          _,_,index,field = path.split('/')
          target = f'/coefficients/{int(index)+1}/{field}'
        elif path in ('/observation_count','/df_residual'): target = path
        elif path == '/group_count': target = '/panel/group_count'
    elif (path.startswith('/covariance_matrix/') or
          (path.startswith('/coefficients/') and '/ci_' not in path) or
          path in ('/observation_count','/df_residual')): target = path
    elif path == '/group_count': target = '/panel/group_count'
    elif path == '/ml/log_likelihood': target = '/log_likelihood'
    if target is not None:
      near(pointer(model,target),row['value'],path.endswith('p_value'))
      count += 1
  return count


def audit(native, reference, raw, mode='--ascii'):
  assert mode in ('--ascii','--dta','--csv')
  for kind in ('gee','fe','re_gls','re_ml'):
    source = audit_source(native,reference_file(reference,kind,'input.csv' if kind=='gee' else 'source.csv'),raw,mode)
  assert len(source)<=40000
  keys = list(source[0])
  selected = sorted((r for r in source if r['lnexp12m']!=''),key=lambda r:int(r['source_row']))
  n = len(selected)
  assert len({r['source_row'] for r in source}) == len(source)
  groups = [str(int(r['commune'])) for r in selected]
  g = len(set(groups))
  balanced = len({groups.count(group) for group in set(groups)}) == 1
  events = [json.loads(line) for line in (native/'result.jsonl').read_text().splitlines()]
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  assert [e['result_type'] for e in events] == [
    'LoadResult','IsidResult','TableResult','SqlCreateResult','TableResult',
    'GaussianGeeResult','EstimateStoreResult','FixedEffectsResult','EstimateStoreResult',
    'RandomEffectsResult' if balanced else 'UnbalancedRandomEffectsGlsResult','EstimateStoreResult',
    'RandomEffectsResult' if balanced else 'UnbalancedRandomEffectsMlResult','EstimateStoreResult',
    'EstimateTableResult','PredictionResult','PredictionResult','SummarizeResult','TableResult','ExportResult']
  assert events[2]['data']=={'headers':['source_rows','missing_outcomes','source_communes'],
    'rows':[[len(source),len(source)-n,len({r['commune'] for r in source})]]}
  assert events[4]['data']=={'headers':['complete_case_communes'],'rows':[[g]]}
  table = events[13]['data']
  models = table['models']
  names = ['intercept','lnhhexp','age','sex','hhsize','farm','educ']
  assert table['parameter_names']==names
  assert [m['name'] for m in models]==['pa','fe','refgls','remle']
  methods = ['gaussian_exchangeable_gee','within_fixed_effects',
    'equal_group_between_random_effects_gls','gaussian_random_intercept_marginal_ml']
  covariances = ['working_gaussian_exchangeable','model_based_within_fe_normalized_levels',
    'model_based_quasi_gls_rss_over_n_minus_k','model_based_observed_marginal_information']
  gates = 0
  for i,(m,kind) in enumerate(zip(models,('gee','fe','re_gls','re_ml'))):
    assert m['estimator']==methods[i] and m['covariance']==covariances[i]
    assert m['outcome']=='lnexp12m' and m['predictors']==names[1:] and m['include_intercept'] is True
    assert m['covariance_parameter_names']==names and [c['name'] for c in m['coefficients']]==names
    assert m['panel']=={'entity_variable':'commune','group_count':g,
      'time_variable':None if i==0 else 'source_row',
      'intercept_convention':'observation_weighted_zero_effects' if i==1 else 'population_mean_intercept'}
    assert m['cluster_variable'] is None and m['cluster_count'] is None
    assert m['observation_count']==n and m['sample']=={'cluster_groups':groups,'weights':None,
      'total_observations':n,'dropped_observations':0,'retained_indices':list(range(n))}
    assert m['reference_distribution']==('student_t' if i==1 else 'asymptotic_standard_normal')
    assert m['inference_degrees_of_freedom']==(n-g-6 if i==1 else None)
    assert m['df_residual']==(None if i==0 else n-g-6 if i==1 else n-7)
    assert all(m[k] is None for k in ('r_squared','mcfadden_r_squared','root_mse','event_count'))
    assert not any(k.startswith(('poisson_','nls_')) for k in m)
    assert m['log_likelihood_convention']==('gaussian_random_intercept_marginal_ml_full_constants' if i==3 else None)
    if i!=3: assert m['log_likelihood'] is None
    fit = events[5+2*i]['data']
    if i==0:
      coefficients = fit['working']['coefficients']
      matrix = fit['working']['covariance']
    elif i==1:
      coefficients = [fit['common_intercept_statistics']]+fit['coefficients']
      matrix = fit['normalized_level']['covariance_matrix']
    else:
      coefficients = fit['coefficients'];matrix = fit['covariance_matrix']
      assert fit['variance_component_method']==('equal_group_between' if i==2 else
        'balanced_gaussian_marginal_ml' if balanced else 'unbalanced_gaussian_marginal_ml')
    expected = [{k:('intercept' if j==0 else c[k]) if k=='name' else c[k]
      for k in ('name','value','standard_error','statistic','p_value')} for j,c in enumerate(coefficients)]
    assert m['coefficients']==expected and m['covariance_matrix']==matrix
    assert len(matrix)==7 and all(len(row)==7 for row in matrix)
    for j,c in enumerate(m['coefficients']):
      near(c['standard_error']**2,matrix[j][j])
      near(c['statistic'],c['value']/c['standard_error'])
      for k in range(7): near(matrix[j][k],matrix[k][j])
    if i==3: assert m['log_likelihood']==fit['log_likelihood']
    assert events[6+2*i]['data']=={'name':m['name'],'estimator':m['estimator'],'stored_count':i+1}
    checked = reference_gates(m,kind,rows(reference_file(reference,kind,'reference.csv')))
    assert checked==[79,87,80,81][i]
    gates += checked
  actual = rows(native/'household_mixed_report_predictions.csv')
  expected = rows(reference_file(reference,'re_ml','predictions.csv'))
  assert len(actual)==len(expected)==n
  for a,b,s in zip(actual,expected,selected):
    assert list(a)==list(b)==keys+['ml_xb','ml_residual']
    for key in keys: assert float(a[key])==float(b[key])==float(s[key]),(s['source_row'],key)
    for key in ('ml_xb','ml_residual'): near(a[key],b[key])
    xb = models[3]['coefficients'][0]['value']+math.fsum(c['value']*float(s[c['name']]) for c in models[3]['coefficients'][1:])
    near(a['ml_xb'],xb);near(float(a['ml_xb'])+float(a['ml_residual']),s['lnexp12m'])
  loaded,exported = events[0]['data']['dataset'],events[-1]['data']['dataset']
  assert loaded['row_count']==len(source) and exported['row_count']==n
  assert exported['columns'][:len(keys)]==loaded['columns']
  assert [c['name'] for c in exported['columns']]==keys+['ml_xb','ml_residual']
  assert dict((c['name'],c['data_type']) for c in loaded['columns'])['commune']=='BIGINT'
  assert dict((c['name'],c['data_type']) for c in loaded['columns'])['source_row']=='BIGINT'
  return n,g,gates


def main():
  if len(sys.argv) not in (4,5):
    raise SystemExit('usage: check_household_mixed_report.py native-dir reference-root source [--ascii|--dta|--csv]')
  n,g,gates = audit(*map(Path,sys.argv[1:4]),sys.argv[4] if len(sys.argv)==5 else '--ascii')
  print('validated',gates,'independent R reporting quantities, all source rows and',n,'prediction rows;',g,'groups')


if __name__=='__main__':
  main()
