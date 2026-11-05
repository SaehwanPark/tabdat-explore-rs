#!/usr/bin/env python3
"""Separate DTA/full-dummy-R vs displayed FE author-log investigation."""
import csv
from decimal import Decimal
import hashlib
import math
import re
import sys
from pathlib import Path


def main():
  if len(sys.argv)!=4:
    raise SystemExit('usage: check_household_fe_author.py vietnam_ex1.dta author.log dta-reference-directory')
  dta,log,ref=map(Path,sys.argv[1:])
  assert hashlib.sha256(dta.read_bytes()).hexdigest()=='0ea6639f93a5bc3f53bcbd26b6865ad6908a04f41ab04d90e7c9aa4b7e9787bf'
  assert hashlib.sha256(log.read_bytes()).hexdigest()=='5e74c8de58eaf833b0070d68b289d61fad43831f3eb8b57c06818a6bf549837e'
  with (ref/'reference.csv').open(newline='') as f: reference=list(csv.DictReader(f))
  def value(path):
    matches=[r['value'] for r in reference if r['kind']=='fit' and r['path']==path]
    assert len(matches)==1
    return Decimal(matches[0])
  section=log.read_text().split('. xtreg LNEXP12M $XLISTLINEAR, fe',1)[1].split('. estimates store',1)[0]
  found={}
  for line in section.splitlines():
    m=re.match(r'^\s*(LNHHEXP|AGE|SEX|HHSIZE|FARM|EDUC|_cons)\s*\|\s*([-+\d.]+)\s+([-+\d.]+)',line,re.I)
    if m:
      key=m.group(1).lower().replace('_cons','intercept')
      assert key not in found
      found[key]=[m.group(2),m.group(3)]
  names=['intercept','lnhhexp','age','sex','hhsize','farm','educ']
  assert set(names)==set(found)
  for i,name in enumerate(names):
    coef=value(f'/normalized_level/parameters/{i}')
    se=Decimal(str(math.sqrt(float(value(f'/normalized_level/covariance_matrix/{i}/{i}')))))
    for actual,text in zip((coef,se),found[name]):
      half=Decimal('0.5')*Decimal(10)**(-len(text.split('.')[-1]))
      assert abs(actual-Decimal(text))<=half,(name,actual,text,half)
  print('14 DTA full-dummy FE coefficient/SE values match author-log displayed precision')


if __name__=='__main__':
  main()
