"""Pinned executor balanced FE and explicitly constant-augmented RE."""
import csv,sys,tempfile
from pathlib import Path
import numpy as np,pandas as pd
from tabdat.executor import Executor
from tabdat.models import UseCommand
from tabdat.parser import parse_command
root=Path(__file__).parent
if sys.argv[1:]==['--generate-data']:
  rng=np.random.default_rng(8147);g,t=24,8
  u=rng.normal(size=g);x1,x2,e=rng.normal(size=(3,g*t))
  y=1.4+.6*x1-.9*x2+np.repeat(u,t)+.7*e
  with (root/'panel.csv').open('w',newline='') as f:
    w=csv.writer(f,lineterminator='\n');w.writerow(['id','time','y','x1','x2'])
    w.writerows(zip(np.repeat(np.arange(g),t),np.tile(np.arange(t),g),y,x1,x2,strict=True))
with tempfile.TemporaryDirectory() as tmp:
  data=pd.read_csv(root/'panel.csv');data['const']=1.;p=Path(tmp)/'input.parquet';data.to_parquet(p,index=False)
  ex=Executor()
  try:
    ex.execute(UseCommand(path=p));ex.execute(parse_command('panel id time'))
    for family,predictors in [('fe','x1 x2'),('re','const x1 x2')]:
      r=ex.execute(parse_command(f'xtreg y {predictors}, {family}'))
      fit=getattr(ex.state.xt_regressions,family).fitted_model
      assert fit.nobs==192
      values=list(fit.params)+list(fit.std_errors)+list(np.asarray(fit.cov).reshape(-1))
      print('python,'+family+','+','.join(format(float(v),'.17g') for v in values))
  finally:ex.close()
