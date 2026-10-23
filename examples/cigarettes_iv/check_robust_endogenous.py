"""Development-only audit of native output against base-R reference evidence."""
import csv
import json
import math
import pathlib
import sys

work = pathlib.Path(sys.argv[1])
rows = [json.loads(line) for line in (work / "robust-endogenous.jsonl").read_text().splitlines()]
data = next(row["data"] for row in rows if row["result_type"] == "RobustEndogeneityResult")
# Independent R 4.6.1 reference_robust_endogenous.R, on the pinned source/sample.
for key, expected in (
  ("statistic", 3.504844914950468),
  ("p_value", 0.06118957434570806),
  ("covariance_correction", 48 / 44),
):
  assert math.isfinite(data[key]) and abs(data[key] - expected) < 1e-12, (key, data[key], expected)
assert data["observation_count"] == 48
assert data["auxiliary_parameter_count"] == 4
assert data["auxiliary_residual_df"] == 44
assert data["degrees_of_freedom"] == 1
assert data["covariance"] == "robust_hc1"
assert data["reference_distribution"] == "asymptotic_chi_square"
assert data["weak_iv_robust"] is False
with (work / "robust_endogenous_sample.csv").open() as source:
  actual = list(csv.DictReader(source))
with (work / "expected_robust_sample.csv").open() as source:
  expected = list(csv.DictReader(source))
assert len(actual) == len(expected) == 48
for native, reference in zip(actual, expected):
  assert native["state"] == reference["state"]
  for key in ("lpacks", "lrprice", "lrincome", "tdiff", "rtax"):
    a, b = float(native[key]), float(reference[key])
    assert math.isfinite(a) and math.isfinite(b) and abs(a - b) < 1e-12, (key, a, b)
print("Validated 48 ordered state identities, 240 prepared cells and HC1 Wald/p/correction (<1e-12 absolute).")
