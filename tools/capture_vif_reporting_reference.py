"""Capture bounded VIF reference data outside production.

Run with NumPy/SciPy/statsmodels:
  PYTHONDONTWRITEBYTECODE=1 <python> tools/capture_vif_reporting_reference.py

Retained pin: Python 3.13.3, NumPy 2.4.4, SciPy 1.17.1, statsmodels 0.14.6.
The standalone statsmodels VIF helper fails on a zero-column auxiliary model.
For singleton/no-intercept cases, use NumPy's empty least-squares projection and
R² = 1 - SSR / uncentered TSS (the statsmodels no-constant R² definition).
"""
import json
from pathlib import Path
import platform

import numpy as np
import scipy
import statsmodels
import statsmodels.api as sm
from statsmodels.stats.outliers_influence import variance_inflation_factor

# y, predictor and weight/group missingness deliberately differ. Auxiliary
# regressions are unweighted on the retained parent-fit sample, not all rows.
rows = [
    [-2., -3., 2., 1., 100., 100., "a"],
    [-1., -4., 3., 2., 1., 1., "a"],
    [0., 2., 1., 3., 1., 1., "b"],
    [1., 0., 6., 4., 1., 1., "b"],
    [2., 5., 4., 6., 1., 1., "c"],
    [3., 2., 1., 4., None, None, None],
    [4., 4., 2., None, 1., 1., "c"],
    ["NaN", 3., 3., 8., 1., 1., "c"],
    [5., "Infinity", 2., 9., 1., 1., "c"],
]
columns = ["x1", "x2", "z", "y", "w", "sigma", "group"]
numeric = np.array([[np.nan if value is None else float(value) for value in row[:6]] for row in rows])
cases = []
for name, suffix, predictors in [
    ("ols", "", ["x2", "x1", "z"]),
    ("robust", ", robust", ["x2", "x1", "z"]),
    ("cluster", ", cluster(group)", ["x2", "x1", "z"]),
    ("wls", ", wls(w)", ["x2", "x1", "z"]),
    ("gls", ", gls(sigma)", ["x2", "x1", "z"]),
    ("no_intercept", ", noconstant", ["x2", "x1", "z"]),
    ("single", "", ["x1"]),
    ("single_no_intercept", ", noconstant", ["x1"]),
    ("single_no_intercept_robust", ", noconstant robust", ["x1"]),
    ("single_no_intercept_cluster", ", noconstant cluster(group)", ["x1"]),
    ("single_no_intercept_wls", ", noconstant wls(w)", ["x1"]),
    ("single_no_intercept_gls", ", noconstant gls(sigma)", ["x1"]),
]:
    indices = [columns.index(predictor) for predictor in predictors]
    keep = np.isfinite(numeric[:, [3, *indices]]).all(axis=1)
    if "wls(" in suffix or "gls(" in suffix:
        weight_index = columns.index("w" if "wls(" in suffix else "sigma")
        keep &= np.isfinite(numeric[:, weight_index]) & (numeric[:, weight_index] > 0)
    if "cluster(" in suffix:
        keep &= np.array([row[6] is not None for row in rows])
    design = numeric[keep][:, indices]
    intercept = "noconstant" not in suffix
    zero_column_auxiliary = None
    if len(predictors) == 1 and not intercept:
        response = design[:, 0]
        auxiliary_design = np.empty((len(response), 0))
        coefficients, _, rank, _ = np.linalg.lstsq(auxiliary_design, response, rcond=None)
        fitted = auxiliary_design @ coefficients
        residuals = response - fitted
        ssr, tss = float(residuals @ residuals), float(response @ response)
        r_squared = 1 - ssr / tss
        values = [float(1 / (1 - r_squared))]
        zero_column_auxiliary = {
            "column_count": 0, "rank": int(rank), "coefficients": coefficients.tolist(),
            "fitted_values": fitted.tolist(), "residual_sum_squares": ssr,
            "uncentered_tss": tss, "r_squared": r_squared,
        }
    else:
        auxiliary_design = sm.add_constant(design, has_constant="add") if intercept else design
        offset = int(intercept)
        values = [float(variance_inflation_factor(auxiliary_design, index + offset)) for index in range(len(predictors))]
    case = {
        "name": name, "command": "regress y " + " ".join(predictors) + suffix,
        "predictors": predictors, "include_intercept": intercept,
        "retained_indices": np.flatnonzero(keep).tolist(),
        "vifs": values, "mean_vif": float(np.mean(values)),
    }
    if zero_column_auxiliary is not None:
        case["zero_column_auxiliary"] = zero_column_auxiliary
    cases.append(case)
reference = {
    "provenance": {
        "python": platform.python_version(), "numpy": np.__version__, "scipy": scipy.__version__, "statsmodels": statsmodels.__version__,
        "method": "statsmodels variance_inflation_factor: unweighted auxiliary OLS on each parent's retained sample; matching intercept convention. Singleton/no-intercept uses NumPy zero-column least squares with uncentered R-squared; standalone statsmodels VIF rejects that empty design.",
        "tolerance": "max(1e-10 absolute, 1e-9 * abs(expected)); no rounding",
    },
    "columns": columns, "rows": rows, "cases": cases,
}
path = Path(__file__).resolve().parents[1] / "tests/fixtures/vif-reporting-reference.json"
path.write_text(json.dumps(reference, indent=2, allow_nan=False) + "\n")
print(path)
