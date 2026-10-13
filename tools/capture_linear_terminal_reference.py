"""Generate the small independent terminal-workflow reference, outside production.

Run with an environment containing NumPy, SciPy and statsmodels:
  PYTHONDONTWRITEBYTECODE=1 <python> tools/capture_linear_terminal_reference.py
"""
import json
from pathlib import Path
import platform

import numpy as np
import scipy
import statsmodels
import statsmodels.api as sm

# Original candidate rows: six complete, then missing y and missing x.
x = np.array([0., 1., 2., 3., 4., 5., 6., np.nan])
y = np.array([1., 2., 1.5, 4., 3., 5., np.nan, 8.])
keep = np.isfinite(x) & np.isfinite(y)
model = sm.OLS(y[keep], sm.add_constant(x[keep])).fit()
combination = model.t_test([1., 1.])
restriction = model.f_test([0., 1.])
reference = {
    "provenance": {
        "python": platform.python_version(), "numpy": np.__version__,
        "scipy": scipy.__version__, "statsmodels": statsmodels.__version__,
        "method": "OLS with intercept, classical covariance, Student-t and F inference",
        "sample": "complete cases of y and x; original zero-based row indices",
        "tolerance": "max(1e-10 absolute, 1e-9 * abs(expected)); no display rounding",
    },
    "retained_indices": np.flatnonzero(keep).tolist(),
    "n": int(model.nobs), "df_residual": int(model.df_resid),
    "coefficient_names": ["intercept", "x"],
    "coefficients": model.params.tolist(), "standard_errors": model.bse.tolist(),
    "statistics": model.tvalues.tolist(), "p_values": model.pvalues.tolist(),
    "r_squared": model.rsquared, "adjusted_r_squared": model.rsquared_adj,
    "root_mse": float(np.sqrt(model.mse_resid)),
    "lincom": [float(combination.effect.item()), float(combination.sd.item()),
               float(combination.tvalue.item()), float(combination.pvalue.item()),
               *combination.conf_int()[0].tolist()],
    "test": [float(restriction.fvalue), float(restriction.pvalue)],
    # xb uses any finite predictor, including a row excluded due to missing y.
    "predictions": [float(model.params[0] + model.params[1] * value)
                    if np.isfinite(value) else None for value in x],
}
path = Path(__file__).resolve().parents[1] / "tests/fixtures/linear-terminal-reference.json"
path.write_text(json.dumps(reference, indent=2, allow_nan=False) + "\n")
print(path)
