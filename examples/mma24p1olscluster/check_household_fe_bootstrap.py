#!/usr/bin/env python3
"""Bind every whole-commune draw, raw copied R QR fit and prediction to source."""
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
  if len(sys.argv) not in (4,5) or (len(sys.argv)==5 and sys.argv[4] not in ('--dta','--synthetic')):
    raise SystemExit('usage: check_household_fe_bootstrap.py native-dir reference-dir source [--dta | --synthetic]')
  native,ref,raw=map(Path,sys.argv[1:4]);mode=sys.argv[4] if len(sys.argv)==5 else 'ascii'
  if mode!='--synthetic':
    digest=('0ea6639f93a5bc3f53bcbd26b6865ad6908a04f41ab04d90e7c9aa4b7e9787bf' if mode=='--dta' else
            '74cb37e8c14609e382d334e368a54f63bccd4e952d02bcbf535baf19115d4061')
    assert hashlib.sha256(raw.read_bytes()).hexdigest()==digest
  source=rows(native/'vietnam.csv');original=rows(ref/'input.csv')
  assert len(source)==len(original) and (mode=='--synthetic' or len(source)==5999)
  rawtokens=shlex.split(raw.read_text(encoding='ascii')) if mode=='ascii' else None
  if rawtokens is not None: assert len(rawtokens)==len(source)*8
  farms={'non farm':0,'farm':1}
  education={'Never':0,'Nhatre':1,'<cap I':2,'Cap I':3,'Cap II':4,'Cap III':5,
             'Nghe SC':6,'THCN':7,'DHDC':8,'DHCD':9,'Thac sy':10,'PTS':11,'TS':12}
  required={'source_row','lnexp12m','lnhhexp','age','sex','hhsize','farm','educ','commune'}
  ids=set()
  for i,(a,b) in enumerate(zip(source,original)):
    assert set(a)==set(b)==required and int(a['source_row'])==int(b['source_row']) and int(a['source_row']) not in ids
    ids.add(int(a['source_row']))
    for key in a:
      if not a[key]: assert b[key]==''
      else: assert float(a[key])==float(b[key]),(i,key,a[key],b[key])
    if rawtokens is not None:
      y,age,sex,size,farm,educ,x,group=rawtokens[8*i:8*i+8]
      values={'lnexp12m':'' if y=='.' else y,'age':age,'sex':sex,'hhsize':size,
              'farm':farms[farm],'educ':education[educ],'lnhhexp':x,'commune':group}
      assert int(a['source_row'])==i+1
      for key,value in values.items():
        if value=='': assert a[key]==''
        else: assert Decimal(a[key])==Decimal(str(value)),(i,key)
  if mode=='--synthetic':
    rawsource=rows(raw);assert rawsource==source
  selected=sorted((r for r in source if r['lnexp12m']!=''),key=lambda r:int(r['source_row']))
  N=len(selected);sizes={}
  for row in selected:
    key=int(row['commune']);sizes[key]=sizes.get(key,0)+1
  groups=sorted(sizes);G=len(groups);assert 7<=G<=194 and max(sizes.values())<=39
  if mode!='--synthetic': assert (N,G,len(source)-N,min(sizes.values()),max(sizes.values()))==(5006,194,993,1,39)
  expected_groups=rows(ref/'groups.csv');assert [(int(g['commune']),int(g['observations'])) for g in expected_groups]==[(g,sizes[g]) for g in groups]
  draws=rows(native/'household_fe_bootstrap_draws.csv');assert len(draws)==500*G
  oracle_path=Path(__file__).resolve().parents[2]/'crates/tabdat-stats/tests/fixtures/bootstrap/draws.py'
  spec=importlib.util.spec_from_file_location('bootstrap_draws',oracle_path);oracle=importlib.util.module_from_spec(spec);spec.loader.exec_module(oracle)
  counts=[]
  for r in range(1,501):
    stream=oracle.stream(10000+r);zone=oracle.MASK-oracle.MASK%G;step=0;position=0;n=0
    while position<G:
      word=next(stream);step+=1
      assert step<=G+1024
      if word>=zone: continue
      row=draws[(r-1)*G+position];position+=1;group=groups[word%G]
      assert [int(row[k]) for k in ('replicate','copy_id','step','commune')]==[r,position,step,group]
      n+=sizes[group]
    assert n>G+6;counts.append(n)
  fits=rows(native/'household_fe_bootstrap_fits.csv');expected_fits=rows(ref/'fits.csv')
  summary=rows(native/'household_fe_bootstrap_summary.csv');expected_summary=rows(ref/'summary.csv')
  assert len(fits)==len(expected_fits)==500 and len(summary)==len(expected_summary)==49
  names=['_cons','lnhhexp','age','sex','hhsize','farm','educ'];fields=['intercept',*names[1:]]
  max_fit=max_summary=max_prediction=0.0
  for i,(a,b) in enumerate(zip(fits,expected_fits)):
    assert int(a['replicate'])==int(b['replicate'])==i+1 and int(a['sample_n'])==int(b['sample_n'])==counts[i]
    assert int(a['communes_per_draw'])==G and int(a['base_seed'])==10001
    for k in fields:
      near(a[k],b[k]);max_fit=max(max_fit,abs(float(a[k])-float(b[k])))
    for j,k in enumerate(fields): near(a['estimate_'+k],summary[j*7]['estimate'])
  for i,(a,b) in enumerate(zip(summary,expected_summary)):
    p,q=divmod(i,7);assert int(a['parameter_index'])==int(b['parameter_index'])==p+1
    assert int(a['covariance_index'])==int(b['covariance_index'])==q+1
    assert a['parameter']==b['parameter']==names[p] and a['covariance_parameter']==b['covariance_parameter']==names[q]
    for k in ('estimate','bootstrap_mean','bias','standard_error','ci_lower','ci_upper','covariance'):
      near(a[k],b[k]);max_summary=max(max_summary,abs(float(a[k])-float(b[k])))
    for k,v in [('replications','500'),('communes_per_draw',str(G)),('base_seed','10001'),
      ('rng','xoshiro256++/splitmix64/per_replicate_seed_plus_index/u64_rejection_v1'),
      ('method','whole_commune_pairs_refit_normalized_level_fe'),('interval_method','type7_percentile_95')]:
      assert a[k]==b[k]==v
  events=[json.loads(line) for line in (native/'result.jsonl').read_text().splitlines()]
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  fe=[e['data'] for e in events if e['result_type']=='FixedEffectsResult'];assert len(fe)==1
  fit=fe[0];assert fit['observation_count']==N and fit['absorbed_intercepts']==G
  assert [c['name'] for c in fit['coefficients']]==names[1:]
  near(fit['common_intercept'],summary[0]['estimate'])
  for i,c in enumerate(fit['coefficients'],1): near(c['value'],summary[i*7]['estimate'])
  exported=rows(native/'household_fe_bootstrap_predictions.csv');prediction=rows(ref/'predictions.csv')
  assert len(exported)==len(prediction)==N
  for a,b,s in zip(exported,prediction,selected):
    assert set(a)==set(b)==set(s)|{'fe_xb','fe_xbu','fe_u','fe_residual'}
    assert int(a['source_row'])==int(b['source_row'])==int(s['source_row'])
    for key in s: assert float(a[key])==float(b[key])==float(s[key]),(s['source_row'],key)
    for key in ('fe_xb','fe_xbu','fe_u','fe_residual'):
      near(a[key],b[key]);max_prediction=max(max_prediction,abs(float(a[key])-float(b[key])))
    near(float(a['fe_xb'])+float(a['fe_u']),a['fe_xbu'])
    near(float(a['fe_xbu'])+float(a['fe_residual']),a['lnexp12m'])
  assert events[-1]['result_type']=='ExportResult' and events[-1]['data']['dataset']['row_count']==N
  print(f'validated {len(source)} source/{N} prediction rows, {500*G} exact draws, 500 raw-copy R QR refits and full 7x7 covariance;')
  print(f'max absolute differences: fits {max_fit:.3g}, summary {max_summary:.3g}, predictions {max_prediction:.3g}')


if __name__=='__main__':
  main()
