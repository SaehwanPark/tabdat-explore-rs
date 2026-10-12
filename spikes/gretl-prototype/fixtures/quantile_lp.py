"""Independent HiGHS exact check-loss LP plus pinned iid covariance convention.

Not the unmodified executor: solver differs, kernel/bandwidth convention does not.
"""
from pathlib import Path
import numpy as np,pandas as pd
from scipy.optimize import linprog
from scipy.stats import norm
from statsmodels.regression.quantile_regression import hall_sheather,kernels
root=Path(__file__).parent;data=pd.read_csv(root/'quantile.csv');y=np.asarray(data.y);X=np.column_stack([np.ones(len(y)),data[['x1','x2']]]);n,k=X.shape
for q in [.25,.5,.75]:
  cost=np.r_[np.zeros(k),np.full(n,q),np.full(n,1-q)]
  fit=linprog(cost,A_eq=np.c_[X,np.eye(n),-np.eye(n)],b_eq=y,bounds=[(None,None)]*k+[(0,None)]*(2*n),method='highs')
  assert fit.success and np.max(np.abs(X@fit.x[:k]+fit.x[k:k+n]-fit.x[k+n:]-y))<1e-8
  beta=fit.x[:k];e=y-X@beta;h0=hall_sheather(n,q);h=min(np.std(y),(np.percentile(e,75)-np.percentile(e,25))/1.34)*(norm.ppf(q+h0)-norm.ppf(q-h0))
  density=np.sum(kernels['epa'](e/h))/(n*h);v=q*(1-q)/(density*density)*np.linalg.inv(X.T@X)
  rho=lambda u:np.where(u>=0,q*u,(q-1)*u)
  prsquared=1-np.sum(rho(e))/np.sum(rho(y-np.quantile(y,q)))
  values=list(beta)+list(np.sqrt(np.diag(v)))+list(v.reshape(-1))+[h,density,prsquared]
  print('python-lp,'+str(q)+','+','.join(format(float(v),'.17g') for v in values))
