#!/usr/bin/env python3
"""Separate DTA/printed-author check; not the strict ASCII numerical oracle."""
import csv
from decimal import Decimal
import hashlib
import json
from pathlib import Path
import re
import sys


def main():
  if len(sys.argv)!=5:
    raise SystemExit('usage: check_conditional_fe_author.py vietnam_ex2.dta author.log reference-dir native-result.jsonl')
  dta,log,ref,native=map(Path,sys.argv[1:])
  assert hashlib.sha256(dta.read_bytes()).hexdigest()=='284a5dd732926fe6ee02157012b17d8751523b89f1dd2f652a6f4dd021f3dac3'
  assert hashlib.sha256(log.read_bytes()).hexdigest()=='b86983173c9c712d10df522a0d3ce566fd7c03b77d9f9739b9e776c44223d33e'
  with (ref/'coefficients.csv').open(newline='') as f: reference=list(csv.DictReader(f))
  events=[json.loads(line) for line in native.read_text().splitlines()]
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  fits=[e['data'] for e in events if e['result_type']=='ConditionalPoissonResult']
  assert len(fits)==1
  fit=fits[0]
  assert fit['estimator']=='conditional_poisson_fe' and fit['observation_count']==27671 and fit['group_count']==193
  assert fit['covariance']=='inverse_conditional_information'
  section=log.read_text().split('. xtpois PHARVIS $XLISTPOISSON, fe',1)[1].split('. estimates store',1)[0]
  printed={}
  for line in section.splitlines():
    m=re.match(r'^\s*(LNHHEXP|INSURANCE|SEX|AGE|MARRIED|ILLDAYS|ACTDAYS|INJURY|ILLNESS|EDUC)\s*\|\s*([-+\d.]+)\s+([-+\d.]+)',line,re.I)
    if m:
      name=m[1].lower();assert name not in printed
      printed[name]=(m[2],m[3])
  assert len(printed)==len(reference)==len(fit['coefficients'])==10
  for label,report in [('R dummy-Poisson', {r['parameter']:r for r in reference}),('native conditional FE',{r['name']:r for r in fit['coefficients']})]:
    assert set(report)==set(printed)
    for name,values in printed.items():
      for key,text in zip(('value','standard_error'),values):
        half=Decimal('.5')*Decimal(10)**(-len(text.split('.')[-1]))
        assert abs(Decimal(str(report[name][key]))-Decimal(text))<=half,(label,name,key,text,report[name][key])
    print('20 printed author coefficient/SE facts match',label,'on pinned DTA')


if __name__=='__main__':
  main()
