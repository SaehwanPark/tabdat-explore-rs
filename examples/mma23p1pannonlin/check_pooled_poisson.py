#!/usr/bin/env python3
"""Independent original-source/R/native audit; Python standard library only."""
import csv
from decimal import Decimal
import hashlib
import json
import math
from pathlib import Path
import sys

if len(sys.argv)!=4:
  raise SystemExit('usage: check_pooled_poisson.py native-dir reference-dir patr7079.asc')
native, reference, raw = map(Path,sys.argv[1:])
assert hashlib.sha256(raw.read_bytes()).hexdigest()=='87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8'
def records(path):
  with path.open(newline='') as stream: return list(csv.DictReader(stream))
def near(actual, expected, label):
  a,e=float(actual),float(expected)
  assert math.isfinite(a) and math.isfinite(e),(label,a,e)
  tolerance = max(abs(e)*1e-8, sys.float_info.min) if label.endswith('p_value') else 1e-9+1e-9*abs(e)
  assert abs(a-e)<=tolerance,(label,a,e,tolerance)

tokens=raw.read_text().split(); assert len(tokens)==346*25
source=[tokens[25*i:25*(i+1)] for i in range(346)]
assert [(i,j) for i,r in enumerate(source) for j,v in enumerate(r) if v=='.']==[(i,1) for i in (9,40,57,80,123,133,149,205,230,232)]
events=[json.loads(line) for line in (native/'result.jsonl').read_text().splitlines()]
assert all(e['schema_version']==1 and 'error' not in e for e in events)
assert [e['result_type'] for e in events]==['LoadResult','IsidResult','TableResult','TableResult','SummarizeResult',
  'PoissonResult','PoissonMarginalEffectsResult','PoissonResult','PoissonMarginalEffectsResult',
  'PoissonResult','PoissonMarginalEffectsResult','PredictionResult','PredictionResult','SummarizeResult','ExportResult']
modes=['classical','robust','cluster']
fits=dict(zip(modes,[e['data'] for e in events if e['result_type']=='PoissonResult']))
effects=dict(zip(modes,[e['data'] for e in events if e['result_type']=='PoissonMarginalEffectsResult']))
conventions={'classical':'model_based_poisson_information','robust':'robust_sandwich_n_over_n_minus_1',
  'cluster':'cluster_sandwich_g_over_g_minus_1'}
for mode in modes:
  fit=fits[mode]; effect=effects[mode]
  assert fit['estimator']==effect['estimator']=='poisson' and fit['outcome']=='pat' and fit['predictors']==['logr']
  assert fit['observation_count']==effect['observation_count']==1730 and fit['covariance']==effect['covariance']==conventions[mode]
  assert fit['reference_distribution']==effect['reference_distribution']=='asymptotic_standard_normal'
  assert fit['covariance_parameter_names']==['intercept','logr'] and fit['include_intercept'] and fit['converged']
  assert fit['sample']['retained_indices']==list(range(1730)) and fit['sample']['dropped_observations']==0
  assert effect['effect']=='average_continuous_predictor_column_derivative' and effect['uncertainty']=='conditional_on_observed_design_delta_method'
  if mode=='cluster':
    assert fit['cluster']['group_count']==346 and fit['cluster']['variable']=='id'
    near(fit['cluster']['finite_sample_multiplier'],346/345,'cluster_correction')
    assert fit['sample']['cluster_groups']==[str(1+i//5) for i in range(1730)]
  assert (fit['likelihood_ratio_p_value'] is not None)==(mode=='classical')
refs=records(reference/'reference.csv'); assert len(refs)==103
for r in refs:
  mode,q,i,j=r['mode'],r['quantity'],int(r['i']),int(r['j']); fit=fits[mode]
  if q.startswith('coefficient_'): actual=fit['coefficients'][i][q.removeprefix('coefficient_')]
  elif q.startswith('effect_'): actual=effects[mode]['effects'][i][q.removeprefix('effect_')]
  elif q=='covariance': actual=fit['covariance_matrix'][i][j]
  elif q.startswith('wald_'): actual=fit['omnibus_wald'][q.removeprefix('wald_')]
  else: actual=fit[q]
  near(actual,r['value'],q)
for mode,printed in zip(modes,[.0022454,.0198092,.0430441]):
  assert abs(fits[mode]['coefficients'][1]['standard_error']-printed)<.00000005
assert abs(fits['classical']['coefficients'][1]['value']-.6929337)<.00000005
actual=records(native/'pooled_poisson_predictions.csv'); expected=records(reference/'predictions.csv')
assert len(actual)==len(expected)==1730
assert list(actual[0])==['id','year','cusip','ardssic','scisect','logk','sumpat','logr','pat','poisson_mu','poisson_xb']
for i,(a,e) in enumerate(zip(actual,expected)):
  r=source[i//5]; t=i%5
  expected_source={'id':str(i//5+1),'year':str(1975+t),'cusip':r[0],'ardssic':'' if r[1]=='.' else r[1],
    'scisect':r[2],'logk':r[3],'sumpat':r[4],'logr':r[10+t],'pat':r[20+t]}
  for key,v in expected_source.items():
    if v=='': assert a[key]==e[key]==''
    else: assert Decimal(a[key])==Decimal(e[key])==Decimal(v),(i,key,a[key],e[key],v)
  near(a['poisson_mu'],e['mu'],'mean'); near(a['poisson_xb'],e['xb'],'xb')
  near(a['poisson_mu'],math.exp(float(a['poisson_xb'])),'exp_xb')
print('PASS: 103 R quantities, 1,730 source-derived records and 19,030 exported cells; 346 firms')
