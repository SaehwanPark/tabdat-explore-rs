#!/usr/bin/env python3
"""Compare original DTA-based R fits to separately rounded author-log fields."""
import csv
import hashlib
import re
import sys
from pathlib import Path


def rows(path):
  with path.open(newline='') as f:
    return list(csv.DictReader(f))


def main():
  if len(sys.argv)!=6:
    raise SystemExit('usage: check_author.py vietnam_ex1.dta author.log dta.csv dta-reference-directory vietnam.csv')
  dta, log, converted, ref, ascii_csv = map(Path,sys.argv[1:])
  assert hashlib.sha256(dta.read_bytes()).hexdigest()=='0ea6639f93a5bc3f53bcbd26b6865ad6908a04f41ab04d90e7c9aa4b7e9787bf'
  assert hashlib.sha256(log.read_bytes()).hexdigest()=='5e74c8de58eaf833b0070d68b289d61fad43831f3eb8b57c06818a6bf549837e'
  a, b = rows(ascii_csv), rows(converted)
  assert len(a)==len(b)==5999
  maxdiff=0.0
  for index,(aa,bb) in enumerate(zip(a,b),1):
    assert set(aa)==set(bb) and int(aa['source_row'])==index
    for key in aa:
      if aa[key]=='' or bb[key]=='':
        assert aa[key]==bb[key]==''
      elif key in ('lnexp12m','lnhhexp'):
        diff=abs(float(aa[key])-float(bb[key]))
        assert diff <= 5e-6
        maxdiff=max(maxdiff,diff)
      else:
        assert float(aa[key])==float(bb[key]), (index,key)
  assert maxdiff>0
  sections=log.read_text().split('. regress LNEXP12M $XLISTLINEAR')[1:4]
  assert len(sections)==3
  fits=rows(ref/'fits.csv')
  facts=0
  for model,section in zip(('nonrobust','robust','cluster'),sections):
    printed={}
    for line in section.split('. estimates store')[0].splitlines():
      match=re.match(r'^\s*(LNHHEXP|AGE|SEX|HHSIZE|FARM|EDUC|_cons)\s*\|\s*([-+\d.]+)\s+([-+\d.]+)',line,re.I)
      if match:
        key=match.group(1).lower().replace('_cons','intercept')
        assert key not in printed
        printed[key]=[float(match.group(2)),float(match.group(3))]
    assert len(printed)==7
    for i,name in enumerate(('intercept','lnhhexp','age','sex','hhsize','farm','educ')):
      for j,field in enumerate(('value','standard_error')):
        selected=[r for r in fits if r['model']==model and r['path']==f'/coefficients/{i}/{field}']
        assert len(selected)==1
        assert abs(float(selected[0]['value'])-printed[name][j])<=5e-8, (model,name,field)
        facts+=1
  assert facts==42
  print(f'validated {facts} rounded author facts with DTA R fits; max DTA/ASCII cell difference {maxdiff:g}')


if __name__=='__main__':
  main()
