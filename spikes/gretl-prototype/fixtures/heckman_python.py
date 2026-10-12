"""Actual pinned two-step executor, original ordinary-OLS SEs preserved."""
import csv,sys,tempfile
from pathlib import Path
import numpy as np,pandas as pd
from tabdat.executor import Executor
from tabdat.models import UseCommand
from tabdat.parser import parse_command
root=Path(__file__).parent
if sys.argv[1:]==['--generate-data']:
  rng=np.random.default_rng(44107);x1,x2,z,u,v=rng.normal(size=(5,480));selected=(-.15+.5*x1+.7*z+v>0).astype(int)
  latent=1+.6*x1-.35*x2+.8*(.4*v+np.sqrt(1-.4**2)*u);y=np.where(selected,latent,0.)
  with (root/'heckman.csv').open('w',newline='') as f:
    w=csv.writer(f,lineterminator='\n');w.writerow(['y','x1','x2','selected','z']);w.writerows(zip(y,x1,x2,selected,z,strict=True))
with tempfile.TemporaryDirectory() as tmp:
  d=pd.read_csv(root/'heckman.csv');p=Path(tmp)/'input.parquet';d.to_parquet(p,index=False);ex=Executor()
  try:
    ex.execute(UseCommand(path=p));r=ex.execute(parse_command('heckman y x1 x2, selectdep(selected) select(x1 z)'))
    assert r.observation_count==480
    assert [c.name for c in r.outcome_coefficients]==['intercept','x1','x2','mills_lambda']
    assert [c.name for c in r.selection_coefficients]==['intercept','x1','z']
    coefs=list(r.outcome_coefficients)+list(r.selection_coefficients)
    values=[c.value for c in coefs]+[c.standard_error for c in coefs]
    print('python-original,'+','.join(format(float(v),'.17g') for v in values))
  finally:ex.close()
