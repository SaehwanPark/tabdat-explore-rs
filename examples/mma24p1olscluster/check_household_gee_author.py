#!/usr/bin/env python3
"""Check DTA household GEE against the exact printed author-log facts."""
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
    raise SystemExit('usage: check_household_gee_author.py vietnam_ex1.dta author.log dta-reference-dir [native-result.jsonl]')
  dta,log,ref=map(Path,sys.argv[1:4])
  assert hashlib.sha256(dta.read_bytes()).hexdigest()=='0ea6639f93a5bc3f53bcbd26b6865ad6908a04f41ab04d90e7c9aa4b7e9787bf'
  assert hashlib.sha256(log.read_bytes()).hexdigest()=='5e74c8de58eaf833b0070d68b289d61fad43831f3eb8b57c06818a6bf549837e'
  section=log.read_text().split('. xtreg LNEXP12M $XLISTLINEAR, pa',1)[1].split('. estimates store pa',1)[0]
  printed={}
  for line in section.splitlines():
    m=re.match(r'^\s*(LNHHEXP|AGE|SEX|HHSIZE|FARM|EDUC|_cons)\s*\|\s*([-+\d.]+)\s+([-+\d.]+)',line,re.I)
    if m:
      name=m.group(1).lower();assert name not in printed
      printed[name]=m.group(2,3)
  assert set(printed)=={'_cons','lnhhexp','age','sex','hhsize','farm','educ'}
  scale=re.search(r'Scale parameter:\s*([-+\d.]+)',section)
  wald=re.search(r'Wald chi2\(6\)\s*=\s*([-+\d.]+)',section)
  assert scale and wald
  with (ref/'reference.csv').open(newline='') as f:
    reference=list(csv.DictReader(f))
  assert len(reference)==198 and len({r['path'] for r in reference})==198
  reference={r['path']:r['value'] for r in reference}
  candidates=[('independent R/geeM',reference)]
  if len(sys.argv)==5:
    events=[json.loads(line) for line in Path(sys.argv[4]).read_text().splitlines()]
    assert all(e['schema_version']==1 and 'error' not in e for e in events)
    fits=[e['data'] for e in events if e['result_type']=='GaussianGeeResult'];assert len(fits)==1
    fit=fits[0];assert fit['observation_count']==5006 and fit['group_count']==194
    assert fit['estimator']=='gaussian_identity_exchangeable_gee' and fit['working_covariance']=='inverse_gee_bread'
    cs=fit['working']['coefficients'];assert len(cs)==7 and {c['name'] for c in cs}==set(printed)
    mapped={f"working.{c['name']}.{key}":c[field] for c in cs for key,field in [('estimate','value'),('standard_error','standard_error')]}
    mapped.update(phi=fit['phi'],**{'working.wald_statistic':fit['working']['wald_statistic']})
    candidates.append(('native GEE on converted DTA',mapped))
  for label,values in candidates:
    for name,(b,se) in printed.items():
      within_printed(values[f'working.{name}.estimate'],b)
      within_printed(values[f'working.{name}.standard_error'],se)
    within_printed(values['phi'],scale.group(1))
    within_printed(values['working.wald_statistic'],wald.group(1))
    print('16 printed author facts match',label)


if __name__=='__main__':
  main()
