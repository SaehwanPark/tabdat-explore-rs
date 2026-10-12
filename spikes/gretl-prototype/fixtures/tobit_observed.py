"""Separately labeled fallback likelihood with observed, not BFGS, covariance.

Does not alter pinned executor or its original output. Same zero-start BFGS
optimizer/settings, bounds and likelihood. Only Hessian extraction is replaced
by symmetric finite differences at that optimizer's solution.
"""
from pathlib import Path
import numpy as np,pandas as pd
from scipy.optimize import minimize
from scipy.stats import norm
from statsmodels.tools.numdiff import approx_hess
root=Path(__file__).parent;data=pd.read_csv(root/'tobit.csv');X=np.column_stack([np.ones(len(data)),data[['x1','x2']]])
for family in ['left','both']:
  y=np.asarray(data['y_'+family]);left=y<=0;right=y>=2 if family=='both' else np.zeros(len(y),dtype=bool);uncensored=~(left|right)
  def objective(params):
    sigma=np.exp(np.clip(params[-1],-10,10));mu=X@params[:-1];z=(y-mu)/sigma
    ll=np.zeros(len(y));ll[uncensored]=norm.logpdf(z[uncensored])-np.log(sigma)
    ll[left]=norm.logcdf(-mu[left]/sigma)
    ll[right]=np.log(np.maximum(1-norm.cdf((2-mu[right])/sigma),1e-12))
    return -float(np.sum(ll))
  fit=minimize(objective,np.zeros(4),method='BFGS')
  if not fit.success:fit=minimize(objective,np.zeros(4),method='L-BFGS-B')
  assert fit.success
  v=np.linalg.inv(approx_hess(fit.x,objective))
  values=list(fit.x[:-1])+list(np.sqrt(np.diag(v))[:3])+list(v.reshape(-1))+[fit.x[-1],-fit.fun]
  print('python-observed,'+family+','+','.join(format(float(x),'.17g') for x in values))
