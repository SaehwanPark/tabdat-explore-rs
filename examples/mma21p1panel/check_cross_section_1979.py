#!/usr/bin/env python3
"""Audit the 1979 cross-sectional OLS fit against raw MOM, R and author log."""
import argparse
import csv
import hashlib
import json
import math
from pathlib import Path
import re

SOURCE='lnhr lnwg kids ageh agesq disab id year'.split()
RAW=['source_row']+SOURCE


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
  text=path.read_text();block=text[text.index('. regress lnhr1979 lnwg1979'):text.index('. ************ (2) ANALYSIS')]
  assert 'Number of obs =     532' in block and 'F(  1,   530)' in block
  result={}
  for name,author in [('intercept','_cons'),('lnwg','lnwg1979')]:
    match=re.search(r'^\s*'+author+r'\s*\|\s*(.*)$',block,re.MULTILINE);assert match
    vals=match.group(1).split();assert len(vals)==6
    for field,value in zip(['value','standard_error','statistic','p_value','ci_lower','ci_upper'],vals):
      assert '.' in value and 'e' not in value.lower();decimals=len(value.split('.')[1]);result[(name,field)]=(float(value),0.5*10**(-decimals))
  return result


def audit(native,reference,raw,author_log=None,synthetic=False):
  if not synthetic:
    assert hashlib.sha256(raw.read_bytes()).hexdigest()=='2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77'
    assert author_log is not None
  values=[float(v) for v in raw.read_text().split()];assert len(values)%8==0 and all(math.isfinite(v) for v in values)
  source=[dict(source_row=i//8+1,**dict(zip(SOURCE,values[i:i+8]))) for i in range(0,len(values),8)]
  ids={r['id'] for r in source};g=len(ids);assert 3<=g<=532 and len(source)==10*g
  year=[r for r in source if r['year']==1979];assert len(year)==g and len({r['id'] for r in year})==g
  events=[json.loads(l) for l in (native/'cross_section_1979.jsonl').read_text().splitlines()]
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  select=lambda kind:[e['data'] for e in events if e['result_type']==kind]
  fit,=select('RegressionResult');lincom,=select('LincomResult');test,=select('TestResult')
  assert fit['outcome']=='lnhr' and fit['predictors']==['lnwg'] and fit['covariance']=='nonrobust' and fit['include_intercept']
  assert fit['sample']==dict(total_observations=g,dropped_observations=0)
  assert test['constraints']==['lnwg = 0'] and not test['is_chi2']
  refs=records(reference/'reference.csv');assert len(refs)==len({(r['result_type'],r['path']) for r in refs})==29
  for row in refs:close(pointer({'RegressionResult':fit,'LincomResult':lincom,'TestResult':test}[row['result_type']],row['path']),float(row['value']),row['path'])
  assert [e['result_type'] for e in events][-4:]==['SqlCreateResult','TestResult','LincomResult','ExportResult']
  assert 'not panel FE or a causal claim' in json.dumps(events)
  exports=select('ExportResult');assert len(exports)==1
  event=exports[0];name='panel_1979_cross_section_predictions.csv';assert event['path']==event['dataset']['path']==name and event['dataset']['row_count']==g
  actual=records(native/name);expected=records(reference/'rows.csv')
  assert len(actual)==len(expected)==g and list(actual[0])==list(expected[0]) and len(actual[0])==11
  for a,b,original in zip(actual,expected,year):
    for key in RAW:assert float(a[key])==float(b[key])==original[key],key
    close(float(a['lnhr'])-float(a['cross_fitted']),float(a['cross_residual']),'residual identity')
    for key in ['cross_fitted','cross_residual']:close(float(a[key]),float(b[key]),key)
  if author_log is not None:
    printed=author_values(author_log);historical=records(reference/'historical_float32.csv');assert len(historical)==12
    for row in historical:
      key=(row['name'],row['field']);value,tolerance=printed[key]
      assert abs(float(row['value'])-value)<=tolerance,(key,row['value'],value,tolerance)
  print(f'PASS {"synthetic" if synthetic else "original MOM"}: 29 QR fit/test/lincom fields, {g}x11 source/prediction cells'+(', 12 printed author facts under float32 storage hypothesis' if author_log else ''))


if __name__=='__main__':
  parser=argparse.ArgumentParser(description=__doc__)
  for name in ['native','reference','raw']:parser.add_argument(name,type=Path)
  parser.add_argument('--author-log',type=Path);parser.add_argument('--synthetic',action='store_true')
  args=parser.parse_args();audit(args.native,args.reference,args.raw,args.author_log,args.synthetic)
