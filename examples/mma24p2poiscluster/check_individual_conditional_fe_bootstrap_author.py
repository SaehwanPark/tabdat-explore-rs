#!/usr/bin/env python3
"""Check only printed original points, not historic B50 uncertainty or RNG."""
import csv
from decimal import Decimal
import hashlib
import json
from pathlib import Path
import re
import sys


def main():
  if len(sys.argv)!=5:
    raise SystemExit('usage: check_individual_conditional_fe_bootstrap_author.py source.dta author.log reference-dir native-result.jsonl')
  dta,log,ref,native=map(Path,sys.argv[1:])
  assert hashlib.sha256(dta.read_bytes()).hexdigest()=='284a5dd732926fe6ee02157012b17d8751523b89f1dd2f652a6f4dd021f3dac3'
  assert hashlib.sha256(log.read_bytes()).hexdigest()=='b86983173c9c712d10df522a0d3ce566fd7c03b77d9f9739b9e776c44223d33e'
  section=log.read_text().split('. bootstrap "xtpois PHARVIS $XLISTPOISSON, fe" _b, cluster(COMMUNE)',1)[1].split('. matrix tpoisfe',1)[0]
  assert re.search(r'Number of obs\s*=\s*27671',section)
  assert re.search(r'N of clusters\s*=\s*193',section)
  assert re.search(r'Replications\s*=\s*50',section)
  printed={}
  for line in section.splitlines():
    m=re.match(r'^\s*b_(LNHHEXP|INSURANCE|SEX|AGE|MARRIED|ILLDAYS|ACTDAYS|INJURY|ILLNESS|EDUC)\s*\|\s*50\s+([-+\d.]+)',line,re.I)
    if m:
      key=m[1].lower();assert key not in printed;printed[key]=m[2]
  with (ref/'summary.csv').open(newline='') as f: reference=list(csv.DictReader(f))
  events=[json.loads(line) for line in native.read_text().splitlines()]
  assert all(e['schema_version']==1 and 'error' not in e for e in events)
  reports=[e['data'] for e in events if e['result_type']=='ConditionalPoissonBootstrapResult'];assert len(reports)==1
  report=reports[0];assert report['observation_count']==27671 and report['cluster_count']==193 and report['replications']==500
  r_points={r['parameter']:r['estimate'] for r in reference if r['parameter_index']==r['covariance_index']}
  native_points={r['name']:r['estimate'] for r in report['coefficients']}
  assert len(printed)==len(r_points)==len(native_points)==10 and set(printed)==set(r_points)==set(native_points)
  direct=log.read_text().split('. xtpois PHARVIS $XLISTPOISSON, fe',1)[1].split('. estimates store',1)[0]
  assert re.search(r'^\s*MARRIED\s*\|\s*\.1606792\s',direct,re.M)
  for label,values in [('R conditional score',r_points),('native conditional FE',native_points)]:
    mismatches=[]
    for name,text in printed.items():
      half=Decimal('.5')*Decimal(10)**(-len(text.split('.')[-1]))
      if abs(Decimal(str(values[name]))-Decimal(text))>half: mismatches.append(name)
    assert mismatches==['married'] and printed['married']=='.1606793',(label,mismatches)
    assert abs(Decimal(str(values['married']))-Decimal('.16067924785'))<Decimal('1e-10')
    print('9/10 printed bootstrap points match',label,'to half a printed digit;')
    print('MARRIED discrepancy: author bootstrap .1606793, direct model .1606792, computed',values['married'])
    print('B50 bootstrap uncertainty not compared; cause of differing author final digits is unverified')


if __name__=='__main__': main()
