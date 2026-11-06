#!/usr/bin/env python3
"""Compare original DTA Poisson points, not the B50 historical uncertainty."""
import csv
from decimal import Decimal
import hashlib
import json
from pathlib import Path
import re
import sys


def main():
  if len(sys.argv) not in (4,5):
    raise SystemExit('usage: check_individual_pooled_bootstrap_author.py vietnam_ex2.dta author.log reference-dir [native-dir]')
  dta,log,ref=map(Path,sys.argv[1:4])
  assert hashlib.sha256(dta.read_bytes()).hexdigest()=='284a5dd732926fe6ee02157012b17d8751523b89f1dd2f652a6f4dd021f3dac3'
  assert hashlib.sha256(log.read_bytes()).hexdigest()=='b86983173c9c712d10df522a0d3ce566fd7c03b77d9f9739b9e776c44223d33e'
  section=log.read_text().split('. bootstrap "poisson PHARVIS $XLISTPOISSON" _b, cluster(COMMUNE) reps($breps) level(95)',1)[1].split('. matrix tpois ',1)[0]
  printed={}
  for line in section.splitlines():
    m=re.match(r'^\s*b_(LNHHEXP|INSURANCE|SEX|AGE|MARRIED|ILLDAYS|ACTDAYS|INJURY|ILLNESS|EDUC|cons)\s*\|\s*50\s+([-+\d.]+)',line,re.I)
    if m:
      key=m.group(1).lower();key='intercept' if key=='cons' else key;assert key not in printed;printed[key]=m.group(2)
  names=['intercept','lnhhexp','insurance','sex','age','married','illdays','actdays','injury','illness','educ'];assert set(printed)==set(names)
  with (ref/'summary.csv').open(newline='') as f:summary=list(csv.DictReader(f))
  assert len(summary)==121
  values=[('independent R',[r['estimate'] for r in summary[::11]])]
  if len(sys.argv)==5:
    events=[json.loads(x) for x in (Path(sys.argv[4])/'result.jsonl').read_text().splitlines()]
    reports=[e['data'] for e in events if e['result_type']=='ClusterBootstrapPoissonResult'];assert len(reports)==1
    assert [c['name'] for c in reports[0]['coefficients']]==names
    values.append(('native on precise DTA',[c['estimate'] for c in reports[0]['coefficients']]))
  for label,coefficients in values:
    for name,value in zip(names,coefficients):
      text=printed[name];places=len(text.split('.')[-1]) if '.' in text else 0
      assert abs(Decimal(str(value))-Decimal(text))<=Decimal('.5')*Decimal(10)**(-places),(name,value,text)
    print(f'11 printed original Poisson coefficient facts match {label}; historical B50 uncertainty is not compared')


if __name__=='__main__':
  main()
