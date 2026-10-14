"""Dev-only selected-cohort linear references; no production Python dependency."""
import json
import platform
from pathlib import Path
import numpy as np
import scipy
import statsmodels
import statsmodels.api as sm
from statsmodels.stats.outliers_influence import variance_inflation_factor

# id, eligibility, y, x, z, analytic weight, diagonal variance, cluster
records = [
  [0,0,99,1,1,-1,-1,"EX"],
  [1,1,2.1,0,1,1,1.4,"A"],
  [2,1,4.8,1,0,2,2,"A"],
  [3,None,88,2,2,-3,-3,"EX"],
  [4,1,2.9,2,2,0.5,1.1,"B"],
  [5,1,5.7,3,1,3,3,"B"],
  [6,0,77,4,0,0,0,"EX"],
  [7,1,9.2,4,3,1.5,0.8,"C"],
  [8,1,8.4,5,1,2.5,2.3,"C"],
  [9,1,None,6,2,1,1,"D"],
  [10,1,15,None,2,1,1,"D"],
  [11,1,16,7,None,1,1,"D"],
  [12,1,12.8,6,3,None,None,"D"],
  [13,1,11.1,7,0,1,1,None],
  [14,0,22,8,1,-1,-1,"EX"],
  [15,1,None,8,0,0,0,None],
]
numeric = np.array([[np.nan if value is None else value for value in row[:7]] for row in records], dtype=float)
eligibility = numeric[:,1] == 1
y, x, z, w, sigma = [numeric[:,index] for index in range(2,7)]
clusters = np.array([row[7] for row in records], dtype=object)
finite_design = np.isfinite(x) & np.isfinite(z)
cases = []
for estimator in ["ols", "wls", "gls"]:
  for covariance in ["nonrobust", "robust", "cluster"]:
    for intercept in [True, False]:
      selected = eligibility & np.isfinite(y) & finite_design
      if estimator != "ols":
        weight = w if estimator == "wls" else sigma
        selected &= np.isfinite(weight)
        assert np.all(weight[selected] > 0)
      if covariance == "cluster":
        selected &= np.array([group is not None for group in clusters])
      design = np.column_stack([x[selected],z[selected]])
      if intercept:
        design = sm.add_constant(design)
      if estimator == "ols":
        model = sm.OLS(y[selected],design)
      elif estimator == "wls":
        model = sm.WLS(y[selected],design,weights=w[selected])
      else:
        model = sm.GLS(y[selected],design,sigma=np.diag(sigma[selected]))
      options = {"use_t": True}
      if covariance == "robust":
        options.update(cov_type="HC1")
      elif covariance == "cluster":
        # Cluster covariance and all Student-t/F inference use the retained groups.
        options.update(cov_type="cluster",cov_kwds={"groups":clusters[selected],"use_correction":True,"df_correction":True})
      fit = model.fit(**options)
      combination = fit.t_test(([0] if intercept else []) + [1,1])
      restrictions = np.eye(design.shape[1])[-2:]
      joint = fit.f_test(restrictions)
      prediction_design = np.column_stack([x,z])
      if intercept:
        prediction_design = sm.add_constant(prediction_design)
      predictions = prediction_design @ fit.params
      flags = (["wls(w)"] if estimator == "wls" else ["gls(sigma)"] if estimator == "gls" else [])
      flags += (["robust"] if covariance == "robust" else ["cluster(g)"] if covariance == "cluster" else [])
      if not intercept:
        flags += ["noconstant"]
      command = "regress y x z if eligible == 1" + (", " + " ".join(flags) if flags else "")
      cases.append({
        "command":command,"estimator":estimator,"covariance":covariance,"include_intercept":intercept,
        "retained_indices":np.flatnonzero(selected).tolist(),"n":int(fit.nobs),"df_residual":int(fit.df_resid),"df_inference":len(set(clusters[selected]))-1 if covariance == "cluster" else int(fit.df_resid),
        "coefficient_names":(["intercept"] if intercept else [])+["x","z"],
        "coefficients":fit.params.tolist(),"standard_errors":fit.bse.tolist(),"statistics":fit.tvalues.tolist(),"p_values":fit.pvalues.tolist(),
        "covariance_matrix":fit.cov_params().tolist(),
        "r_squared":float(fit.rsquared),"adjusted_r_squared":float(fit.rsquared_adj),"root_mse":float(np.sqrt(fit.mse_resid)),
        "lincom":[float(combination.effect.item()),float(combination.sd.item()),float(combination.tvalue.item()),float(combination.pvalue.item()),*combination.conf_int()[0].tolist()],
        "test":[float(joint.fvalue),float(joint.pvalue)],
        "vif":[float(variance_inflation_factor(design,index)) for index in range(int(intercept),design.shape[1])],
        "predictions":[float(value) if valid else None for value,valid in zip(predictions,finite_design)],
        "weights":(w[selected] if estimator=="wls" else 1/sigma[selected]).tolist() if estimator != "ols" else None,
        "clusters":clusters[selected].tolist() if covariance=="cluster" else None,
      })
result = {"provenance":{"python":platform.python_version(),"numpy":np.__version__,"scipy":scipy.__version__,"statsmodels":statsmodels.__version__,"sample":"TRUE eligibility intersect finite y/x/z and applicable weight/cluster exclusions; original zero-based active indices","cluster_inference":"finite-sample covariance correction G/(G-1)*(N-1)/(N-K); coefficients/lincom/tests use Student-t/F G-1 inference df; fit residual df remains N-K","tolerance":"max(1e-10 absolute, 1e-9 * abs(expected)); exact indices/counts/df"},"records":records,"eligible_indices":np.flatnonzero(eligibility).tolist(),"cases":cases}
path=Path(__file__).resolve().parents[1]/"tests/fixtures/conditional-linear-reference.json"
path.write_text(json.dumps(result,indent=2,allow_nan=False)+"\n")
print(path)
