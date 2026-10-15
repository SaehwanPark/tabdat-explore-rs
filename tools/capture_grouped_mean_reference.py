"""Independent native grouped-mean reference; development-only standard library.

  python3 tools/capture_grouped_mean_reference.py

None denotes SQL NULL; NaN/infinities remain observations, unlike ordinary finite
summaries. statistics.mean is independent of DuckDB and the Rust implementation.
"""
import json
import math
from pathlib import Path
import platform
from statistics import mean

inputs = [
    ["B", 1.0, 10.0, None], ["A", 2.0, None, None],
    ["B", None, 30.0, None], ["A", 4.0, 40.0, None],
    [None, None, None, None], [None, 8.0, None, None],
    ["", 10.0, 70.0, None], ["C", "positive_infinity", 80.0, None],
    ["C", 3.0, 90.0, None], ["D", "negative_infinity", 100.0, None],
    ["D", 5.0, 110.0, None], ["E", "positive_infinity", 120.0, None],
    ["E", "negative_infinity", 130.0, None], ["F", "nan", 140.0, None],
    ["F", 7.0, 150.0, None],
]
nonfinite = {"nan": math.nan, "positive_infinity": math.inf, "negative_infinity": -math.inf}
rows = []
for key in sorted({row[0] for row in inputs}, key=lambda key: (key is None, key or "")):
    indices = [index for index, row in enumerate(inputs) if row[0] == key]
    samples = {}
    for column, name in enumerate(["x", "y", "bad"], start=1):
        retained = [index for index in indices if inputs[index][column] is not None]
        values = [nonfinite.get(inputs[index][column], inputs[index][column]) for index in retained]
        result = mean(values) if values else None
        if result is not None and not math.isfinite(result):
            result = "nan" if math.isnan(result) else "positive_infinity" if result > 0 else "negative_infinity"
        samples[name] = {"indices": retained, "mean": result}
    rows.append({"key": key, "row_indices": indices, "samples": samples})
reference = {
    "provenance": {
        "python": platform.python_version(),
        "method": "Python standard-library statistics.mean over per-group, per-variable non-None entries; IEEE NaN/infinities retained",
        "scope": "native AVG sample/reporting contract, not ordinary finite summarize or an overflow/stability guarantee",
        "tolerance": "max(1e-10 absolute, 1e-9 * abs(expected)) for finite means; exact nonfinite categories/NULL and sample indices",
    },
    "columns": ["g", "x", "y", "bad"], "inputs": inputs, "rows": rows,
}
path = Path(__file__).resolve().parents[1] / "tests/fixtures/grouped-mean-reference.json"
path.write_text(json.dumps(reference, indent=2, allow_nan=False) + "\n")
print(path)
