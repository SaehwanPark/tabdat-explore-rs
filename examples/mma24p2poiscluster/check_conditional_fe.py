#!/usr/bin/env python3
"""Audit pinned source, conditional Poisson FE inference, exclusions and predictions."""
import csv
from collections import defaultdict
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


def near(a, b, label):
  a,b=float(a),float(b)
  assert math.isfinite(a) and math.isfinite(b),(label,a,b)
  tol=abs(b)*1e-8 if label.endswith('p_value') else 1e-8 if 'log_likelihood' in label else 1e-9
  assert abs(a-b)<=tol,(label,a,b,tol)


def main():
  if len(sys.argv)!=4:
    raise SystemExit('usage: check_conditional_fe.py native-dir reference-dir vietnam_ex2.asc')
  native,ref,raw=map(Path,sys.argv[1:])
  assert hashlib.sha256(raw.read_bytes()).hexdigest()=='e876f8ee17791881c6378de2761436e1e72e1eda84347089a9dc365ed290c85d'
  tokens=shlex.split(raw.read_text(encoding='ascii')); assert len(tokens)==27766*12
  source,expected=rows(native/'vietnam_individual.csv'),rows(ref/'source.csv')
  names=['pharvis','lnhhexp','age','sex','married','educ','illness','injury','illdays','actdays','insurance','commune']
  assert len(source)==len(expected)==27766
  for i,(a,b) in enumerate(zip(source,expected)):
    assert set(a)==set(b)==set(names)|{'source_row'}
    assert int(a['source_row'])==int(b['source_row'])==i+1
    for j,key in enumerate(names):
      original=tokens[12*i+j]
      if original=='.': assert a[key]==b[key]==''
      else: assert Decimal(a[key])==Decimal(b[key])==Decimal(original),(i,key)
  events=[json.loads(line) for line in (native/'result.jsonl').read_text().splitlines()]
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  assert [e['result_type'] for e in events]==['LoadResult','IsidResult','TableResult','SqlCreateResult','ConditionalPoissonResult','PredictionResult','PredictionResult','PredictionResult','SummarizeResult','TableResult','ExportResult']
  fit=events[4]['data']
  assert fit['estimator']=='conditional_poisson_fe' and fit['entity_variable']=='commune'
  assert fit['outcome']=='pharvis' and fit['observation_count']==27671 and fit['group_count']==193
  assert fit['joint_finite_observation_count']==27765 and fit['zero_total_group_count']==1 and fit['zero_total_observation_count']==94
  predictors=['lnhhexp','insurance','sex','age','married','illdays','actdays','injury','illness','educ']
  assert fit['predictors']==fit['covariance_parameter_names']==[c['name'] for c in fit['coefficients']]==predictors
  assert fit['include_intercept'] is False and fit['covariance']=='inverse_conditional_information'
  assert fit['covariance_correction']==1 and fit['reference_distribution']=='asymptotic_standard_normal'
  assert fit['joint_test_reference']=='asymptotic_chi_square' and fit['confidence_level']==95
  assert fit['converged'] and fit['convergence_absolute_tolerance']==1e-10
  assert 0<=fit['max_abs_scaled_score']<=1e-10 and 0<=fit['max_abs_scaled_newton_step']<=1e-10
  reference=rows(ref/'reference.csv')
  assert len(reference)==len({r['path'] for r in reference})==179
  for r in reference:
    v=fit
    for key in r['path'].strip('/').split('/'):
      v=v[int(key)] if isinstance(v,list) else v[key]
    near(v,r['value'],r['path'])
  actual,expected=rows(native/'individual_conditional_fe_predictions.csv'),rows(ref/'predictions.csv')
  original=[r for r in source if r['pharvis']!='']
  assert len(actual)==len(expected)==len(original)==27765
  totals=defaultdict(int); means=defaultdict(float); kept=0
  for s in original: totals[s['commune']]+=int(s['pharvis'])
  for a,b,s in zip(actual,expected,original):
    assert set(a)==set(b)==set(s)|{'selected_commune_total','included_in_fe','fe_mean','fe_xb','fe_residual'}
    for key in s: assert Decimal(a[key])==Decimal(b[key])==Decimal(s[key]),(s['source_row'],key)
    group=s['commune']; included=totals[group]>0
    assert int(a['selected_commune_total'])==int(b['selected_commune_total'])==totals[group]
    assert a['included_in_fe'].lower()==b['included_in_fe'].lower()==str(included).lower()
    near(a['fe_xb'],b['fe_xb'],'fe_xb')
    if included:
      kept+=1
      for key in ('fe_mean','fe_residual'): near(a[key],b[key],key)
      near(a['pharvis'],float(a['fe_mean'])+float(a['fe_residual']),'decomposition')
      means[group]+=float(a['fe_mean'])
    else: assert a['fe_mean']==a['fe_residual']==b['fe_mean']==b['fe_residual']==''
  assert kept==27671 and {g for g,t in totals.items() if t==0}=={'64'}
  for g,value in means.items(): near(value,totals[g],'group fitted total')
  assert events[-1]['data']['dataset']['row_count']==27765
  print('validated 179 independent R quantities, 27766 source rows and 27765 prediction/exclusion rows')


if __name__=='__main__':
  main()
