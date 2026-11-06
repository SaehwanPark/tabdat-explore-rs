#!/usr/bin/env python3
"""Compare only original OLS points, not Stata's differently seeded bootstrap."""
import csv
from decimal import Decimal
import hashlib
import json
from pathlib import Path
import re
import sys


def within_printed(actual,printed):
  places=len(printed.split('.')[-1]) if '.' in printed else 0
  half=Decimal('.5')*Decimal(10)**(-places)
  assert abs(Decimal(str(actual))-Decimal(printed))<=half,(actual,printed,half)


def main():
  if len(sys.argv) not in (4,5):
    raise SystemExit('usage: check_household_ols_bootstrap_author.py vietnam_ex1.dta author.log dta-reference-dir [native-result.jsonl]')
  dta,log,ref=map(Path,sys.argv[1:4])
  assert hashlib.sha256(dta.read_bytes()).hexdigest()=='0ea6639f93a5bc3f53bcbd26b6865ad6908a04f41ab04d90e7c9aa4b7e9787bf'
  assert hashlib.sha256(log.read_bytes()).hexdigest()=='5e74c8de58eaf833b0070d68b289d61fad43831f3eb8b57c06818a6bf549837e'
  section=log.read_text().split('. bootstrap "reg LNEXP12M $XLISTLINEAR" _b, cluster(COMMUNE) reps($breps) level(95)',1)[1].split('. matrix tols',1)[0]
  printed={}
  for line in section.splitlines():
    m=re.match(r'^\s*b_(LNHHEXP|AGE|SEX|HHSIZE|FARM|EDUC|cons)\s*\|\s*500\s+([-+\d.]+)',line,re.I)
    if m:
      name=m.group(1).lower().replace('cons','intercept');assert name not in printed
      printed[name]=m.group(2)
  names=['intercept','lnhhexp','age','sex','hhsize','farm','educ']
  assert set(printed)==set(names)
  with (ref/'reference.csv').open(newline='') as f: expected=list(csv.DictReader(f))
  assert len(expected)==94 and len({r['path'] for r in expected})==94
  expected={r['path']:r['value'] for r in expected}
  candidates=[('independent R', {name:expected[f'/coefficients/{i}/estimate'] for i,name in enumerate(names)})]
  if len(sys.argv)==5:
    events=[json.loads(line) for line in Path(sys.argv[4]).read_text().splitlines()]
    assert all(e['schema_version']==1 and 'error' not in e for e in events)
    fits=[e['data'] for e in events if e['result_type']=='ClusterBootstrapOlsResult'];assert len(fits)==1
    fit=fits[0];assert fit['observation_count']==5006 and fit['cluster_count']==194 and fit['replications']==500
    assert fit['estimator']=='cluster_pairs_bootstrap_ols' and fit['seed']=='10001'
    assert [c['name'] for c in fit['coefficients']]==names
    candidates.append(('native on converted DTA',{c['name']:c['estimate'] for c in fit['coefficients']}))
  for label,values in candidates:
    for name,text in printed.items(): within_printed(values[name],text)
    print('7 printed original OLS coefficient facts match',label,'(bootstrap SEs and intervals not compared)')


if __name__=='__main__':
  main()
