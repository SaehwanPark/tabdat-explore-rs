"""External proper two-step covariance, not original ordinary-OLS inference."""
from pathlib import Path
import numpy as np,pandas as pd,statsmodels.api as sm
from scipy.stats import norm
r=Path(__file__).parent;d=pd.read_csv(r/'heckman.csv');X=np.c_[np.ones(len(d)),d[['x1','x2']]];Z=np.c_[np.ones(len(d)),d[['x1','z']]];s=np.asarray(d.selected)==1
first=sm.Probit(np.asarray(d.selected),Z).fit(disp=0);eta=Z@first.params;lam=norm.pdf(eta)/np.maximum(norm.cdf(eta),1e-12)
W=np.c_[X[s],lam[s]];second=sm.OLS(np.asarray(d.y)[s],W).fit();b=np.asarray(second.params);delta=lam[s]*(lam[s]+eta[s]);sigma2=np.mean(second.resid**2)+b[-1]**2*np.mean(delta);rho=b[-1]/np.sqrt(sigma2);assert abs(rho)<1
C=np.linalg.inv(W.T@W);B=W.T@(delta[:,None]*Z[s]);Vp=np.asarray(first.cov_params());S=sigma2*C@(W.T@W-rho**2*(W.T@(delta[:,None]*W)-B@Vp@B.T))@C
values=list(b)+list(first.params)+list(np.sqrt(np.diag(S)))+list(first.bse)+list(S.reshape(-1))+list(Vp.reshape(-1))+[np.sqrt(sigma2),rho]
print('python-corrected,'+','.join(format(float(v),'.17g') for v in values))
