#!/usr/bin/env python3
"""Independent development-only xoshiro256++/SplitMix64 draw oracle.
Reference algorithms: https://prng.di.unimi.it/xoshiro256plusplus.c and splitmix64.c
Run from repository root; no production dependency.
"""
import csv
import sys
from pathlib import Path

MASK=(1<<64)-1

def rotate(x,k):
  return ((x<<k)|(x>>(64-k)))&MASK

def stream(seed):
  state=[]
  for _ in range(4):
    seed=(seed+0x9e3779b97f4a7c15)&MASK
    z=seed
    z=((z^(z>>30))*0xbf58476d1ce4e5b9)&MASK
    z=((z^(z>>27))*0x94d049bb133111eb)&MASK
    state.append(z^(z>>31))
  while True:
    value=(rotate((state[0]+state[3])&MASK,23)+state[0])&MASK
    t=(state[1]<<17)&MASK
    state[2]^=state[0]; state[3]^=state[1]; state[1]^=state[2]; state[0]^=state[3]
    state[2]^=t; state[3]=rotate(state[3],45)
    yield value

def draws(seed,n,b):
  rng=stream(seed); zone=MASK-MASK%n
  for _ in range(b):
    row=[]
    while len(row)<n:
      value=next(rng)
      if value<zone: row.append(value%n)
    yield row

if __name__=='__main__':
  if len(sys.argv)==6 and sys.argv[1]=='--draws':
    seed,n,b=map(int,sys.argv[2:5])
    with open(sys.argv[5],'x',newline='') as f:
      w=csv.writer(f,lineterminator='\n'); w.writerow(['replication','indices'])
      for r,index in enumerate(draws(seed,n,b)):
        w.writerow([r+1,':'.join(map(str,index))])
    sys.exit(0)
  root=Path(__file__).parent
  with (root/'inputs.csv').open('w',newline='') as f:
    w=csv.writer(f,lineterminator='\n'); w.writerow(['y','x','z'])
    xs=[-2,-1,0,1,2,3,.5,1.5,-.5,2.5,-1.5,3.5]
    zs=[0,1,0,1,0,1,0,1,1,0,.2,.7]
    for i,(x,z) in enumerate(zip(xs,zs)):
      w.writerow([1+.7*x-.3*z+.11*((i*i+3)%7),x,z])
  with (root/'draws.csv').open('w',newline='') as f:
    w=csv.writer(f,lineterminator='\n'); w.writerow(['case','seed','n','replication','indices'])
    for case,seed in [('multiple',MASK),('noconstant',7),('shifted',7)]:
      for r,indices in enumerate(draws(seed,12,40)):
        w.writerow([case,seed,12,r+1,':'.join(map(str,indices))])
