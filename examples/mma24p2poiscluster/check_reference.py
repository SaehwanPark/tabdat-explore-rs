#!/usr/bin/env python3
"""Pin and audit every source/cohort row and independent R Poisson quantity."""
import csv
from decimal import Decimal
import hashlib
import json
import math
from pathlib import Path
import shlex
import sys


def rows(path):
  with path.open(newline='') as f:
    return list(csv.DictReader(f))


def near(actual, expected, label):
  a, b = float(actual), float(expected)
  assert math.isfinite(a) and math.isfinite(b), (label,a,b)
  tol = (abs(b)*1e-8 if b else 0) if label.endswith('p_value') else 1e-9
  assert abs(a-b)<=tol, (label,a,b,tol)


def main():
  if len(sys.argv)!=4:
    raise SystemExit('usage: check_reference.py native-dir reference-dir vietnam_ex2.asc')
  native, reference, raw = map(Path,sys.argv[1:])
  assert hashlib.sha256(raw.read_bytes()).hexdigest()=='e876f8ee17791881c6378de2761436e1e72e1eda84347089a9dc365ed290c85d'
  tokens=shlex.split(raw.read_text(encoding='ascii'))
  assert len(tokens)==27766*12
  source=rows(native/'vietnam_individual.csv'); refsource=rows(reference/'source.csv')
  assert len(source)==len(refsource)==27766
  columns=['pharvis','lnhhexp','age','sex','married','educ','illness','injury',
           'illdays','actdays','insurance','commune']
  for i,(a,b) in enumerate(zip(source,refsource)):
    assert int(a['source_row'])==int(b['source_row'])==i+1
    assert set(a)==set(b)==set(['source_row']+columns)
    for j,key in enumerate(columns):
      original=tokens[12*i+j]
      if original=='.':
        assert a[key]==b[key]==''
      else:
        assert Decimal(a[key])==Decimal(b[key])==Decimal(original),(i+1,key)
  events=[json.loads(line) for line in (native/'result.jsonl').read_text().splitlines()]
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  assert [e['result_type'] for e in events]==['LoadResult','IsidResult','TableResult','SqlCreateResult',
    'SummarizeResult','PoissonResult','PoissonMarginalEffectsResult','PoissonResult',
    'PoissonMarginalEffectsResult','PoissonResult','PoissonMarginalEffectsResult',
    'PredictionResult','PredictionResult','SummarizeResult','TableResult','ExportResult']
  select=lambda name:[e['data'] for e in events if e['result_type']==name]
  assert select('TableResult')[0]['rows']==[[27766,1,194]]
  modes=['classical','robust','cluster']
  fits=dict(zip(modes,select('PoissonResult')))
  effects=dict(zip(modes,select('PoissonMarginalEffectsResult')))
  names=['lnhhexp','insurance','sex','age','married','illdays','actdays','injury','illness','educ']
  conventions=['model_based_poisson_information','robust_sandwich_n_over_n_minus_1',
               'cluster_sandwich_g_over_g_minus_1']
  for mode,convention in zip(modes,conventions):
    fit,effect=fits[mode],effects[mode]
    assert fit['estimator']==effect['estimator']=='poisson' and fit['outcome']=='pharvis'
    assert fit['predictors']==names and fit['covariance_parameter_names']==['intercept']+names
    assert fit['observation_count']==effect['observation_count']==27765
    assert fit['covariance']==effect['covariance']==convention and fit['include_intercept'] and fit['converged']
    assert fit['reference_distribution']==effect['reference_distribution']=='asymptotic_standard_normal'
    assert fit['sample']['retained_indices']==list(range(27765))
    assert fit['sample']['total_observations']==27765 and fit['sample']['dropped_observations']==0
    assert effect['effect']=='average_continuous_predictor_column_derivative'
    assert effect['uncertainty']=='conditional_on_observed_design_delta_method'
    assert effect['sample']=='retained_estimation_design' and len(effect['effects'])==10
    assert (fit['likelihood_ratio_p_value'] is not None)==(mode=='classical')
    if mode=='cluster':
      assert fit['cluster']['group_count']==194 and fit['cluster']['variable']=='commune'
      near(fit['cluster']['finite_sample_multiplier'],194/193,'cluster_multiplier')
  refs=rows(reference/'reference.csv')
  assert len(refs)==778 and len({(r['mode'],r['quantity'],r['i'],r['j']) for r in refs})==778
  for r in refs:
    mode,q,i,j=r['mode'],r['quantity'],int(r['i']),int(r['j']); fit=fits[mode]
    if q.startswith('coefficient_'): actual=fit['coefficients'][i][q.removeprefix('coefficient_')]
    elif q.startswith('effect_'): actual=effects[mode]['effects'][i][q.removeprefix('effect_')]
    elif q=='covariance': actual=fit['covariance_matrix'][i][j]
    elif q.startswith('wald_'): actual=fit['omnibus_wald'][q.removeprefix('wald_')]
    else: actual=fit[q]
    near(actual,r['value'],q)
  actual=rows(native/'individual_poisson_predictions.csv'); expected=rows(reference/'predictions.csv')
  assert len(actual)==len(expected)==27765
  original=[r for r in source if r['pharvis']!='']
  assert len(original)==27765 and sum(r['pharvis']=='0' for r in original)==fits['classical']['zero_count']
  for a,b,s in zip(actual,expected,original):
    assert set(a)==set(b)==set(s)|{'pooled_mu','pooled_xb'}
    for key in s:
      assert Decimal(a[key])==Decimal(b[key])==Decimal(s[key]), (s['source_row'],key)
    near(a['pooled_mu'],b['pooled_mu'],'pooled_mu')
    near(a['pooled_xb'],b['pooled_xb'],'pooled_xb')
    near(a['pooled_mu'],math.exp(float(a['pooled_xb'])),'exp_xb')
  assert select('ExportResult')[0]['dataset']['row_count']==27765
  print('validated 778 independent R quantities, 27766 source rows and 27765 prediction rows')


if __name__=='__main__':
  main()
