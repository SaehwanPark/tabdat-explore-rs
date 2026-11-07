#!/usr/bin/env python3
"""Native patents gamma-RE bootstrap: independent raw scores, RNG and R fits."""
from collections import Counter
import csv
import importlib.util
import json
import math
from pathlib import Path
import sys
from check_native_gamma_re import SOURCE, EXTRA, raw_source, rows, source_equal, independent_identity

EXPORT='native_gamma_re_bootstrap_predictions.csv'
NAMES=['_cons','logr','lnalpha']
R_NAMES=['intercept','slope','lnalpha']


def near(actual,expected,label=''):
  a,b=float(actual),float(expected)
  assert math.isfinite(a) and math.isfinite(b) and abs(a-b)<=1e-9+1e-10*abs(b),(label,a,b)


def empirical_checks(report):
  values=report['replicate_coefficients'];assert len(values)==500
  assert all(len(row)==3 and all(math.isfinite(v) for v in row) for row in values)
  columns=list(zip(*values));means=[v[0]+math.fsum(x-v[0] for x in v)/500 for v in columns]
  covariance=[[math.fsum((row[i]-means[i])*(row[j]-means[j]) for row in values)/499 for j in range(3)] for i in range(3)]
  actual=report['covariance_matrix'];assert len(actual)==3 and all(len(row)==3 for row in actual)
  for i in range(3):
    for j in range(3):
      assert math.isfinite(actual[i][j])
      scale=math.sqrt(covariance[i][i])*math.sqrt(covariance[j][j])
      if scale: assert abs(actual[i][j]-covariance[i][j])<=1e-9*scale,('independent empirical covariance',i,j)
      else: assert actual[i][j]==0,('independent zero covariance',i,j)
    c=report['coefficients'][i];near(c['bootstrap_mean'],means[i],'empirical mean');near(c['bias'],means[i]-c['estimate'],'empirical bias')
    near(c['standard_error'],math.sqrt(covariance[i][i]),'empirical SE')
    ordered=sorted(columns[i])
    for field,p in [('ci_lower',.025),('ci_upper',.975)]:
      h=499*p;lo=math.floor(h);near(c[field],ordered[lo]+(h-lo)*(ordered[lo+1]-ordered[lo]),'empirical type7')


def replicate_score(parameters,groups,frequencies):
  a,b,t=parameters;assert all(math.isfinite(v) for v in parameters) and abs(t)<12
  r=math.exp(-t);scores=[[],[],[]]
  for index,multiplicity in frequencies.items():
    x,y,total=groups[index];eta=[a+b*v for v in x]
    assert all(math.isfinite(v) and -40<=v<=40 for v in eta)
    mu=[math.exp(v) for v in eta];m=math.fsum(mu);h=math.fsum(v*z for v,z in zip(mu,x));q=(r+total)/(r+m)
    # Exact integer-count digamma difference, independent of R/native special functions.
    dg=math.fsum(1/(r+j) for j in range(total))+math.log(r)+1-math.log(r+m)-q
    scores[0].append(multiplicity*(total-q*m))
    scores[1].append(multiplicity*(math.fsum(v*z for v,z in zip(y,x))-q*h))
    scores[2].append(multiplicity*(-r*dg))
  for component,values in enumerate(scores):
    assert abs(math.fsum(values))<=1e-8+1e-10*math.fsum(abs(v) for v in values),('independent replicate score',component)


def load(native,raw,original=True):
  expected=raw_source(raw,original);source=rows(native/'patents.csv');assert len(source)==len(expected)
  seen=set();by_id={}
  for row in source:
    key=(int(row['id']),int(row['year']));assert key not in seen and list(row)==SOURCE;seen.add(key);source_equal(row,expected[key])
  assert seen==set(expected)
  for key in sorted(expected): by_id.setdefault(key[0],[]).append(expected[key])
  assert all(len(group)==5 for group in by_id.values())
  groups=[]
  for group in by_id.values():
    x=[float(v['logr']) for v in group];y=[float(v['pat']) for v in group]
    assert all(math.isfinite(v) and abs(v)<=20 for v in x)
    assert all(math.isfinite(v) and 0<=v<=1000 and v==math.floor(v) for v in y)
    groups.append((x,y,int(math.fsum(y))))
  n=len(source);g=len(groups);zero=sum(total==0 for x,y,total in groups);labels=[str(key) for key in by_id]
  events=[json.loads(line) for line in (native/'result.jsonl').read_text().splitlines()]
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  assert [e['result_type'] for e in events]==['LoadResult','IsidResult','TableResult','TableResult','SqlCreateResult','GammaPoissonResult','GammaPoissonBootstrapResult','PredictionResult','PredictionResult','PredictionResult','SummarizeResult','TableResult','ExportResult']
  fit=events[5]['data'];report=events[6]['data']
  for key,value in {'estimator':'cluster_pairs_bootstrap_shared_gamma_poisson_re','outcome':'pat','predictors':['logr'],'entity_variable':'id','replications':500,
    'seed':'10001','rng':'xoshiro256++/splitmix64/u64-rejection-v1','cluster_order':'first_seen_complete_case','cluster_index_base':0,
    'confidence_level':.95,'interval_method':'type7_percentile_95','covariance_convention':'whole_cluster_pairs_b_minus_1',
    'covariance_parameter_names':NAMES,'cluster_labels':labels,'cluster_sizes':[5]*g,'cluster_count':g,'observation_count':n,
    'zero_total_group_count':zero,'singleton_group_count':0,'sample':{'total_observations':n,'dropped_observations':0}}.items(): assert report[key]==value,(key,report[key],value)
  assert len(report['coefficients'])==3 and [c['name'] for c in report['coefficients']]==NAMES
  point=[c['value'] for c in fit['coefficients']]+[fit['ln_alpha']['value']]
  assert [c['estimate'] for c in report['coefficients']]==point
  assert fit['group_count']==g and fit['observation_count']==n and fit['sample']==report['sample'] and fit['zero_total_group_count']==zero
  for key in ['replicate_cluster_indices','replicate_observation_counts','replicate_coefficients']: assert len(report[key])==500
  path=Path(__file__).resolve().parents[2]/'crates/tabdat-stats/tests/fixtures/bootstrap/draws.py'
  spec=importlib.util.spec_from_file_location('draws',path);oracle=importlib.util.module_from_spec(spec);spec.loader.exec_module(oracle)
  stream=oracle.stream(10001);zone=oracle.MASK-oracle.MASK%g;draws=[]
  for rep in range(500):
    indices=[];steps=[];step=0
    while len(indices)<g:
      word=next(stream);step+=1;assert step<=g+1024
      if word<zone: indices.append(word%g);steps.append(step)
    assert report['replicate_cluster_indices'][rep]==indices,('independent RNG',rep)
    assert report['replicate_observation_counts'][rep]==n
    parameters=report['replicate_coefficients'][rep];assert len(parameters)==3
    replicate_score(parameters,groups,Counter(indices))
    draws.extend((rep+1,pos+1,steps[pos],index,labels[index]) for pos,index in enumerate(indices))
  empirical_checks(report)
  exported=rows(native/EXPORT);assert len(exported)==n
  for row,key in zip(exported,sorted(expected)):
    assert list(row)==SOURCE+EXTRA and (int(row['id']),int(row['year']))==key;source_equal(row,expected[key])
    assert row['included_in_native_re']=='true';near(row['selected_firm_total'],groups[key[0]-1][2])
    eta=point[0]+point[1]*float(row['logr']);near(row['native_re_xb'],eta);near(row['native_re_mean'],math.exp(eta))
    near(row['native_re_residual'],float(row['pat'])-math.exp(eta))
  independent_identity(fit,exported)
  assert events[-1]['data']['path']==EXPORT and events[-1]['data']['dataset']['row_count']==n
  if original: assert (n,g,zero)==(1730,346,22)
  return report,draws,exported


def audit(native,reference,raw,original=True):
  report,draws,exported=load(native,raw,original)
  actual={'candidate_rows':report['observation_count'],'firms_per_draw':report['cluster_count'],'replications':500}
  for i,name in enumerate(R_NAMES):
    c=report['coefficients'][i]
    for prefix,key in [('estimate','estimate'),('mean','bootstrap_mean'),('bias','bias'),('se','standard_error'),('lower','ci_lower'),('upper','ci_upper')]: actual[prefix+'_'+name]=c[key]
  for i,a in enumerate('abt'):
    for j,b in enumerate('abt'):
      if i<=j: actual['covariance_'+a+b]=report['covariance_matrix'][i][j]
  mu=[float(row['native_re_mean']) for row in exported];res=[float(row['native_re_residual']) for row in exported]
  actual.update(original_mu_mean=math.fsum(mu)/len(mu),original_mu_min=min(mu),original_mu_max=max(mu),original_residual_mean=math.fsum(res)/len(res))
  records=rows(reference/'reference.csv');oracle={r['field']:float(r['value']) for r in records}
  assert len(records)==len(oracle)==31 and oracle.keys()==actual.keys()
  for key,value in oracle.items(): near(actual[key],value,'R '+key)
  fits=rows(reference/'fits.csv');assert len(fits)==500
  for index,(row,parameters) in enumerate(zip(fits,report['replicate_coefficients'])):
    assert int(row['replicate'])==index+1
    for i,name in enumerate(R_NAMES): near(parameters[i],row[name],'R refit '+name)
  return len(draws),len(records),len(fits)


def main():
  assert len(sys.argv) in (5,6),'usage: check_native_gamma_re_bootstrap.py oracle|check native-dir raw.asc draws.csv|reference-dir [--control]'
  mode=sys.argv[1];native,raw,destination=map(Path,sys.argv[2:5]);original=len(sys.argv)==5
  if not original: assert sys.argv[5]=='--control'
  if mode=='oracle':
    report,draws,exported=load(native,raw,original);assert not destination.exists()
    with destination.open('x',newline='') as stream:
      writer=csv.writer(stream);writer.writerow(['replicate','position','step','index0','firm_id']);writer.writerows(draws)
    print('PASS:',len(draws),'independent draws/raw joint-score/empirical identities')
  else:
    assert mode=='check';print('PASS: draws/R fields/R joint refits',audit(native,destination,raw,original))


if __name__=='__main__': main()
