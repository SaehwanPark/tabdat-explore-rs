"""Generate independent conditional finite summaries (development only).

  PYTHONDONTWRITEBYTECODE=1 <numpy/scipy python> tools/capture_conditional_summary_reference.py
"""
import json
from pathlib import Path
import platform

import numpy as np
import scipy
from scipy import stats

# Matches conditional_summary_contract.rs; g='A' selects rows 1,2,3,6.
columns = {
    "x": [1.5, np.nan, np.nan, np.inf, -np.inf, 3., 4.],
    "y": [2., 3., 4., 5., 6., 6., 8.],
    "bad": [np.nan, np.inf, -np.inf, np.nan, np.nan, np.nan, np.inf],
    "singleton": [np.nan, np.inf, -np.inf, np.nan, np.nan, 5., np.nan],
}
selected = np.array([0, 1, 2, 5])
rows = []
for name, raw in columns.items():
    values = np.asarray(raw, dtype=float)
    retained = selected[np.isfinite(values[selected])]
    sample = values[retained]
    rows.append({"variable": name, "retained_indices": retained.tolist(), "count": len(sample),
                 "mean": float(np.mean(sample)) if len(sample) else None,
                 "std_dev": float(stats.tstd(sample, ddof=1)) if len(sample) > 1 else None,
                 "minimum": float(np.min(sample)) if len(sample) else None,
                 "maximum": float(np.max(sample)) if len(sample) else None})
reference = {"provenance": {"python": platform.python_version(), "numpy": np.__version__, "scipy": scipy.__version__,
                           "selection": "g='A': indices [0,1,2,5]; FALSE/UNKNOWN excluded, then per-variable finite non-NULL sample",
                           "method": "NumPy mean/min/max; scipy.stats.tstd(ddof=1)",
                           "tolerance": "max(1e-10 absolute, 1e-9 * abs(expected)); no display rounding"},
             "selected_indices": selected.tolist(), "rows": rows}
path = Path(__file__).resolve().parents[1] / "tests/fixtures/conditional-summary-reference.json"
path.write_text(json.dumps(reference, indent=2, allow_nan=False) + "\n")
print(path)
