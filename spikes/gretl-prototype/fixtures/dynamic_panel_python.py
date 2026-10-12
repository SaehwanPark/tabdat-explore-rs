"""Original pooled executor and independent entity-moment reference; no runtime dependency."""
import csv,sys,tempfile
from pathlib import Path
import numpy as np,pandas as pd
from tabdat.executor import Executor
from tabdat.models import UseCommand
from tabdat.parser import parse_command
root=Path(__file__).parent
if sys.argv[1:]==['--generate-data']:
  rng=np.random.default_rng(29183);g,t,burn=64,8,32;a=rng.normal(size=g);x=rng.normal(size=(g,t+burn,2));eps=rng.normal(scale=.65,size=(g,t+burn));y=np.zeros((g,t+burn))
  for j in range(1,t+burn):y[:,j]=.55*y[:,j-1]+.8*x[:,j,0]-.4*x[:,j,1]+a+eps[:,j]
  with (root/'dynamic-panel.csv').open('w',newline='') as f:
    w=csv.writer(f,lineterminator='\n');w.writerow(['id','time','y','x1','x2'])
    for i in range(g):
      for j in range(t):w.writerow([i,j,y[i,j+burn],*x[i,j+burn]])
d=pd.read_csv(root/'dynamic-panel.csv');Y=[];X=[];Z=[];ids=[]
for entity,frame in d.groupby('id',sort=True):
  r=frame[['y','x1','x2']].to_numpy()
  for j in range(2,len(r)):
    Y.append(r[j,0]-r[j-1,0]);X.append([r[j-1,0]-r[j-2,0],*(r[j,1:]-r[j-1,1:])]);Z.append([r[j-2,0],*(r[j,1:]-r[j-1,1:])]);ids.append(entity)
Y=np.asarray(Y);X=np.asarray(X);Z=np.asarray(Z);ids=np.asarray(ids);G=Z.T@X;b=np.linalg.solve(G,Z.T@Y);u=Y-X@b;meat=np.zeros((3,3))
for entity in np.unique(ids):
  m=Z[ids==entity].T@u[ids==entity];meat+=np.outer(m,m)
inv=np.linalg.inv(G);cov=inv@meat@inv.T
print('entity-python,'+','.join(format(float(v),'.17g') for v in [*b,*np.sqrt(np.diag(cov)),*cov.reshape(-1)]))
with tempfile.TemporaryDirectory() as tmp:
  p=Path(tmp)/'input.parquet';d.to_parquet(p,index=False);ex=Executor()
  try:
    ex.execute(UseCommand(path=p));ex.execute(parse_command('panel id time'));r=ex.execute(parse_command('xtabond y x1 x2, lags(1) instlag(2) robust'));fit=ex.state.xtabond_regression.fitted_model;assert r.observation_count==384
    order=[2,0,1];b=np.asarray(fit.params)[order];cov=np.asarray(fit.cov)[np.ix_(order,order)]
    print('original-python,'+','.join(format(float(v),'.17g') for v in [*b,*np.sqrt(np.diag(cov)),*cov.reshape(-1)]))
  finally:ex.close()
