"""Generate an independent finite-summary reference (development only).

  PYTHONDONTWRITEBYTECODE=1 <numpy/scipy python> tools/capture_summary_finite_reference.py
"""
import json
from pathlib import Path
import platform

import numpy as np
import scipy
from scipy import stats

columns = {
    "x": [1.5, np.nan, np.nan, np.inf, -np.inf, 3., 4.],
    "y": [2., 3., 4., 5., 6., 6., 8.],
    "bad": [np.nan, np.inf, -np.inf, np.nan, np.nan, np.nan, np.inf],
    "singleton": [np.nan, np.inf, -np.inf, np.nan, np.nan, 5., np.nan],
}
rows = []
for name, raw in columns.items():
    values = np.asarray(raw, dtype=float)
    retained = np.flatnonzero(np.isfinite(values))
    sample = values[retained]
    rows.append({"variable": name, "retained_indices": retained.tolist(), "count": len(sample),
                 "mean": float(np.mean(sample)) if len(sample) else None,
                 "std_dev": float(stats.tstd(sample, ddof=1)) if len(sample) > 1 else None,
                 "minimum": float(np.min(sample)) if len(sample) else None,
                 "maximum": float(np.max(sample)) if len(sample) else None})
reference = {"provenance": {"python": platform.python_version(), "numpy": np.__version__, "scipy": scipy.__version__,
                            "sample": "per-variable finite, non-NULL values; x index 1 is NULL, index 2 is NaN",
                            "method": "NumPy mean/min/max; scipy.stats.tstd(ddof=1)",
                            "tolerance": "max(1e-10 absolute, 1e-9 * abs(expected)); no display rounding"}, "rows": rows}
path = Path(__file__).resolve().parents[1] / "tests/fixtures/summary-finite-reference.json"
path.write_text(json.dumps(reference, indent=2, allow_nan=False) + "\n")
print(path)
