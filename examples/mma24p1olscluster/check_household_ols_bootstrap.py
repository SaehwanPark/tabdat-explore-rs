#!/usr/bin/env python3
"""Audit the original-data, full-cluster bootstrap and its independent draw stream."""
import csv
from decimal import Decimal
import hashlib
import importlib.util
import json
import math
from pathlib import Path
import shlex
import sys


def rows(path):
  with path.open(newline='') as f:
    return list(csv.DictReader(f))


def near(a,b):
  a,b=float(a),float(b)
  assert math.isfinite(a) and math.isfinite(b) and abs(a-b)<=1e-9,(a,b)


def main():
  if len(sys.argv) not in (4,5) or (len(sys.argv)==5 and sys.argv[4]!='--dta'):
    raise SystemExit('usage: check_household_ols_bootstrap.py native-dir reference-dir vietnam_ex1.asc|dta [--dta]')
  native,ref,raw=map(Path,sys.argv[1:4]);dta=len(sys.argv)==5
  expected_hash=('0ea6639f93a5bc3f53bcbd26b6865ad6908a04f41ab04d90e7c9aa4b7e9787bf' if dta else
                 '74cb37e8c14609e382d334e368a54f63bccd4e952d02bcbf535baf19115d4061')
  assert hashlib.sha256(raw.read_bytes()).hexdigest()==expected_hash
  source=rows(native/'vietnam.csv'); original=rows(ref/'input.csv')
  assert len(source)==len(original)==5999
  tokens=None if dta else shlex.split(raw.read_text(encoding='ascii'))
  if tokens is not None: assert len(tokens)==5999*8
  farms={'non farm':0,'farm':1}
  education={'Never':0,'Nhatre':1,'<cap I':2,'Cap I':3,'Cap II':4,'Cap III':5,
             'Nghe SC':6,'THCN':7,'DHDC':8,'DHCD':9,'Thac sy':10,'PTS':11,'TS':12}
  required={'source_row','lnexp12m','lnhhexp','age','sex','hhsize','farm','educ','commune'}
  for i,(a,b) in enumerate(zip(source,original)):
    assert set(a)==set(b)==required and int(a['source_row'])==int(b['source_row'])==i+1
    for key in a:
      if not a[key]: assert b[key]==''
      else: assert float(a[key])==float(b[key]),(i,key,a[key],b[key])
    if tokens is not None:
      y,age,sex,size,farm,educ,x,group=tokens[8*i:8*i+8]
      values={'lnexp12m':'' if y=='.' else y,'age':age,'sex':sex,'hhsize':size,
              'farm':farms[farm],'educ':education[educ],'lnhhexp':x,'commune':group}
      for key,value in values.items():
        if value=='': assert a[key]==''
        else: assert Decimal(a[key])==Decimal(str(value)),(i,key)
  selected=[r for r in source if r['lnexp12m']!='']
  assert len(selected)==5006 and sum(r['lnexp12m']=='' for r in source)==993
  groups=rows(ref/'groups.csv');counts=rows(ref/'replicate_counts.csv');draws=rows(ref/'draws.csv')
  seen={}
  for row in selected: seen[row['commune']]=seen.get(row['commune'],0)+1
  assert len(seen)==len(groups)==194 and len(counts)==len(draws)==500
  assert [(float(g['commune']),int(g['observation_count'])) for g in groups]==[(float(k),v) for k,v in seen.items()]
  sizes=list(seen.values())
  assert min(sizes)==1 and max(sizes)==39
  # Independently regenerate each complete stream; no data or native output seeds the oracle.
  module=Path(__file__).resolve().parents[2]/'crates/tabdat-stats/tests/fixtures/bootstrap/draws.py'
  spec=importlib.util.spec_from_file_location('bootstrap_draws',module);rng=importlib.util.module_from_spec(spec);spec.loader.exec_module(rng)
  calculated=[];draw_count=0
  for r,(actual,expected,stream) in enumerate(zip(draws,counts,rng.draws(10001,194,500)),1):
    index=[int(v) for v in actual['indices'].split(':')]
    assert int(actual['replication'])==int(expected['replication'])==r and index==stream and len(index)==194
    n=sum(sizes[i] for i in index);assert n==int(expected['observations']);calculated.append(n);draw_count+=len(index)
  assert draw_count==97000
  events=[json.loads(line) for line in (native/'result.jsonl').read_text().splitlines()]
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  assert [e['result_type'] for e in events]==['LoadResult','IsidResult','TableResult','TableResult','SqlCreateResult',
    'RegressionResult','ClusterBootstrapOlsResult','PredictionResult','PredictionResult','SummarizeResult','TableResult','ExportResult']
  assert events[3]['data']['headers']==['raw_rows','retained_rows','excluded_rows']
  assert events[3]['data']['rows']==[[5999,5006,993]]
  fit=next(e['data'] for e in events if e['result_type']=='ClusterBootstrapOlsResult')
  names=['intercept','lnhhexp','age','sex','hhsize','farm','educ']
  assert fit['estimator']=='cluster_pairs_bootstrap_ols' and fit['seed']=='10001'
  assert fit['rng']=='xoshiro256++/splitmix64/u64-rejection-v1' and fit['interval_method']=='percentile_r_type7'
  assert fit['confidence_level']==95 and fit['include_intercept'] is True
  assert fit['outcome']=='lnexp12m' and fit['predictors']==names[1:]
  assert fit['cluster_order']=='first_seen_complete_case' and fit['cluster_sizes']==sizes
  assert fit['replicate_observation_counts']==calculated and fit['sample']=={'total_observations':5006,'dropped_observations':0}
  assert fit['covariance_parameter_names']==names and [c['name'] for c in fit['coefficients']]==names
  assert len(fit['covariance_matrix'])==7 and all(len(row)==7 for row in fit['covariance_matrix'])
  expected=rows(ref/'reference.csv');assert len(expected)==94 and len({r['path'] for r in expected})==94
  max_abs=0.0
  for r in expected:
    v=fit
    for key in r['path'].strip('/').split('/'):
      v=v[int(key)] if isinstance(v,list) else v[key]
    near(v,r['value']);max_abs=max(max_abs,abs(float(v)-float(r['value'])))
  actual=rows(native/'household_ols_bootstrap_predictions.csv');pred=rows(ref/'predictions.csv')
  assert len(actual)==len(pred)==len(selected)==5006
  for a,b,s in zip(actual,pred,selected):
    assert set(a)==set(b)==set(s)|{'pooled_fitted','pooled_residual'}
    assert a['source_row']==b['source_row']==s['source_row']
    for key in s: assert float(a[key])==float(b[key])==float(s[key]),(s['source_row'],key)
    near(a['pooled_fitted'],b['pooled_fitted']);near(a['pooled_residual'],b['pooled_residual'])
    near(float(a['pooled_fitted'])+float(a['pooled_residual']),a['lnexp12m'])
  assert events[-1]['data']['dataset']['row_count']==5006
  print('validated 94 R quantities, regenerated 97000 reference group draws, matched all 500 native replicate sizes, 5999 source and 5006 prediction rows;',
        'DTA' if dta else 'ASCII','maximum absolute discrepancy',max_abs)


if __name__=='__main__':
  main()
