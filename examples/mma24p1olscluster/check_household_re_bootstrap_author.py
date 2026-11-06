#!/usr/bin/env python3
"""Compare only original DTA RE points, not historical bootstrap uncertainty."""
import csv
from decimal import Decimal
import hashlib
import re
from pathlib import Path
import sys


def main():
  if len(sys.argv) not in (4,5):
    raise SystemExit('usage: check_household_re_bootstrap_author.py vietnam_ex1.dta author.log reference-dir [native-dir]')
  dta,log,ref=map(Path,sys.argv[1:4])
  assert hashlib.sha256(dta.read_bytes()).hexdigest()=='0ea6639f93a5bc3f53bcbd26b6865ad6908a04f41ab04d90e7c9aa4b7e9787bf'
  assert hashlib.sha256(log.read_bytes()).hexdigest()=='5e74c8de58eaf833b0070d68b289d61fad43831f3eb8b57c06818a6bf549837e'
  section=log.read_text().split('. bootstrap "xtreg LNEXP12M $XLISTLINEAR, re" _b, cluster(COMMUNE) reps($breps) level(95)',1)[1].split('. matrix tre',1)[0]
  printed={}
  for line in section.splitlines():
    m=re.match(r'^\s*b_(LNHHEXP|AGE|SEX|HHSIZE|FARM|EDUC|cons)\s*\|\s*500\s+([-+\d.]+)',line,re.I)
    if m:
      name=m.group(1).lower().replace('cons','_cons');assert name not in printed
      printed[name]=m.group(2)
  names=['_cons','lnhhexp','age','sex','hhsize','farm','educ'];assert set(printed)==set(names)
  candidates=[('independent R',ref/'summary.csv')]
  if len(sys.argv)==5: candidates.append(('native on converted DTA',Path(sys.argv[4])/'household_re_bootstrap_summary.csv'))
  for label,path in candidates:
    with path.open(newline='') as f:data=list(csv.DictReader(f))
    assert len(data)==49
    for i,name in enumerate(names):
      row=data[i*7];assert row['parameter']==name and row['covariance_parameter']=='_cons' and row['replications']=='500' and row['communes_per_draw']=='194'
      text=printed[name];places=len(text.split('.')[-1]) if '.' in text else 0
      assert abs(Decimal(row['estimate'])-Decimal(text))<=Decimal('.5')*Decimal(10)**(-places),(name,row['estimate'],text)
    print('7 printed original RE coefficient facts match',label,'(bootstrap SEs and intervals not compared)')


if __name__=='__main__': main()
