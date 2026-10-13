"""Capture bounded VIF reference data outside production.

Run with NumPy/SciPy/statsmodels:
  PYTHONDONTWRITEBYTECODE=1 <python> tools/capture_vif_reporting_reference.py
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
]:
    indices = [columns.index(predictor) for predictor in predictors]
    keep = np.isfinite(numeric[:, [3, *indices]]).all(axis=1)
    if name in ("wls", "gls"):
        weight_index = columns.index("w" if name == "wls" else "sigma")
        keep &= np.isfinite(numeric[:, weight_index]) & (numeric[:, weight_index] > 0)
    if name == "cluster":
        keep &= np.array([row[6] is not None for row in rows])
    design = numeric[keep][:, indices]
    intercept = name != "no_intercept"
    auxiliary_design = sm.add_constant(design, has_constant="add") if intercept else design
    offset = int(intercept)
    values = [float(variance_inflation_factor(auxiliary_design, index + offset)) for index in range(len(predictors))]
    cases.append({
        "name": name, "command": "regress y " + " ".join(predictors) + suffix,
        "predictors": predictors, "include_intercept": intercept,
        "retained_indices": np.flatnonzero(keep).tolist(),
        "vifs": values, "mean_vif": float(np.mean(values)),
    })
reference = {
    "provenance": {
        "python": platform.python_version(), "numpy": np.__version__, "scipy": scipy.__version__, "statsmodels": statsmodels.__version__,
        "method": "statsmodels variance_inflation_factor: unweighted auxiliary OLS on each parent's retained sample; matching intercept convention",
        "tolerance": "max(1e-10 absolute, 1e-9 * abs(expected)); no rounding",
    },
    "columns": columns, "rows": rows, "cases": cases,
}
path = Path(__file__).resolve().parents[1] / "tests/fixtures/vif-reporting-reference.json"
path.write_text(json.dumps(reference, indent=2, allow_nan=False) + "\n")
print(path)
