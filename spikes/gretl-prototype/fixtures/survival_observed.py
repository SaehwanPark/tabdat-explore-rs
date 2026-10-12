"""Same pinned PH objective/start/optimizer; separately observed Hessian."""
from pathlib import Path
import numpy as np,pandas as pd
from scipy.optimize import minimize
from statsmodels.tools.numdiff import approx_hess
root=Path(__file__).parent;d=pd.read_csv(root/'survival.csv');X=np.c_[np.ones(len(d)),d[['x1','x2']]]
for family in ['exponential','weibull']:
  t=np.asarray(d['time_exp' if family=='exponential' else 'time_weibull']);fail=np.asarray(d['fail_exp' if family=='exponential' else 'fail_weibull']);k=3+(family=='weibull')
  def objective(params):
    rate=np.exp(np.clip(X@params[:3],-30,30))
    if family=='exponential':ll=fail*np.log(np.maximum(rate,1e-12))-rate*t
    else:
      shape=np.exp(np.clip(params[-1],-10,10));ll=fail*(np.log(shape)+np.log(np.maximum(rate,1e-12))+(shape-1)*np.log(np.maximum(t,1e-12)))-rate*t**shape
    return -float(np.sum(ll))
  fit=minimize(objective,np.zeros(k),method='BFGS')
  if not fit.success:fit=minimize(objective,np.zeros(k),method='L-BFGS-B')
  assert fit.success
  v=np.linalg.inv(approx_hess(fit.x,objective));values=list(fit.x)+list(np.sqrt(np.diag(v)))+list(v.reshape(-1))+[-fit.fun]
  print('python-observed,'+family+','+','.join(format(float(v),'.17g') for v in values))
