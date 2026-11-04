#!/usr/bin/env python3
"""Generate small balanced controls; no author data (development only)."""
import csv
import math
from pathlib import Path
import sys

if len(sys.argv)!=3 or sys.argv[1] not in ('interior','boundary','replicate_rank_failure'):
  raise SystemExit('usage: random_effects_ml_bootstrap_synthetic.py interior|boundary|replicate_rank_failure fresh_directory')
kind, out = sys.argv[1], Path(sys.argv[2])
out.mkdir()
fields = ['source_row','lnhr','lnwg','kids','ageh','agesq','disab','id','year']
records = []
G = 3 if kind=='replicate_rank_failure' else 16
for person in range(1,G+1):
  for t in range(10):
    within = .12*math.sin(t+1)+.03*math.cos(person*(t+1))
    x = 2+.04*person+within
    j = t % 5
    noise = .16*math.sin(1.7*(j+1)+.4*person)*(1 if t<5 else -1)
    unit = .22*math.sin(person*1.2) if kind=='interior' else 0
    if kind=='replicate_rank_failure':
      x = 2+.3*person+.08*(t-4.5)
      unit = .2*math.sin(person)
    y = 7+.2*x+unit+noise
    records.append([len(records)+1,round(y,6),round(x,6),person%4,25+person,(25+person)**2,0,person,1979+t])
# Deliberately interleave people and reverse times; source_row follows physical input.
records.sort(key=lambda r: (-r[-1], -r[-2]))
for i,r in enumerate(records,1):
  r[0] = i
with (out/'MOM.csv').open('x',newline='') as f:
  w = csv.writer(f,lineterminator='\n'); w.writerow(fields); w.writerows(records)
with (out/'MOM.dat').open('x') as f:
  for r in records:
    f.write(' '.join(str(x) for x in r[1:])+'\n')
