#!/usr/bin/env python3
"""Generated FE controls: varying composition, exact zero slope variance, late rank failure."""
import csv
import math
from pathlib import Path
import sys

if len(sys.argv)!=3 or sys.argv[1] not in ('heterogeneous','zero_variance','rank_failure'):
  raise SystemExit('usage: fe_level_bootstrap_synthetic.py heterogeneous|zero_variance|rank_failure fresh_directory')
kind,out=sys.argv[1],Path(sys.argv[2]);out.mkdir();records=[]
g=3 if kind=='rank_failure' else 16
for person in range(1,g+1):
  for t in range(10):
    x=2+.04*person+.12*math.sin(t+1)+.03*math.cos(person*(t+1))
    y=7+.2*x+.20*math.sin(person*1.2)+.16*math.sin(1.7*(t%5+1)+.4*person)*(1 if t<5 else -1)
    x,y=round(x,6),round(y,6)
    if kind=='zero_variance':
      x=2+person*.0625+.0625*(t-4.5)
      y=7+.25*x+.125*((person%5)-2)+((2*t-9)**2-33)/256
    if kind=='rank_failure' and person>1:x=2+.25*person
    records.append([0,y,x,person%4,25+person,(25+person)**2,0,person,1979+t])
records.sort(key=lambda r:(-r[-1],-r[-2]))
for i,r in enumerate(records,1):r[0]=i
with (out/'MOM.csv').open('x',newline='') as f:
  w=csv.writer(f,lineterminator='\n');w.writerow(['source_row','lnhr','lnwg','kids','ageh','agesq','disab','id','year']);w.writerows(records)
with (out/'MOM.dat').open('x') as f:
  for r in records:f.write(' '.join(str(v) for v in r[1:])+'\n')
