"""Development-only native score/sample audit against independent base-R evidence."""
import csv
import hashlib
import json
import math
import pathlib
import sys

work = pathlib.Path(sys.argv[1])
assert hashlib.sha256((work / "CigarettesSW.csv").read_bytes()).hexdigest() == (
  "629e887a8104d483d450a9bbe487cb5613e62d116c4ecc84d6c8893b7393fcd2"
)
rows = [json.loads(line) for line in (work / "robust-overid.jsonl").read_text().splitlines()]
data = next(row["data"] for row in rows if row["result_type"] == "RobustOveridScoreResult")
# Base R4.6.1 reference_robust_overid.R; no N-K/parent HC1 correction.
for key, expected in (("statistic", 0.3347358817061415), ("p_value", 0.5628836468492445)):
  assert math.isfinite(data[key]) and abs(data[key] - expected) < 1e-12, (key, data[key], expected)
assert data["observation_count"] == 48
assert data["instrument_parameter_count"] == 4
assert data["structural_parameter_count"] == 3
assert data["degrees_of_freedom"] == 1
assert data["covariance_correction"] == 1.0
assert data["covariance"] == "independent_row_uncentered_score_opg"
assert data["reference_distribution"] == "asymptotic_chi_square"
assert data["weak_iv_robust"] is False
assert data["endogenous"] == ["lrprice"]
assert data["excluded_instruments"] == ["tdiff", "rtax"]
with (work / "robust_overid_sample.csv").open() as source:
  actual = list(csv.DictReader(source))
with (work / "expected_robust_overid_sample.csv").open() as source:
  expected = list(csv.DictReader(source))
assert len(actual) == len(expected) == 48
for native, reference in zip(actual, expected):
  assert native["state"] == reference["state"]
  for key in ("lpacks", "lrprice", "lrincome", "tdiff", "rtax"):
    a, b = float(native[key]), float(reference[key])
    assert math.isfinite(a) and math.isfinite(b) and abs(a - b) < 1e-12, (key, a, b)
print("Validated source hash, 48 ordered states, 240 design cells and robust score/p (<1e-12 absolute).")
