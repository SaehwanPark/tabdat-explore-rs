#!/usr/bin/env python3
"""Balanced Gaussian GEE controls (development only; no author data)."""
import csv
import math
from pathlib import Path
import sys

if len(sys.argv)!=3 or sys.argv[1] not in ('positive','negative','singular','rank'):
  raise SystemExit('usage: gaussian_pa_synthetic.py positive|negative|singular|rank fresh_directory')
kind,out = sys.argv[1],Path(sys.argv[2])
out.mkdir()
records=[]
for person in range(1,33):
  for t in range(10):
    x=2+.04*person+.12*math.sin(t+1)+.03*math.cos(person*(t+1))
    noise=.16*math.sin(1.7*(t%5+1)+.4*person)*(1 if t<5 else -1)
    unit=(.20 if kind=='positive' else .04)*math.sin(person*1.2)
    y=7+.2*x+unit+noise
    if kind=='singular':
      x=2+.125*person+.0625*(t-4.5)
      y=7+.25*x+.125*(t-4.5)
    if kind=='rank':
      x=2+.04*person
      y=7+.2*x+unit+noise
    records.append([0,round(y,6),round(x,6),person%4,25+person,(25+person)**2,0,person,1979+t])
# Interleave people and reverse time; source rows retain physical input provenance.
records.sort(key=lambda r:(-r[-1],-r[-2]))
for i,r in enumerate(records,1):r[0]=i
with (out/'MOM.csv').open('x',newline='') as f:
  w=csv.writer(f,lineterminator='\n');w.writerow(['source_row','lnhr','lnwg','kids','ageh','agesq','disab','id','year']);w.writerows(records)
with (out/'MOM.dat').open('x') as f:
  for r in records:f.write(' '.join(str(x) for x in r[1:])+'\n')
