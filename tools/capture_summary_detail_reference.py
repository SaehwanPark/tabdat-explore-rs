"""Independent finite-sample type-7 percentiles; development-only NumPy.

Regenerate:
  uv run --python 3.13.3 --with numpy==2.4.4 python tools/capture_summary_detail_reference.py
No Rust/DuckDB outputs are used to construct expected values.
"""
import csv
import io
import json
from pathlib import Path
import platform
import numpy as np

PERCENTILES = [1, 5, 10, 25, 50, 75, 90, 95, 99]
MIXED = [
    [-7, 1, True], [-1, None, True], [0, 7, True], [4, 9, True],
    [12, 19, False], [None, 2, True], ["NaN", 3, True],
    ["Infinity", 4, True], ["-Infinity", "NaN", True],
    [3, "Infinity", False], [-2, "-Infinity", False],
]
CASES = [
    ("mixed", MIXED, False),
    ("selected", MIXED, True),
    ("even_repeated_constant", [[-2, 3, True], [-2, 3, True], [4, 3, True], [4, 3, True]], False),
    ("singleton", [[2, None, True], [None, "NaN", True]], False),
    ("no_finite_x", [[None, 1, True], ["NaN", 2, True], ["Infinity", 3, True], ["-Infinity", 4, True]], False),
    ("empty_selected", MIXED, None),
    ("empty_dataset", [], False),
]


def capture():
    cases = []
    for name, rows, selected_only in CASES:
        buffer = io.StringIO(newline="")
        writer = csv.writer(buffer, lineterminator="\n")
        writer.writerow(["x", "y", "selected"])
        writer.writerows(rows)
        selected_indices = [i for i, row in enumerate(rows)
                            if selected_only is False or (selected_only is True and row[2] is True)]
        condition = "selected" if selected_only is True else "(x + 0) > 1000" if selected_only is None else None
        summaries = []
        for column, variable in enumerate(["x", "y"]):
            finite_indices = [i for i in selected_indices
                              if rows[i][column] is not None and np.isfinite(float(rows[i][column]))]
            values = np.asarray([float(rows[i][column]) for i in finite_indices], dtype=np.float64)
            summaries.append({
                "variable": variable, "finite_indices": finite_indices, "count": len(values),
                "mean": float(np.mean(values)) if len(values) else None,
                "std_dev": float(np.std(values, ddof=1)) if len(values) > 1 else None,
                "minimum": float(np.min(values)) if len(values) else None,
                "maximum": float(np.max(values)) if len(values) else None,
                "percentiles": np.quantile(values, np.asarray(PERCENTILES) / 100, method="linear").tolist() if len(values) else [None] * len(PERCENTILES),
            })
        cases.append({"name": name, "csv": buffer.getvalue(), "condition": condition,
                      "selected_indices": selected_indices, "rows": summaries})
    return {"provenance": {"python": platform.python_version(), "numpy": np.__version__,
                            "sample": "per-variable finite non-NULL values within TRUE selected rows; original zero-based indices",
                            "quantile_method": "linear_type_7",
                            "definition": "sorted DOUBLE values; zero-based position p*(n-1), linearly interpolate adjacent values",
                            "tolerance": "max(1e-10 absolute, 1e-9 * abs(expected)); exact counts/indices/nulls"},
            "percentiles": PERCENTILES, "cases": cases}


if __name__ == "__main__":
    output = Path(__file__).resolve().parents[1] / "tests/fixtures/summary-detail-reference.json"
    output.write_text(json.dumps(capture(), indent=2, allow_nan=False) + "\n")
