#!/usr/bin/env python3
"""Audit native patents FE bootstrap against raw equations, independent RNG and R."""
import csv
import importlib.util
import json
import math
from pathlib import Path
import sys
from check_native_gamma_re import SOURCE, raw_source, rows, source_equal
from check_native_conditional_fe import EXTRA, independent_equations

EXPORT='native_conditional_fe_bootstrap_predictions.csv'


def near(actual,expected,label=''):
  a,b=float(actual),float(expected)
  assert math.isfinite(a) and math.isfinite(b) and abs(a-b)<=1e-9,(label,a,b)


def empirical_checks(report):
  values=[row[0] for row in report['replicate_coefficients']]
  assert len(values)==500 and all(math.isfinite(v) for v in values)
  mean=math.fsum(values)/500;variance=math.fsum((v-mean)**2 for v in values)/499
  covariance=report['covariance_matrix'];assert len(covariance)==len(covariance[0])==1
  if variance: assert abs(covariance[0][0]/variance-1)<=1e-9,'independent empirical covariance'
  else: assert covariance[0][0]==0
  coefficient=report['coefficients'][0]
  near(coefficient['bootstrap_mean'],mean,'empirical mean');near(coefficient['bias'],mean-coefficient['estimate'],'empirical bias')
  near(coefficient['standard_error'],math.sqrt(variance),'empirical SE')
  ordered=sorted(values)
  for field,p in [('ci_lower',.025),('ci_upper',.975)]:
    index=499*p;lo=math.floor(index);fraction=index-lo
    near(coefficient[field],ordered[lo]+fraction*(ordered[lo+1]-ordered[lo]),'empirical type7')


def load(native,raw,original=True):
  expected=raw_source(raw,original);source=rows(native/'patents.csv');assert len(source)==len(expected)
  seen=set();groups={}
  for row in source:
    key=(int(row['id']),int(row['year']));assert key not in seen and list(row)==SOURCE;seen.add(key);source_equal(row,expected[key])
  assert seen==set(expected)
  for key in sorted(expected): groups.setdefault(key[0],[]).append(expected[key])
  assert all(len(group)==5 for group in groups.values())
  totals={key:math.fsum(float(row['pat']) for row in group) for key,group in groups.items()}
  ids=[key for key,total in totals.items() if total>0];g=len(ids);n=len(source);assert 2<=g<=346
  labels=[str(key) for key in ids];selected=[groups[key] for key in ids]
  events=[json.loads(line) for line in (native/'result.jsonl').read_text().splitlines()]
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  assert [e['result_type'] for e in events]==['LoadResult','IsidResult','TableResult','TableResult','SqlCreateResult','ConditionalPoissonResult','ConditionalPoissonBootstrapResult','PredictionResult','PredictionResult','PredictionResult','SummarizeResult','TableResult','ExportResult']
  fit=events[5]['data'];report=events[6]['data'];positive_rows=5*g;excluded=n-positive_rows
  for field,value in {'estimator':'cluster_pairs_bootstrap_conditional_poisson_fe','outcome':'pat','predictors':['logr'],'entity_variable':'id','replications':500,
    'seed':'10001','rng':'xoshiro256++/splitmix64/u64-rejection-v1','cluster_order':'first_seen_positive_total_complete_case','cluster_index_base':0,
    'confidence_level':.95,'interval_method':'type7_percentile_95','covariance_convention':'whole_cluster_pairs_b_minus_1',
    'covariance_parameter_names':['logr'],'cluster_labels':labels,'cluster_sizes':[5]*g,'cluster_count':g,'observation_count':positive_rows,
    'joint_finite_observation_count':n,'candidate_group_count':len(groups),'zero_total_group_count':len(groups)-g,'zero_total_observation_count':excluded,
    'sample':{'total_observations':n,'dropped_observations':excluded}}.items(): assert report[field]==value,(field,report[field],value)
  assert len(report['coefficients'])==1 and report['coefficients'][0]['name']=='logr'
  assert report['coefficients'][0]['estimate']==fit['coefficients'][0]['value']
  assert fit['group_count']==g and fit['observation_count']==positive_rows
  assert fit['sample']==report['sample'] and fit['candidate_group_count']==len(groups)
  for field in ['replicate_cluster_indices','replicate_observation_counts','replicate_coefficients']: assert len(report[field])==500
  path=Path(__file__).resolve().parents[2]/'crates/tabdat-stats/tests/fixtures/bootstrap/draws.py'
  spec=importlib.util.spec_from_file_location('draws',path);oracle=importlib.util.module_from_spec(spec);spec.loader.exec_module(oracle)
  stream=oracle.stream(10001);zone=oracle.MASK-oracle.MASK%g;draws=[]
  for r in range(500):
    indices=[];steps=0
    while len(indices)<g:
      word=next(stream);steps+=1;assert steps<=g+1024
      if word<zone: indices.append(word%g)
    assert report['replicate_cluster_indices'][r]==indices,('independent RNG',r)
    assert report['replicate_observation_counts'][r]==positive_rows
    coefficients=report['replicate_coefficients'][r];assert len(coefficients)==1 and math.isfinite(coefficients[0])
    beta=coefficients[0];scores=[]
    for index in indices:
      group=selected[index];x=[float(row['logr']) for row in group];y=[float(row['pat']) for row in group]
      eta=[beta*(v-x[0]) for v in x];assert all(math.isfinite(v) for v in eta)
      shift=max(eta);weights=[math.exp(v-shift) for v in eta];den=math.fsum(weights)
      center=math.fsum(v*w for v,w in zip(x,weights))/den;total=math.fsum(y)
      scores.append(math.fsum(v*(z-center) for v,z in zip(y,x)))
    assert abs(math.fsum(scores))<=1e-8+1e-10*math.fsum(abs(v) for v in scores),('independent replicate score',r)
    draws.extend((r+1,pos+1,index,labels[index]) for pos,index in enumerate(indices))
  empirical_checks(report)
  exported=rows(native/EXPORT);assert len(exported)==n
  for row,key in zip(exported,sorted(expected)):
    assert list(row)==SOURCE+EXTRA and (int(row['id']),int(row['year']))==key;source_equal(row,expected[key])
    included=totals[key[0]]>0;assert row['included_in_native_fe']==str(included).lower()
    near(row['selected_firm_total'],totals[key[0]]);near(row['native_fe_xb'],fit['coefficients'][0]['value']*float(row['logr']))
    if included: near(row['native_fe_residual'],float(row['pat'])-float(row['native_fe_mean']))
    else: assert row['native_fe_mean']==row['native_fe_residual']==''
  independent_equations(fit,exported)
  assert events[-1]['data']['path']==EXPORT and events[-1]['data']['dataset']['row_count']==n
  if original: assert (n,len(groups),g,excluded)==(1730,346,324,110)
  return report,draws


def audit(native,reference,raw,original=True):
  report,draws=load(native,raw,original);c=report['coefficients'][0]
  actual={'candidate_rows':report['joint_finite_observation_count'],'candidate_firms':report['candidate_group_count'],'retained_rows':report['observation_count'],
    'firms_per_draw':report['cluster_count'],'replications':500,'estimate':c['estimate'],'bootstrap_mean':c['bootstrap_mean'],'bias':c['bias'],
    'bootstrap_se':c['standard_error'],'ci_lower':c['ci_lower'],'ci_upper':c['ci_upper']}
  records=rows(reference/'reference.csv');oracle={r['field']:float(r['value']) for r in records}
  assert len(records)==len(oracle)==11 and oracle.keys()==actual.keys()
  for key,value in oracle.items(): near(actual[key],value,'R '+key)
  near(report['covariance_matrix'][0][0],oracle['bootstrap_se']**2,'R covariance')
  replicates=rows(reference/'replicates.csv');assert len(replicates)==500
  for i,(row,values) in enumerate(zip(replicates,report['replicate_coefficients'])):
    assert int(row['replicate'])==i+1;near(values[0],row['beta'],'R refit')
  return len(draws),len(records),len(replicates)


def main():
  assert len(sys.argv) in (5,6),'usage: check_native_conditional_fe_bootstrap.py oracle|check native-dir raw.asc draws.csv|reference-dir [--control]'
  mode=sys.argv[1];native,raw,destination=map(Path,sys.argv[2:5]);original=len(sys.argv)==5
  if not original: assert sys.argv[5]=='--control'
  if mode=='oracle':
    report,draws=load(native,raw,original);assert not destination.exists()
    with destination.open('x',newline='') as stream:
      writer=csv.writer(stream);writer.writerow(['replicate','position','index0','firm_id']);writer.writerows(draws)
    print('PASS:',len(draws),'independently checked native draws/raw score and empirical identities')
  else:
    assert mode=='check';print('PASS: draws/R fields/R refits',audit(native,destination,raw,original))


if __name__=='__main__': main()
