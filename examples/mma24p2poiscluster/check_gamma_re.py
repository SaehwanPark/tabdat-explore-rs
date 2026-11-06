#!/usr/bin/env python3
"""Audit pinned ASCII source, shared-gamma inference and every marginal prediction."""
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


def near(actual, expected, label):
  a,b=float(actual),float(expected)
  assert math.isfinite(a) and math.isfinite(b),(label,a,b)
  tol=abs(b)*1e-8 if label.endswith('p_value') else 1e-8 if 'log_likelihood' in label else 1e-9
  assert abs(a-b)<=tol,(label,a,b,tol)
  return abs(a-b)


def main():
  if len(sys.argv) not in (4,5) or (len(sys.argv)==5 and sys.argv[4]!='--dta'):
    raise SystemExit('usage: check_gamma_re.py native-dir reference-dir vietnam_ex2.asc|dta [--dta]')
  native,ref,raw=map(Path,sys.argv[1:4]);dta=len(sys.argv)==5
  digest='284a5dd732926fe6ee02157012b17d8751523b89f1dd2f652a6f4dd021f3dac3' if dta else 'e876f8ee17791881c6378de2761436e1e72e1eda84347089a9dc365ed290c85d'
  assert hashlib.sha256(raw.read_bytes()).hexdigest()==digest
  tokens=None if dta else shlex.split(raw.read_text(encoding='ascii'))
  if not dta: assert len(tokens)==27766*12
  names=['pharvis','lnhhexp','age','sex','married','educ','illness','injury','illdays','actdays','insurance','commune']
  source,expected=rows(native/'vietnam_individual.csv'),rows(ref/'source.csv')
  assert len(source)==len(expected)==27766
  for i,(a,b) in enumerate(zip(source,expected)):
    assert set(a)==set(b)==set(names)|{'source_row'}
    assert int(a['source_row'])==int(b['source_row'])==i+1
    for j,key in enumerate(names):
      if dta:
        # The independent R reference loads the pinned DTA directly with
        # foreign; compare its typed source export, not the rounded ASCII.
        if b[key]=='': assert a[key]==''
        else: assert Decimal(a[key])==Decimal(b[key]),(i,key)
      else:
        original=tokens[12*i+j]
        if original=='.': assert a[key]==b[key]==''
        else: assert Decimal(a[key])==Decimal(b[key])==Decimal(original),(i,key)
  events=[json.loads(line) for line in (native/'result.jsonl').read_text().splitlines()]
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  assert [e['result_type'] for e in events]==['LoadResult','IsidResult','TableResult','SqlCreateResult','GammaPoissonResult','PredictionResult','PredictionResult','PredictionResult','SummarizeResult','TableResult','ExportResult']
  fit=events[4]['data'];pred=['lnhhexp','insurance','sex','age','married','illdays','actdays','injury','illness','educ']
  assert fit['estimator']=='shared_gamma_poisson_re' and fit['entity_variable']=='commune' and fit['outcome']=='pharvis'
  assert fit['predictors']==pred and fit['include_intercept'] is True
  assert [c['name'] for c in fit['coefficients']]==['_cons']+pred
  assert fit['ln_alpha']['name']=='lnalpha' and fit['covariance_parameter_names']==['_cons']+pred+['lnalpha']
  assert fit['covariance']=='inverse_full_joint_observed_information' and fit['covariance_correction']==1
  assert fit['random_effects_distribution']=='mean_one_gamma' and fit['alpha_parameterization']=='variance_of_shared_multiplicative_group_effect'
  assert fit['reference_distribution']=='asymptotic_standard_normal' and fit['confidence_level']==95
  assert fit['joint_test_reference']=='asymptotic_chi_square' and fit['dispersion_likelihood_ratio_reference']=='equal_mixture_chi_square_0_1'
  assert fit['dispersion_likelihood_ratio_null']=='alpha=0_same_sample_refitted_pooled_poisson'
  assert fit['ln_alpha_test_null']=='ln_alpha=0_equivalently_alpha=1_not_zero_variance'
  assert fit['prediction_scope']=='population_marginal_not_posterior_group_effects'
  assert fit['sample']=={'total_observations':27765,'dropped_observations':0}
  assert fit['converged'] and fit['convergence_absolute_tolerance']==1e-10
  assert 0<=fit['max_abs_scaled_score']<=1e-10 and 0<=fit['max_abs_scaled_newton_step']<=1e-10
  reference=rows(ref/'reference.csv');assert len(reference)==len({r['path'] for r in reference})==236
  maximum=0.0
  for row in reference:
    path=row['path'].split('/');path[0]='covariance_matrix' if path[0]=='covariance' else path[0]
    value=fit
    for key in path: value=value[int(key)] if isinstance(value,list) else value[key]
    maximum=max(maximum,near(value,row['value'],row['path']))
  near(math.exp(fit['ln_alpha']['value']),fit['alpha'],'alpha identity')
  actual,expected=rows(native/'individual_gamma_re_predictions.csv'),rows(ref/'predictions.csv')
  original=[s for s in source if s['pharvis']!=''];assert len(actual)==len(expected)==len(original)==27765
  totals=defaultdict(int)
  for s in original:totals[s['commune']]+=int(s['pharvis'])
  assert len(totals)==194 and {g for g,s in totals.items() if s==0}=={'64'}
  for a,b,s in zip(actual,expected,original):
    assert set(a)==set(b)|{'selected_commune_total','included_in_re'} and set(b)==set(s)|{'re_mean','re_xb','re_residual'}
    for key in s:assert Decimal(a[key])==Decimal(b[key])==Decimal(s[key]),(s['source_row'],key)
    assert int(a['selected_commune_total'])==totals[s['commune']] and a['included_in_re'].lower()=='true'
    for key in ('re_mean','re_xb','re_residual'):near(a[key],b[key],key)
    near(a['pharvis'],float(a['re_mean'])+float(a['re_residual']),'residual decomposition')
    assert float(a['re_mean'])>0
  assert fit['observation_count']==27765 and fit['group_count']==194 and fit['zero_total_group_count']==1
  assert events[-1]['data']['dataset']['row_count']==27765
  print(f'validated 236 R quantities, 27766 source and 27765 marginal prediction rows; max quantity difference {maximum:.6g}')


if __name__=='__main__':
  main()
