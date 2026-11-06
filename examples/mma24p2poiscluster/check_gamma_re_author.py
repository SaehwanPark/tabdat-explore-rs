#!/usr/bin/env python3
"""Separate pinned DTA/printed-author audit; not the strict numerical oracle."""
import csv
from decimal import Decimal
import hashlib
import json
from pathlib import Path
import re
import sys


def half_printed(actual,text,label):
  digits=len(text.split('.')[-1]) if '.' in text else 0
  half=Decimal('.5')*Decimal(10)**(-digits)
  assert abs(Decimal(str(actual))-Decimal(text))<=half,(label,text,actual)


def main():
  if len(sys.argv)!=5:
    raise SystemExit('usage: check_gamma_re_author.py vietnam_ex2.dta author.log reference-dir native-result.jsonl')
  dta,log,ref,native=map(Path,sys.argv[1:])
  assert hashlib.sha256(dta.read_bytes()).hexdigest()=='284a5dd732926fe6ee02157012b17d8751523b89f1dd2f652a6f4dd021f3dac3'
  assert hashlib.sha256(log.read_bytes()).hexdigest()=='b86983173c9c712d10df522a0d3ce566fd7c03b77d9f9739b9e776c44223d33e'
  with (ref/'reference.csv').open(newline='') as f:r={row['path']:row['value'] for row in csv.DictReader(f)}
  assert len(r)==236
  events=[json.loads(line) for line in native.read_text().splitlines()]
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  fits=[e['data'] for e in events if e['result_type']=='GammaPoissonResult'];assert len(fits)==1
  fit=fits[0]
  assert fit['estimator']=='shared_gamma_poisson_re' and fit['observation_count']==27765 and fit['group_count']==194
  assert fit['covariance']=='inverse_full_joint_observed_information' and fit['random_effects_distribution']=='mean_one_gamma'
  section=log.read_text().split('. xtpois PHARVIS $XLISTPOISSON, re',1)[1].split('. estimates store',1)[0]
  printed={}
  for line in section.splitlines():
    m=re.match(r'^\s*(LNHHEXP|INSURANCE|SEX|AGE|MARRIED|ILLDAYS|ACTDAYS|INJURY|ILLNESS|EDUC|_cons|/lnalpha|alpha)\s*\|\s*([-+\d.]+)\s+([-+\d.]+)',line,re.I)
    if m:
      name=m[1].lower().lstrip('/');assert name not in printed;printed[name]=(m[2],m[3])
  names=['_cons','lnhhexp','insurance','sex','age','married','illdays','actdays','injury','illness','educ']
  assert len(printed)==13 and [c['name'] for c in fit['coefficients']]==names
  rr={name:{key:r[f'coefficients/{i}/{key}'] for key in ('value','standard_error')} for i,name in enumerate(names)}
  rr['lnalpha']={key:r[f'ln_alpha/{key}'] for key in ('value','standard_error')}
  rr['alpha']={'value':r['alpha'],'standard_error':r['alpha_standard_error']}
  nr={row['name']:row for row in fit['coefficients']};nr['lnalpha']=fit['ln_alpha'];nr['alpha']={'value':fit['alpha'],'standard_error':fit['alpha_standard_error']}
  extra={'log_likelihood':re.search(r'^Log likelihood\s*=\s*([-+\d.]+)',section,re.M)[1],
    'wald_statistic':re.search(r'Wald chi2\(10\)\s*=\s*([\d.]+)',section)[1],
    'dispersion_likelihood_ratio':re.search(r'chibar2\(01\)\s*=\s*([\d.]+)',section)[1]}
  for label,report,summary in [('R shared-gamma',rr,r),('native shared-gamma',nr,fit)]:
    assert set(report)==set(printed)
    for name,values in printed.items():
      for key,text in zip(('value','standard_error'),values):half_printed(report[name][key],text,(label,name,key))
    for key,text in extra.items():half_printed(summary[key],text,(label,key))
    print('29 printed coefficient/dispersion/SE/likelihood/Wald/LR facts match',label,'on pinned DTA')


if __name__=='__main__':
  main()
