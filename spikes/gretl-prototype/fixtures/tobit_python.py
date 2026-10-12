"""Actual pinned executor; explicitly record whether its R route falls back."""
import csv,sys,tempfile
from pathlib import Path
import numpy as np,pandas as pd
import tabdat.executor as executor
from tabdat.models import UseCommand
from tabdat.parser import parse_command
root=Path(__file__).parent
if sys.argv[1:]==['--generate-data']:
  rng=np.random.default_rng(9017);x1,x2,e=rng.normal(size=(3,240))
  latent=.8+.6*x1-.4*x2+.9*e
  with (root/'tobit.csv').open('w',newline='') as f:
    w=csv.writer(f,lineterminator='\n');w.writerow(['y_left','y_both','x1','x2']);w.writerows(zip(np.maximum(0,latent),np.clip(latent,0,2),x1,x2,strict=True))
fallback=[];original=executor._fit_tobit_parametric
def observed_fallback(**kwargs):
  fallback.append(True);return original(**kwargs)
executor._fit_tobit_parametric=observed_fallback
with tempfile.TemporaryDirectory() as tmp:
  data=pd.read_csv(root/'tobit.csv');p=Path(tmp)/'input.parquet';data.to_parquet(p,index=False)
  ex=executor.Executor()
  try:
    ex.execute(UseCommand(path=p))
    for family,outcome,options in [('left','y_left','ll(0)'),('both','y_both','ll(0) ul(2)')]:
      before=len(fallback);r=ex.execute(parse_command(f'tobit {outcome} x1 x2, {options}'))
      assert r.observation_count==240 and [c.name for c in r.coefficients]==['intercept','x1','x2']
      values=[c.value for c in r.coefficients]+[c.standard_error for c in r.coefficients]
      print('python,'+family+','+','.join(format(float(v),'.17g') for v in values))
      print(f'{family} oracle route: '+('Python fallback' if len(fallback)>before else 'R survival adapter'),file=sys.stderr)
  finally:ex.close()
