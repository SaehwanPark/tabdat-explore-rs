"""Dev-only pandas reference for selected frequency samples and denominators."""
import json
import platform
from pathlib import Path

import pandas as pd

records = [
  [1, "A", "X", 1, 2],
  [1, "A", "Y", 2, 5],
  [0, "B", "X", 3, 5],
  [1, "B", "X", 4, 9],
  [None, "B", "Y", 5, 8],
  [1, None, "X", 6, 10],
  [1, "A", None, 7, 12],
  [1, None, None, 8, 13],
]
frame = pd.DataFrame(records, columns=["e", "r", "c", "x", "y"])
selected = frame.loc[frame.e.eq(1)]
sentinel = "<SQL NULL>"

def key(value):
  return None if value == sentinel else value

def levels(series):
  return sorted(value for value in series.unique() if value != sentinel) + (
    [sentinel] if sentinel in series.values else []
  )

cases = {}
for missing in [False, True]:
  one = selected.copy()
  if not missing:
    one = one.dropna(subset=["r"])
  one["r"] = one.r.fillna(sentinel)
  counts = one.r.value_counts()
  cases[f"one_{missing}"] = [
    [key(value), int(counts[value]), 100.0 * counts[value] / counts.sum()]
    for value in levels(one.r)
  ]
  two = selected.copy()
  if not missing:
    two = two.dropna(subset=["r", "c"])
  two[["r", "c"]] = two[["r", "c"]].fillna(sentinel)
  table = pd.crosstab(two.r, two.c)
  row_levels, column_levels = levels(two.r), levels(two.c)
  rows = []
  for row in row_levels:
    output = [key(row)]
    for column in column_levels:
      count = int(table.loc[row, column])
      output.extend([count, 100.0 * count / table.loc[row].sum(), 100.0 * count / table[column].sum()])
    rows.append(output)
  cases[f"two_{missing}"] = {"column_keys": [key(value) for value in column_levels], "rows": rows}
result = {
  "reference": {"python": platform.python_version(), "pandas": pd.__version__, "selection": "e == 1; FALSE/NULL excluded", "sample": "selected rows intersect category SQL-NULL exclusions unless missing", "percentage_tolerance": 1e-10},
  "records": records,
  "selected_indices": selected.index.tolist(),
  "cases": cases,
}
path = Path(__file__).resolve().parents[1] / "tests/fixtures/conditional-frequency-reference.json"
path.write_text(json.dumps(result, indent=2, allow_nan=False) + "\n")
