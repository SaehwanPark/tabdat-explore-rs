"""Pinned executor IRLS, preserved without modifying its stopping policy."""
import csv,sys,tempfile
from pathlib import Path
import numpy as np,pandas as pd
from tabdat.executor import Executor
from tabdat.models import UseCommand
from tabdat.parser import parse_command
root=Path(__file__).parent
if sys.argv[1:]==['--generate-data']:
  rng=np.random.default_rng(21307);x1,x2=rng.normal(size=(2,360));e=.7*rng.standard_t(4,size=360);y=1.2+.8*x1-.4*x2+e
  with (root/'quantile.csv').open('w',newline='') as f:
    w=csv.writer(f,lineterminator='\n');w.writerow(['y','x1','x2']);w.writerows(zip(y,x1,x2,strict=True))
with tempfile.TemporaryDirectory() as tmp:
  data=pd.read_csv(root/'quantile.csv');p=Path(tmp)/'input.parquet';data.to_parquet(p,index=False)
  ex=Executor()
  try:
    ex.execute(UseCommand(path=p))
    for q in [.25,.5,.75]:
      r=ex.execute(parse_command(f'qreg y x1 x2, quantile({q})'));fit=ex.state.qreg_regression.fitted_model
      assert fit.nobs==360 and [c.name for c in r.coefficients]==['intercept','x1','x2']
      values=list(fit.params)+list(fit.bse)+list(np.asarray(fit.cov_params()).reshape(-1))+[fit.bandwidth,1/fit.sparsity,fit.prsquared]
      print('python-original,'+str(q)+','+','.join(format(float(v),'.17g') for v in values))
      print(f'q={q} IRLS iterations={fit.iterations}',file=sys.stderr)
  finally:ex.close()
