#!/usr/bin/env python3
"""Audit combined residual fits/summary against raw MOM, R and author log."""
import argparse
import csv
from decimal import Decimal
import hashlib
import json
import math
from pathlib import Path
import re

SOURCE='lnhr lnwg kids ageh agesq disab id year'.split()
RAW=['source_row']+SOURCE
VARIABLES=['fe_residual','lnhr','lnwg','pooled_residual','re_e']
MAP={'lnhr':'lnhr','lnwg':'lnwg','upols':'pooled_residual','ufe':'fe_residual','ure':'re_e'}


def records(path):
  with path.open(newline='') as f:return list(csv.DictReader(f))


def pointer(data,path):
  for key in path.split('/')[1:]:data=data[int(key)] if isinstance(data,list) else data[key]
  return data


def close(actual,expected,label):
  tolerance=abs(expected)*1e-8 if str(label).endswith('p_value') else 1e-9
  assert math.isfinite(actual) and math.isfinite(expected) and abs(actual-expected)<=tolerance,(label,actual,expected)


def author_values(path):
  assert hashlib.sha256(path.read_bytes()).hexdigest()=='925aff11176cb78bbacf71abc69c2bef4876bdcc0e5544c001c2df4f5f8f56b5'
  text=path.read_text();result={}
  begin=text.index('. reshape wide lnhr lnwg upols');start=text.index('. summarize\n',begin)
  block=text[start:text.index('. ** OBTAIN THE VARIOUS CORRELATIONS',start)]
  for line in block.splitlines():
    m=re.match(r'^\s*(lnhr|lnwg|upols|ufe|ure)(19\d\d)\s*\|\s*(\S+)\s+(\S+)\s+(\S+)\s+(\S+)\s+(\S+)\s*$',line)
    if m:
      var,year,n,*values=m.groups();key=('year',int(year),MAP[var]);assert key not in result
      result[key]=(int(n),values)
  block=text[text.index('. summarize upols ufe ure'):text.index('. save mom3, replace')]
  for line in block.splitlines():
    m=re.match(r'^\s*(upols|ufe|ure)\s*\|\s*(\S+)\s+(\S+)\s+(\S+)\s+(\S+)\s+(\S+)\s*$',line)
    if m:
      var,n,*values=m.groups();key=('overall',None,MAP[var]);assert key not in result
      result[key]=(int(n),values)
  assert len(result)==53
  return result


def audit(native,reference,raw,author_log=None,synthetic=False):
  if not synthetic:
    assert hashlib.sha256(raw.read_bytes()).hexdigest()=='2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77'
    assert author_log is not None
  values=[float(v) for v in raw.read_text().split()];assert len(values)%8==0 and all(math.isfinite(v) for v in values)
  source=[dict(source_row=i//8+1,**dict(zip(SOURCE,values[i:i+8]))) for i in range(0,len(values),8)]
  n=len(source);g=len({r['id'] for r in source});assert 3<=g<=532 and n==10*g
  events=[json.loads(l) for l in (native/'residual_summary.jsonl').read_text().splitlines()]
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  select=lambda kind:[e['data'] for e in events if e['result_type']==kind]
  pool,=select('RegressionResult');fe,=select('FixedEffectsResult');re_model,=select('RandomEffectsResult')
  assert pool['outcome']=='lnhr' and pool['predictors']==['lnwg'] and pool['covariance']=='nonrobust' and pool['include_intercept']
  assert fe['outcome']=='lnhr' and fe['predictors']==['lnwg'] and fe['estimator']=='within_fixed_effects'
  assert fe['entity_variable']=='id' and fe['time_variable']=='year' and fe['unit_effect_normalization']=='estimation_observation_weighted_zero_mean'
  assert re_model['outcome']=='lnhr' and re_model['predictors']==['lnwg'] and re_model['estimator']=='balanced_swamy_arora_random_effects_gls'
  assert re_model['variance_component_method']=='balanced_swamy_arora' and re_model['covariance']=='nonrobust' and re_model['reference_distribution']=='normal'
  for fit in [pool,fe,re_model]:assert fit['sample']==dict(total_observations=n,dropped_observations=0)
  refs=records(reference/'reference.csv');assert len(refs)==len({(r['result_type'],r['path']) for r in refs})==64
  for row in refs:close(pointer({'RegressionResult':pool,'FixedEffectsResult':fe,'RandomEffectsResult':re_model}[row['result_type']],row['path']),float(row['value']),row['path'])
  assert [e['result_type'] for e in events][-6:]==['TableResult','ActivateResult','PredictionResult','TableResult','ActivateResult','ExportResult']
  assert 'descriptive, not a test or covariance correction' in json.dumps(events)
  retained=[v for v in select('TableResult') if v['headers']==['retained_re_prediction_rows']];assert len(retained)==1 and retained[0]['rows']==[[n]]
  exports=select('ExportResult');assert len(exports)==2
  for e,name,count in zip(exports,['panel_joint_residual_rows.csv','panel_joint_residual_summary.csv'],[n,55]):
    assert e['path']==e['dataset']['path']==name and e['dataset']['row_count']==count
  actual=records(native/'panel_joint_residual_rows.csv');expected=records(reference/'rows.csv')
  assert len(actual)==len(expected)==n and list(actual[0])==list(expected[0]) and len(actual[0])==30
  person={}
  for i,(a,b,original) in enumerate(zip(actual,expected,source),1):
    for key in RAW:assert float(a[key])==float(b[key])==(i if key=='source_row' else original[key]),(i,key)
    for key in a.keys()-set(RAW):close(float(a[key]),float(b[key]),key)
    val=lambda key:float(a[key]);person.setdefault(original['id'],[]).append(a)
    close(val('lnhr')-val('pooled_xb'),val('pooled_residual'),'pooled identity')
    close(val('fe_xb')+val('fe_u'),val('fe_xbu'),'FE fitted identity')
    close(val('lnhr')-val('fe_xbu'),val('fe_residual'),'FE residual identity')
    close(val('lnhr')-val('population_xb'),val('population_residual'),'RE population identity')
    close(val('shrink'),1-(1-val('theta'))**2,'RE shrink identity')
    close(val('re_unit_effect'),val('shrink')*val('mean_population_residual'),'RE unit identity')
    close(val('population_xb')+val('re_unit_effect'),val('conditional_xbu'),'RE conditional identity')
    close(val('population_residual')-val('re_unit_effect'),val('re_e'),'RE e identity')
    for key in ['sigma_e_squared','raw_sigma_u_squared','sigma_u_squared','theta']:close(val(key),re_model[key],key)
  assert len(person)==g
  for group in person.values():
    assert len(group)==10
    mean=math.fsum(float(r['population_residual']) for r in group)/10
    assert abs(math.fsum(float(r['fe_residual']) for r in group))<=1e-9
    for r in group:close(float(r['mean_population_residual']),mean,'RE person mean')
  actual=records(native/'panel_joint_residual_summary.csv');expected=records(reference/'summary.csv')
  headers=['scope','year','variable','n','mean','std_dev','minimum','maximum'];tables=[v for v in select('TableResult') if v['headers']==headers]
  assert len(tables)==1 and len(tables[0]['rows'])==55
  assert len(actual)==len(expected)==55 and list(actual[0])==list(expected[0])==headers
  for i,(a,b,row) in enumerate(zip(actual,expected,tables[0]['rows'])):
    scope='overall' if i<5 else 'year';year=None if i<5 else 1979+(i-5)//5;variable=VARIABLES[i%5]
    assert a['scope']==b['scope']==row[0]==scope and a['variable']==b['variable']==row[2]==variable
    if year is None:assert a['year']==b['year']=='' and row[1] is None
    else:assert float(a['year'])==float(b['year'])==row[1]==year
    assert float(a['n'])==float(b['n'])==row[3]==(n if year is None else g)
    for j,key in enumerate(headers[4:],4):close(float(a[key]),float(b[key]),key);close(row[j],float(a[key]),key)
  if author_log is not None:
    historical=records(reference/'historical_float32.csv');assert len(historical)==55
    hist={(r['scope'],None if r['year']=='' else int(float(r['year'])),r['variable']):r for r in historical};assert len(hist)==55
    printed=author_values(author_log);checked=0
    for key,(count,vals) in printed.items():
      r=hist[key];assert int(r['n'])==count
      for field,value in zip(['mean','std_dev','minimum','maximum'],vals):
        actual_value=float(r[field]);expected_value=float(value)
        if key==('overall',None,'re_e') and field=='mean':
          assert abs(actual_value-expected_value)<=1e-12
          continue  # Tiny historical RE rounding discrepancy, not exact printed parity.
        tolerance=float(Decimal('0.5')*(Decimal(10)**Decimal(value).as_tuple().exponent))
        assert abs(actual_value-expected_value)<=tolerance,(key,field,actual_value,value,tolerance)
        checked+=1
    assert checked==211
  print(f'PASS {"synthetic" if synthetic else "original MOM"}: 64 R fit fields, {n}x30 source/prediction cells, 55 summary/table rows'+(', 53 author counts/211 printed numeric facts; overall RE mean separately within 1e-12' if author_log else ''))


if __name__=='__main__':
  parser=argparse.ArgumentParser(description=__doc__)
  for name in ['native','reference','raw']:parser.add_argument(name,type=Path)
  parser.add_argument('--author-log',type=Path);parser.add_argument('--synthetic',action='store_true')
  args=parser.parse_args();audit(args.native,args.reference,args.raw,args.author_log,args.synthetic)
