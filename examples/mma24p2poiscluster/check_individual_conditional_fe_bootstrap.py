#!/usr/bin/env python3
"""Audit source, FE exclusions/predictions, all whole-commune draws and R refits."""
import csv
from collections import defaultdict
from decimal import Decimal
import hashlib
import importlib.util
import json
import math
from pathlib import Path
import sys

NAMES=['lnhhexp','insurance','sex','age','married','illdays','actdays','injury','illness','educ']
SOURCE=['source_row','pharvis','lnhhexp','age','sex','married','educ','illness','injury','illdays','actdays','insurance','commune']
ASCII_HASH='e876f8ee17791881c6378de2761436e1e72e1eda84347089a9dc365ed290c85d'
DTA_HASH='284a5dd732926fe6ee02157012b17d8751523b89f1dd2f652a6f4dd021f3dac3'


def rows(path):
  with path.open(newline='') as f:
    return list(csv.DictReader(f))


def near(a,b):
  a,b=float(a),float(b)
  assert math.isfinite(a) and math.isfinite(b) and abs(a-b)<=1e-9,(a,b)


def load(native,raw,mode):
  if mode!='--synthetic':
    assert hashlib.sha256(raw.read_bytes()).hexdigest()==(DTA_HASH if mode=='--dta' else ASCII_HASH)
  source=rows(native/'vietnam_individual.csv')
  assert 1<=len(source)<=27766 and all(set(r)==set(SOURCE) for r in source)
  assert len({int(r['source_row']) for r in source})==len(source)
  assert all(1<=int(r['source_row'])<=27766 for r in source)
  if mode=='ascii':
    tokens=raw.read_text(encoding='ascii').split();assert len(tokens)==27766*12==12*len(source)
    for row in source:
      index=int(row['source_row'])-1
      for key,token in zip(SOURCE[1:],tokens[12*index:12*index+12]):
        if token=='.': assert row[key]==''
        else: assert Decimal(row[key])==Decimal(token),(index,key,row[key],token)
  if mode=='--synthetic': assert rows(raw)==source
  for row in source:
    if row['pharvis']=='': assert all(row[k]=='' for k in SOURCE[1:])
    else:
      assert all(row[k]!='' and math.isfinite(float(row[k])) for k in SOURCE[1:])
      assert 0<=int(row['pharvis'])<=100000 and 1<=int(row['commune'])<=194
  complete=sorted((r for r in source if r['pharvis']!=''),key=lambda r:int(r['source_row']))
  totals=defaultdict(int);candidate=set()
  for row in complete: totals[row['commune']]+=int(row['pharvis']);candidate.add(row['commune'])
  selected=[r for r in complete if totals[r['commune']]>0]
  labels=[];sizes=[];lookup={}
  for row in selected:
    key=row['commune']
    if key not in lookup: lookup[key]=len(labels);labels.append(key);sizes.append(0)
    sizes[lookup[key]]+=1
  G=len(labels);N=len(selected);zero=[r for r in complete if totals[r['commune']]==0]
  assert 10<G<=194 and max(sizes)<=206
  if mode!='--synthetic':
    assert (len(source),len(complete),N,G,len(candidate),len(zero),set(r['commune'] for r in zero))==(27766,27765,27671,193,194,94,{'64'})
    assert sum(int(r['pharvis']) for r in selected)==14209
  events=[json.loads(line) for line in (native/'result.jsonl').read_text().splitlines()]
  assert events and all(e['schema_version']==1 and 'error' not in e for e in events)
  assert [e['result_type'] for e in events]==['LoadResult','IsidResult','TableResult','SqlCreateResult','ConditionalPoissonResult','ConditionalPoissonBootstrapResult','PredictionResult','PredictionResult','PredictionResult','SummarizeResult','TableResult','ExportResult']
  reports=[e['data'] for e in events if e['result_type']=='ConditionalPoissonBootstrapResult'];assert len(reports)==1
  report=reports[0]
  assert report['estimator']=='cluster_pairs_bootstrap_conditional_poisson_fe' and report['replications']==500
  assert report['seed']=='10001' and report['rng']=='xoshiro256++/splitmix64/u64-rejection-v1'
  assert report['cluster_order']=='first_seen_positive_total_complete_case' and report['cluster_index_base']==0
  assert report['entity_variable']=='commune' and report['cluster_count']==G
  assert report['cluster_labels']==labels and report['cluster_sizes']==sizes and report['observation_count']==N
  assert report['joint_finite_observation_count']==len(complete) and report['candidate_group_count']==len(candidate)
  assert report['sample']=={'total_observations':len(complete),'dropped_observations':len(zero)}
  assert report['confidence_level']==0.95
  assert report['zero_total_group_count']==len(candidate)-G and report['zero_total_observation_count']==len(zero)
  assert report['interval_method']=='type7_percentile_95' and report['covariance_convention']=='whole_cluster_pairs_b_minus_1'
  assert report['covariance_parameter_names']==NAMES and [c['name'] for c in report['coefficients']]==NAMES
  assert len(report['replicate_cluster_indices'])==len(report['replicate_observation_counts'])==len(report['replicate_coefficients'])==500
  oracle_path=Path(__file__).resolve().parents[2]/'crates/tabdat-stats/tests/fixtures/bootstrap/draws.py'
  spec=importlib.util.spec_from_file_location('bootstrap_draws',oracle_path);oracle=importlib.util.module_from_spec(spec);spec.loader.exec_module(oracle)
  stream=oracle.stream(10001);zone=oracle.MASK-oracle.MASK%G;draws=[]
  for r in range(500):
    expected=[];steps=0
    while len(expected)<G:
      word=next(stream);steps+=1;assert steps<=G+1024
      if word<zone: expected.append(word%G)
    assert report['replicate_cluster_indices'][r]==expected,(r,'draws')
    assert report['replicate_observation_counts'][r]==sum(sizes[i] for i in expected)
    assert len(report['replicate_coefficients'][r])==len(NAMES)
    draws.extend((r+1,j+1,index,labels[index]) for j,index in enumerate(expected))
  return source,complete,selected,totals,labels,sizes,events,report,draws


def write_oracle(path,draws):
  with path.open('x',newline='') as f:
    writer=csv.writer(f,lineterminator='\n');writer.writerow(['replicate','copy_id','cluster_index','commune']);writer.writerows(draws)


def check(native,ref,raw,mode):
  source,complete,selected,totals,labels,sizes,events,report,draws=load(native,raw,mode)
  original=rows(ref/'source.csv');assert len(original)==len(source)
  by_id={int(r['source_row']):r for r in original}
  for a in source:
    b=by_id[int(a['source_row'])];assert set(a)==set(b)==set(SOURCE)
    for key in a:
      if a[key]=='': assert b[key]==''
      else: assert float(a[key])==float(b[key]),(a['source_row'],key,a[key],b[key])
  groups=rows(ref/'groups.csv')
  assert [(int(r['commune']),int(r['observations'])) for r in groups]==[(int(g),n) for g,n in zip(labels,sizes)]
  fits=rows(ref/'fits.csv');summary=rows(ref/'summary.csv');assert len(fits)==500 and len(summary)==100
  max_fit=max_summary=max_prediction=0.0
  for i,fit in enumerate(fits):
    assert int(fit['replicate'])==i+1 and int(fit['sample_n'])==report['replicate_observation_counts'][i]
    for j,name in enumerate(NAMES):
      a=report['replicate_coefficients'][i][j];b=float(fit[name]);near(a,b);max_fit=max(max_fit,abs(a-b))
  assert len(report['covariance_matrix'])==10
  for index,row in enumerate(summary):
    i,j=divmod(index,10)
    assert (int(row['parameter_index']),int(row['covariance_index']))==(i+1,j+1)
    assert (row['parameter'],row['covariance_parameter'])==(NAMES[i],NAMES[j])
    c=report['coefficients'][i]
    for key in ('estimate','bootstrap_mean','bias','standard_error','ci_lower','ci_upper'):
      a=c[key];b=float(row[key]);near(a,b);max_summary=max(max_summary,abs(a-b))
    a=report['covariance_matrix'][i][j];b=float(row['covariance']);near(a,b);max_summary=max(max_summary,abs(a-b))
  original_fit=[e['data'] for e in events if e['result_type']=='ConditionalPoissonResult'];assert len(original_fit)==1
  native_fit=original_fit[0];assert native_fit['observation_count']==len(selected)
  assert native_fit['estimator']=='conditional_poisson_fe' and native_fit['group_count']==len(labels)
  assert [c['name'] for c in native_fit['coefficients']]==NAMES
  for a,b in zip(native_fit['coefficients'],report['coefficients']): near(a['value'],b['estimate'])
  predictions=rows(native/'individual_conditional_fe_bootstrap_predictions.csv');expected=rows(ref/'predictions.csv')
  assert len(predictions)==len(expected)==len(complete)
  fitted_totals=defaultdict(float)
  for a,b,s in zip(predictions,expected,complete):
    assert set(a)==set(b)==set(s)|{'selected_commune_total','included_in_fe','fe_mean','fe_xb','fe_residual'}
    for key in s: assert float(a[key])==float(b[key])==float(s[key]),(s['source_row'],key)
    assert int(a['selected_commune_total'])==int(b['selected_commune_total'])==totals[s['commune']]
    included=totals[s['commune']]>0
    assert a['included_in_fe'].lower()==b['included_in_fe'].lower()==str(included).lower()
    near(a['fe_xb'],b['fe_xb']);max_prediction=max(max_prediction,abs(float(a['fe_xb'])-float(b['fe_xb'])))
    for key in ('fe_mean','fe_residual'):
      if included:
        near(a[key],b[key]);max_prediction=max(max_prediction,abs(float(a[key])-float(b[key])))
      else: assert a[key]==b[key]==''
    if included:
      near(float(a['fe_mean'])+float(a['fe_residual']),a['pharvis'])
      fitted_totals[s['commune']]+=float(a['fe_mean'])
  for group,value in fitted_totals.items(): near(value,totals[group])
  assert events[-1]['result_type']=='ExportResult' and events[-1]['data']['dataset']['row_count']==len(complete)
  print(f'validated {len(source)} source/{len(complete)} prediction rows, {len(draws)} exact ordered draws, 500 R conditional-score refits and full 10x10 covariance;')
  print(f'max absolute differences: fits {max_fit:.3g}, summary {max_summary:.3g}, predictions {max_prediction:.3g}')


def main():
  if len(sys.argv) not in (5,6) or sys.argv[1] not in ('oracle','check'):
    raise SystemExit('usage: check_individual_conditional_fe_bootstrap.py oracle native-dir source oracle.csv [--dta | --synthetic]\n       check_individual_conditional_fe_bootstrap.py check native-dir reference-dir source [--dta | --synthetic]')
  mode=sys.argv[5] if len(sys.argv)==6 else 'ascii';assert mode in ('ascii','--dta','--synthetic')
  if sys.argv[1]=='oracle':
    native,raw,out=map(Path,sys.argv[2:5]);*_,draws=load(native,raw,mode);write_oracle(out,draws)
    print(f'validated and wrote {len(draws)} independently replayed ordered draws')
  else:
    native,ref,raw=map(Path,sys.argv[2:5]);check(native,ref,raw,mode)


if __name__=='__main__': main()
