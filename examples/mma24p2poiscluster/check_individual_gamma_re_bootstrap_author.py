#!/usr/bin/env python3
"""Check original point facts, not historic B50 uncertainty or repeated-ID semantics."""
import csv
from decimal import Decimal
import hashlib
import json
from pathlib import Path
import re
import sys


def main():
  if len(sys.argv)!=5:raise SystemExit('usage: check_individual_gamma_re_bootstrap_author.py source.dta author.log reference-dir native-result.jsonl')
  dta,log,ref,native=map(Path,sys.argv[1:])
  assert hashlib.sha256(dta.read_bytes()).hexdigest()=='284a5dd732926fe6ee02157012b17d8751523b89f1dd2f652a6f4dd021f3dac3'
  assert hashlib.sha256(log.read_bytes()).hexdigest()=='b86983173c9c712d10df522a0d3ce566fd7c03b77d9f9739b9e776c44223d33e'
  section=log.read_text().split('. bootstrap "xtpois PHARVIS $XLISTPOISSON, re" _b, cluster(COMMUNE)',1)[1].split('. matrix tpoisre',1)[0]
  assert re.search(r'Number of obs\s*=\s*27765',section) and re.search(r'N of clusters\s*=\s*194',section) and re.search(r'Replications\s*=\s*50',section)
  printed={}
  for line in section.splitlines():
    m=re.match(r'^\s*b_(LNHHEXP|INSURANCE|SEX|AGE|MARRIED|ILLDAYS|ACTDAYS|INJURY|ILLNESS|EDUC|cons|1cons)\s*\|\s*50\s+([-+\d.]+)',line,re.I)
    if m:
      key={'cons':'_cons','1cons':'lnalpha'}.get(m[1].lower(),m[1].lower());assert key not in printed;printed[key]=m[2]
  with (ref/'summary.csv').open(newline='') as f:reference=list(csv.DictReader(f))
  events=[json.loads(line) for line in native.read_text().splitlines()];assert all(e['schema_version']==1 and 'error' not in e for e in events)
  reports=[e['data'] for e in events if e['result_type']=='GammaPoissonBootstrapResult'];assert len(reports)==1
  report=reports[0];assert report['observation_count']==27765 and report['cluster_count']==194 and report['replications']==500
  r_points={r['parameter']:r['estimate'] for r in reference if r['parameter_index']==r['covariance_index']}
  native_points={r['name']:r['estimate'] for r in report['coefficients']}
  assert len(printed)==len(r_points)==len(native_points)==12 and set(printed)==set(r_points)==set(native_points)
  direct=log.read_text().split('. xtpois PHARVIS $XLISTPOISSON, re',1)[1].split('. estimates store',1)[0]
  assert re.search(r'^\s*INSURANCE\s*\|\s*-\.1675953\s',direct,re.M)
  assert re.search(r'^\s*ILLNESS\s*\|\s*\.5801872\s',direct,re.M)
  for label,values in [('R shared-gamma likelihood',r_points),('native shared-gamma RE',native_points)]:
    mismatches=[]
    for name,text in printed.items():
      half=Decimal('.5')*Decimal(10)**(-len(text.split('.')[-1]))
      if abs(Decimal(str(values[name]))-Decimal(text))>half:mismatches.append(name)
    assert mismatches==['insurance','illness'] and printed['insurance']=='-.1675954' and printed['illness']=='.5801873',(label,mismatches)
    assert abs(Decimal(str(values['insurance']))-Decimal('-.16759534936'))<Decimal('1e-10')
    assert abs(Decimal(str(values['illness']))-Decimal('.58018724159'))<Decimal('1e-10')
    print('10/12 printed bootstrap points match',label,'to half a printed digit;')
    print('INSURANCE: author bootstrap -.1675954, direct fit -.1675953, computed',values['insurance'])
    print('ILLNESS: author bootstrap .5801873, direct fit .5801872, computed',values['illness'])
    print('B50 uncertainty not compared; cause of differing author final digits is unverified')


if __name__=='__main__':main()
