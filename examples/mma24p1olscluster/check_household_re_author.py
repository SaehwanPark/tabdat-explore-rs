#!/usr/bin/env python3
"""Check the separate equal-commune hypothesis against Stata's printed RE row."""
import csv
from decimal import Decimal
import hashlib
import json
import re
import sys
from pathlib import Path


def main():
  if len(sys.argv) not in (4,5):
    raise SystemExit('usage: check_household_re_author.py vietnam_ex1.dta author.log dta-reference-directory [native-result.jsonl]')
  dta,log,ref=map(Path,sys.argv[1:4])
  assert hashlib.sha256(dta.read_bytes()).hexdigest()=='0ea6639f93a5bc3f53bcbd26b6865ad6908a04f41ab04d90e7c9aa4b7e9787bf'
  assert hashlib.sha256(log.read_bytes()).hexdigest()=='5e74c8de58eaf833b0070d68b289d61fad43831f3eb8b57c06818a6bf549837e'
  with (ref/'author_hypothesis.csv').open(newline='') as f:
    reports=list(csv.DictReader(f))
  assert len(reports)==7
  reports={r['parameter']:r for r in reports}
  section=log.read_text().split('. xtreg LNEXP12M $XLISTLINEAR, re',1)[1].split('. estimates store',1)[0]
  printed={}
  for line in section.splitlines():
    match=re.match(r'^\s*(LNHHEXP|AGE|SEX|HHSIZE|FARM|EDUC|_cons)\s*\|\s*([-+\d.]+)\s+([-+\d.]+)',line,re.I)
    if match:
      name=match.group(1).lower().replace('_cons','intercept')
      assert name not in printed
      printed[name]=[match.group(2),match.group(3)]
  assert set(printed)==set(reports)
  candidates=[('independent R equal-commune construction',reports)]
  if len(sys.argv)==5:
    events=[json.loads(line) for line in Path(sys.argv[4]).read_text().splitlines()]
    assert all(e['schema_version']==1 and 'error' not in e for e in events)
    fits=[e['data'] for e in events if e['result_type']=='UnbalancedRandomEffectsGlsResult']
    assert len(fits)==1
    fit=fits[0]
    assert fit['variance_component_method']=='equal_group_between'
    assert fit['observation_count']==5006 and fit['group_count']==194
    assert fit['covariance']=='nonrobust' and fit['reference_distribution']=='normal'
    native={r['name']:r for r in fit['coefficients']}
    assert len(fit['coefficients'])==7 and set(native)==set(printed)
    candidates.append(('native equal_group_between on DTA',native))
  for label,report in candidates:
    for name,values in printed.items():
      for key,text in zip(('value','standard_error'),values):
        half=Decimal('.5')*Decimal(10)**(-len(text.split('.')[-1]))
        assert abs(Decimal(str(report[name][key]))-Decimal(text))<=half,(label,name,key,text,report[name][key])
    print('14 printed author coefficients/SEs match',label)


if __name__=='__main__':
  main()
