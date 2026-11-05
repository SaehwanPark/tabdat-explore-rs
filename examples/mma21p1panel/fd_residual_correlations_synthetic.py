#!/usr/bin/env python3
"""Generate deterministic interleaved dyadic FD correlation control (development only)."""
import csv
import sys
from pathlib import Path

out=Path(sys.argv[1]);out.mkdir(exist_ok=False)
fields='source_row lnhr lnwg kids ageh agesq disab id year'.split()
rows=[]
for t in range(10):
  for g in range(1,17):
    x=2+g/16+t/8+((g*t+g*g+3*t*t)%17)/64
    y=7+x/2+((g*7+t*11+g*t*t)%23)/64
    age=24+t
    rows.append([len(rows)+1,y,x,g%3,age,age*age,g%2,g,1979+t])
with (out/'MOM.csv').open('w',newline='') as f:
  writer=csv.writer(f,lineterminator='\n');writer.writerow(fields);writer.writerows(rows)
with (out/'MOM.dat').open('w') as f:
  for row in rows:f.write(' '.join(str(v) for v in row[1:])+'\n')
