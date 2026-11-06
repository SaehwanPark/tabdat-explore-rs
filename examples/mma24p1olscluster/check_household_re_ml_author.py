#!/usr/bin/env python3
"""Audit printed historical ML points without hiding known SE discrepancies."""
import csv
from decimal import Decimal
import hashlib
import json
import re
import sys
from pathlib import Path


def rounded_match(actual, printed):
  value=Decimal(printed)
  half=Decimal('.5')*Decimal(10)**value.as_tuple().exponent
  return abs(Decimal(str(actual))-value)<=half


def main():
  if len(sys.argv)!=5:
    raise SystemExit('usage: check_household_re_ml_author.py vietnam_ex1.dta author.log reference-dir native-result.jsonl')
  dta,log,ref,result=map(Path,sys.argv[1:])
  assert hashlib.sha256(dta.read_bytes()).hexdigest()=='0ea6639f93a5bc3f53bcbd26b6865ad6908a04f41ab04d90e7c9aa4b7e9787bf'
  assert hashlib.sha256(log.read_bytes()).hexdigest()=='5e74c8de58eaf833b0070d68b289d61fad43831f3eb8b57c06818a6bf549837e'
  with (ref/'reference.csv').open(newline='') as f:
    rows=list(csv.DictReader(f))
  assert len(rows)==210 and len({r['path'] for r in rows})==210
  values={r['path']:float(r['value']) for r in rows}
  events=[json.loads(x) for x in result.read_text().splitlines()]
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  fits=[e['data'] for e in events if e['result_type']=='UnbalancedRandomEffectsMlResult'];assert len(fits)==1
  fit=fits[0]; assert fit['observation_count']==5006 and fit['group_count']==194
  assert fit['estimator']=='unbalanced_gaussian_random_effects_ml'
  names=['intercept','lnhhexp','age','sex','hhsize','farm','educ']
  assert [c['name'] for c in fit['coefficients']]==names
  printed={}
  section=log.read_text().split('. xtreg LNEXP12M $XLISTLINEAR, mle',1)[1].split('. estimates store',1)[0]
  for line in section.splitlines():
    m=re.match(r'^\s*(LNHHEXP|AGE|SEX|HHSIZE|FARM|EDUC|_cons)\s*\|\s*([-+\d.]+)\s+([-+\d.]+)',line,re.I)
    if m:
      name=m[1].lower().replace('_cons','intercept');assert name not in printed;printed[name]=m.groups()[1:]
  assert set(printed)==set(names)
  exceptions={'lnhhexp','sex','hhsize','farm','intercept'}
  for i,name in enumerate(names):
    for j,key in enumerate(('value','standard_error')):
      text=printed[name][j]; observed=fit['coefficients'][i][key];reference=values[f'/coefficients/{i}/{key}']
      expected=not (key=='standard_error' and name in exceptions)
      assert rounded_match(observed,text)==expected,(name,key,text,observed)
      assert rounded_match(reference,text)==expected,(name,key,text,reference)
      if not expected: print('unresolved printed SE:',name,'author',text,'R',reference,'native',observed)
  checks=[('Log likelihood',r'Log likelihood\s*=\s*([-+\d.]+)',fit['log_likelihood'],values['/ml/log_likelihood']),
          ('slope LR',r'LR chi2\(6\)\s*=\s*([-+\d.]+)',fit['ml']['likelihood_ratio'],values['/ml/likelihood_ratio']),
          ('unit variance LR',r'chibar2\(01\)=\s*([-+\d.]+)',fit['ml']['unit_variance_test']['statistic'],values['/ml/unit_variance_test/statistic']),
          ('rho',r'^\s*rho\s*\|\s*([-+\d.]+)',fit['rho'],values['/rho'])]
  for name,pattern,actual,expected in checks:
    m=re.search(pattern,section,re.M);assert m,(name,pattern)
    assert rounded_match(actual,m[1]) and rounded_match(expected,m[1]),(name,m[1],actual,expected)
  for component in ('sigma_e','sigma_u'):
    m=re.search(r'^\s*/'+component+r'\s*\|\s*([-+\d.]+)\s+([-+\d.]+)',section,re.M);assert m
    key=component+'_squared';native=fit['ml']['variance_component_inference'][key];r_est=values['/ml/variance_component_inference/'+key+'/estimate'];r_se=values['/ml/variance_component_inference/'+key+'/standard_error']
    for label,estimate,se in [('native',native['estimate'],native['standard_error']),('R',r_est,r_se)]:
      sd=estimate**.5;sd_se=se/(2*sd)
      assert rounded_match(sd,m[1]),(component,label,m[1],sd)
      assert not rounded_match(sd_se,m[2]),(component,label,m[2],sd_se)
      print('unresolved printed component SE:',component,label,'author',m[2],'computed',sd_se)
  print('9/14 printed coefficients/SEs match; seven coefficients and two SEs match; five coefficient SEs and two component SEs remain unresolved. Log likelihood, two LR statistics, rho and both SD points match.')


if __name__=='__main__':
  main()
