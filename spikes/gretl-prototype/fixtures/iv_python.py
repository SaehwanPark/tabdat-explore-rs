"""Pinned executor IV2SLS, classical unadjusted covariance."""
import csv, sys, tempfile
from pathlib import Path
import numpy as np, pandas as pd
from tabdat.executor import Executor
from tabdat.models import UseCommand
from tabdat.parser import parse_command
root=Path(__file__).parent
if sys.argv[1:]==["--generate-data"]:
  rng=np.random.default_rng(6141); x,z1,z2,u,v=rng.normal(size=(5,160))
  d=.8*z1+.4*z2+.2*x+.6*u+.4*v; y=1.2+.3*x+1.7*d+u
  with (root/"iv.csv").open("w",newline="") as f:
    w=csv.writer(f,lineterminator="\n");w.writerow(["y","x","d","z1","z2"]);w.writerows(zip(y,x,d,z1,z2,strict=True))
with tempfile.TemporaryDirectory() as tmp:
  data=pd.read_csv(root/"iv.csv");p=Path(tmp)/"input.parquet";data.to_parquet(p,index=False)
  ex=Executor()
  try:
    ex.execute(UseCommand(path=p));r=ex.execute(parse_command("ivregress 2sls y x, endog(d) iv(z1 z2)"));f=ex.state.iv_regression.fitted_model
    assert f.nobs==160 and not f.debiased
    assert [c.name for c in r.coefficients]==["intercept","x","d"]
    values=list(f.params)+list(f.std_errors)+list(np.asarray(f.cov).reshape(-1))+[f.rsquared]
    print("python,iv,"+",".join(format(float(v),".17g") for v in values))
  finally:ex.close()
