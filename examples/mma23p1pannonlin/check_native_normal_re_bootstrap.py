#!/usr/bin/env python3
"""Audit native normal-RE bootstrap from exact raw source, R and raw equations."""
from collections import Counter
from concurrent.futures import ProcessPoolExecutor, as_completed
import json
import math
from pathlib import Path
import sys
from check_native_gamma_re import rows
from check_native_gamma_re_bootstrap import near, empirical_checks
from check_native_normal_re import audit as base_audit, posterior

NAMES=['_cons','logr','lnsigma2']
R_NAMES=['intercept','slope','lnvariance']
EXPORT='native_normal_re_bootstrap_predictions.csv'


def replicate_score(parameters,groups,frequencies):
  a,b,t=parameters
  assert all(math.isfinite(v) for v in parameters) and -8<t<8
  variance=math.exp(t);scores=[[],[],[]]
  for index,multiplicity in frequencies.items():
    x,y,total=groups[index];eta=[a+b*z for z in x]
    assert all(math.isfinite(z) and -40<=z<=40 for z in eta)
    mu=[math.exp(z) for z in eta];m=math.fsum(mu);h=math.fsum(q*z for q,z in zip(mu,x))
    _,ex,u2=posterior(total,m,variance,covariance=False)
    # Complete-data score expectations via independent adaptive Simpson, not
    # native GH/Stein scores or R/native fitted coefficients as oracle inputs.
    scores[0].append(multiplicity*(total-m*ex))
    scores[1].append(multiplicity*(math.fsum(q*z for q,z in zip(y,x))-h*ex))
    scores[2].append(multiplicity*(u2/variance-1)/2)
  for j,terms in enumerate(scores):
    assert abs(math.fsum(terms))<=1e-8+1e-10*math.fsum(abs(v) for v in terms),('independent replicate score',j)


def verify_one_score(task):
  parameters,groups,indices=task
  replicate_score(parameters,groups,Counter(indices))


def verify_scores(values,groups,indices):
  with ProcessPoolExecutor(max_workers=4) as executor:
    futures=[executor.submit(verify_one_score,task) for task in zip(values,[groups]*len(values),indices)]
    try:
      for future in as_completed(futures):future.result()
    except BaseException:
      for future in futures:future.cancel()
      raise


def audit(native,raw,reference,original_reference,original=True):
  # The original fit retains its full42-field R and raw information/null guards.
  n,g,_=base_audit(native,original_reference,raw,original=original,bootstrap=True)
  events=[json.loads(line) for line in (native/'result.jsonl').read_text().splitlines()]
  fit=events[5]['data'];report=events[6]['data']
  labels=[str(i) for i in range(1,g+1)]
  expected={'estimator':'cluster_pairs_bootstrap_normal_poisson_re','random_effect_distribution':'mean_zero_normal_log_effect',
    'outcome':'pat','predictors':['logr'],'entity_variable':'id','replications':500,'seed':'10001',
    'rng':'xoshiro256++/splitmix64/u64-rejection-v1','cluster_order':'first_seen_complete_case','cluster_index_base':0,
    'confidence_level':.95,'interval_method':'type7_percentile_95','covariance_convention':'whole_cluster_pairs_b_minus_1',
    'covariance_parameter_names':NAMES,'cluster_labels':labels,'cluster_sizes':[5]*g,'cluster_count':g,'observation_count':n,
    'zero_total_group_count':fit['zero_total_group_count'],'singleton_group_count':0,
    'sample':{'total_observations':n,'dropped_observations':0},'original_quadrature_nodes':[80,96],
    'refit_quadrature_nodes':[176,208],'refit_initializer':'original_joint_estimate_raw_to_copy_rms'}
  for key,value in expected.items():assert report[key]==value,(key,report[key],value)
  point=[c['value'] for c in fit['coefficients']]+[fit['ln_variance']['value']]
  assert len(report['coefficients'])==3 and [c['name'] for c in report['coefficients']]==NAMES
  assert [c['estimate'] for c in report['coefficients']]==point
  for key in ['replicate_cluster_indices','replicate_observation_counts','replicate_coefficients']:assert len(report[key])==500
  # Regenerate the ENTIRE stream independently; neither R nor Rust supplies
  # oracle indices and a reported trace is never accepted on its own authority.
  from importlib.util import spec_from_file_location,module_from_spec
  location=Path(__file__).resolve().parents[2]/'crates/tabdat-stats/tests/fixtures/bootstrap/draws.py'
  spec=spec_from_file_location('normal_draw_oracle',location);oracle=module_from_spec(spec);spec.loader.exec_module(oracle)
  stream=oracle.stream(10001);zone=oracle.MASK-oracle.MASK%g
  expected_draws=[]
  for rep in range(500):
    indices=[];step=0
    while len(indices)<g:
      word=next(stream);step+=1;assert step<=g+1024
      if word<zone:indices.append(word%g)
    expected_draws.append(indices)
  assert report['replicate_cluster_indices']==expected_draws,'independent RNG stream'
  assert report['replicate_observation_counts']==[n]*500
  empirical_checks(report)
  exported=rows(native/EXPORT);groups=[]
  for i in range(g):
    group=exported[5*i:5*(i+1)]
    assert [int(v['id']) for v in group]==[i+1]*5
    x=[float(v['logr']) for v in group];y=[float(v['pat']) for v in group]
    groups.append((x,y,int(math.fsum(y))))
  independent=rows(reference/'fits.csv');assert len(independent)==500
  for rep,(values,row) in enumerate(zip(report['replicate_coefficients'],independent)):
    assert int(row['replicate'])==rep+1 and len(values)==3
    for j,name in enumerate(R_NAMES):near(values[j],row['estimate_'+name],'independent R refit')
  facts={'candidate_rows':n,'firms_per_draw':g,'replications':500}
  for c,name in zip(report['coefficients'],R_NAMES):
    for key,prefix in [('estimate','estimate'),('bootstrap_mean','mean'),('bias','bias'),('standard_error','se'),('ci_lower','lower'),('ci_upper','upper')]:facts[prefix+'_'+name]=c[key]
  for key,i,j in [('aa',0,0),('ab',0,1),('at',0,2),('bb',1,1),('bt',1,2),('tt',2,2)]:facts['covariance_'+key]=report['covariance_matrix'][i][j]
  means=[float(v['native_re_mean']) for v in exported];residuals=[float(v['native_re_residual']) for v in exported]
  facts.update(original_mu_mean=math.fsum(means)/n,original_mu_min=min(means),original_mu_max=max(means),original_residual_mean=math.fsum(residuals)/n)
  r={v['field']:float(v['value']) for v in rows(reference/'reference.csv')};assert len(facts)==len(r)==31 and facts.keys()==r.keys()
  for key,value in r.items():near(facts[key],value,key)
  verify_scores(report['replicate_coefficients'],groups,expected_draws)
  return n,g


if __name__=='__main__':
  assert len(sys.argv) in (5,6)
  original=len(sys.argv)==5
  if not original:assert sys.argv[5]=='--control'
  n,g=audit(*map(Path,sys.argv[1:5]),original=original)
  print(f'PASS {n} exact source/prediction rows/{g} firms/500 independent R joint refits/{500*g} regenerated draws/31 bootstrap+42 original R facts/full empirical covariance and raw scores/information/null equations')
