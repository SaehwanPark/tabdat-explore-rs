#!/usr/bin/env python3
"""Development-only independent group-draw oracle; no production dependency."""
import csv, importlib.util
from pathlib import Path
root=Path(__file__).parent
spec=importlib.util.spec_from_file_location('oracle',root.parent/'bootstrap/draws.py')
oracle=importlib.util.module_from_spec(spec);spec.loader.exec_module(oracle)
groups=['C','A','B','D']; remaining=dict(zip(groups,[3,4,6,7])); unbalanced=[]
while any(remaining.values()):
  for key in groups:
    if remaining[key]: unbalanced.append(key);remaining[key]-=1
with (root/'inputs.csv').open('w',newline='') as f:
  w=csv.writer(f,lineterminator='\n');w.writerow(['y','x','z','balanced','unbalanced'])
  for i in range(20):
    x=(i-9)/3;z=((i*i+3*i)%13)/7
    w.writerow([1+.7*x-.3*z+.11*((i*i+3)%7),x,z,groups[i%4],unbalanced[i]])
with (root/'draws.csv').open('w',newline='') as f:
  w=csv.writer(f,lineterminator='\n');w.writerow(['case','seed','replication','groups'])
  for case,seed in [('balanced',oracle.MASK),('unbalanced',7),('noconstant',7),('shifted',7)]:
    for r,index in enumerate(oracle.draws(seed,4,40)):w.writerow([case,seed,r+1,':'.join(map(str,index))])
