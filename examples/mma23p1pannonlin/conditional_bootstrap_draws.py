#!/usr/bin/env python3
"""Independent reference stream and unbiased integer rejection (development only)."""
import csv
import importlib.util
from pathlib import Path
import sys

if len(sys.argv)!=6:
  raise SystemExit('usage: conditional_bootstrap_draws.py seed bound replications draws-per-rep output.csv')
seed,bound,reps,draw_count=map(int,sys.argv[1:5])
assert 0<=seed<2**64 and 0<bound<=2**63-1 and reps>0 and draw_count>0
root=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('xoshiro_oracle',root/'crates/tabdat-stats/tests/fixtures/bootstrap/draws.py')
mod=importlib.util.module_from_spec(spec); spec.loader.exec_module(mod)
with open(sys.argv[5],'x',newline='') as f:
  writer=csv.writer(f,lineterminator='\n'); writer.writerow(['replicate','position','step','index0'])
  for rep in range(1,reps+1):
    rng=mod.stream((seed+rep-1)&mod.MASK); zone=mod.MASK-mod.MASK%bound
    step=0; accepted=0
    while accepted<draw_count:
      step+=1
      if step>draw_count+1024: raise RuntimeError('RNG rejection work bound exhausted')
      value=next(rng)
      if value<zone:
        accepted+=1; writer.writerow([rep,accepted,step,value%bound])
