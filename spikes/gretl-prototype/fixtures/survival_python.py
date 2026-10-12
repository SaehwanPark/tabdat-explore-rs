"""Pinned PH parametric streg; retain its original BFGS covariance."""
import csv,sys,tempfile
from pathlib import Path
import numpy as np,pandas as pd
from tabdat.executor import Executor
from tabdat.models import UseCommand
from tabdat.parser import parse_command
root=Path(__file__).parent
if sys.argv[1:]==['--generate-data']:
  rng=np.random.default_rng(18019);x1,x2=rng.normal(size=(2,320));rate=np.exp(-.4+.35*x1-.25*x2)
  te=rng.exponential(size=320)/rate;tw=(rng.exponential(size=320)/rate)**(1/1.6);ce,cw=rng.exponential(4,size=(2,320))
  with (root/'survival.csv').open('w',newline='') as f:
    w=csv.writer(f,lineterminator='\n');w.writerow(['time_exp','fail_exp','time_weibull','fail_weibull','x1','x2']);w.writerows(zip(np.minimum(te,ce),(te<=ce).astype(int),np.minimum(tw,cw),(tw<=cw).astype(int),x1,x2,strict=True))
with tempfile.TemporaryDirectory() as tmp:
  data=pd.read_csv(root/'survival.csv');p=Path(tmp)/'input.parquet';data.to_parquet(p,index=False)
  ex=Executor()
  try:
    ex.execute(UseCommand(path=p))
    for family,time,failure in [('exponential','time_exp','fail_exp'),('weibull','time_weibull','fail_weibull')]:
      r=ex.execute(parse_command(f'streg {time} x1 x2, failure({failure}) dist({family})'))
      assert r.observation_count==320 and [c.name for c in r.coefficients]==['intercept','x1','x2']
      values=[c.value for c in r.coefficients]+[c.standard_error for c in r.coefficients]
      print('python-original,'+family+','+','.join(format(float(v),'.17g') for v in values))
  finally:ex.close()
