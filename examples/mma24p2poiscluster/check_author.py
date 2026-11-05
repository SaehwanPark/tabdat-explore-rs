#!/usr/bin/env python3
"""Separate DTA/author printed-precision investigation, not the ASCII oracle."""
import csv
from decimal import Decimal
import hashlib
import re
import sys
from pathlib import Path


def rows(path):
  with path.open(newline='') as f:
    return list(csv.DictReader(f))


def main():
  if len(sys.argv)!=6:
    raise SystemExit('usage: check_author.py vietnam_ex2.dta author.log dta.csv dta-reference-dir vietnam_individual.csv')
  dta,log,converted,ref,ascii_csv=map(Path,sys.argv[1:])
  assert hashlib.sha256(dta.read_bytes()).hexdigest()=='284a5dd732926fe6ee02157012b17d8751523b89f1dd2f652a6f4dd021f3dac3'
  assert hashlib.sha256(log.read_bytes()).hexdigest()=='b86983173c9c712d10df522a0d3ce566fd7c03b77d9f9739b9e776c44223d33e'
  a,b=rows(ascii_csv),rows(converted)
  assert len(a)==len(b)==27766
  maxdiff=0.0
  for i,(x,y) in enumerate(zip(a,b),1):
    assert set(x)==set(y) and int(x['source_row'])==i==int(y['source_row'])
    for key in x:
      if not x[key] or not y[key]:
        assert x[key]==y[key]==''
      else:
        delta=abs(float(x[key])-float(y[key]))
        if key in ('lnhhexp','age'):
          assert delta<=5e-7
          maxdiff=max(maxdiff,delta)
        else: assert delta==0, (i,key,delta)
  assert maxdiff>0
  sections=log.read_text().split('. poisson PHARVIS $XLISTPOISSON')[1:4]
  assert len(sections)==3
  refs=rows(ref/'reference.csv'); mismatch=[]; matched=0
  names=['intercept','lnhhexp','insurance','sex','age','married','illdays','actdays','injury','illness','educ']
  for mode,section in zip(('classical','robust','cluster'),sections):
    printed={}
    for line in section.split('. estimates store')[0].splitlines():
      match=re.match(r'^\s*(LNHHEXP|INSURANCE|SEX|AGE|MARRIED|ILLDAYS|ACTDAYS|INJURY|ILLNESS|EDUC|_cons)\s*\|\s*([-+\d.]+)\s+([-+\d.]+)',line,re.I)
      if match:
        name=match.group(1).lower().replace('_cons','intercept')
        assert name not in printed
        printed[name]=[match.group(2),match.group(3)]
    assert set(printed)==set(names)
    for i,name in enumerate(names):
      for j,field in enumerate(('value','standard_error')):
        selected=[r for r in refs if r['mode']==mode and r['quantity']=='coefficient_'+field and int(r['i'])==i]
        assert len(selected)==1
        text=printed[name][j]
        # Compare with the actual half-unit of the logged decimal's last digit.
        half=Decimal('0.5')*Decimal(10)**(-len(text.split('.')[-1]))
        diff=abs(Decimal(selected[0]['value'])-Decimal(text))
        if diff<=half: matched+=1
        else: mismatch.append((mode,name,field,diff,half))
  assert matched==65 and len(mismatch)==1
  mode,name,field,diff,half=mismatch[0]
  assert (mode,name,field)==('cluster','actdays','standard_error')
  assert Decimal(0)<diff-half<Decimal('0.000000001')
  print(f'65 author facts match their printed rounding; unresolved cluster ACTDAYS SE exceeds its last-digit boundary by {diff-half}; DTA/ASCII max {maxdiff:g}')


if __name__=='__main__':
  main()
