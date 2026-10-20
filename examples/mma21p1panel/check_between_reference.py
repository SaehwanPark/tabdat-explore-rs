#!/usr/bin/env python3
"""Development-only full native/R replay audit; Python stdlib only."""
import csv
from decimal import Decimal
import hashlib
import json
import math
from pathlib import Path
import sys

native, reference, raw = map(Path, sys.argv[1:])
assert hashlib.sha256(raw.read_bytes()).hexdigest() == "2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77", "unexpected MOM.dat source"


def records(path):
  with path.open(newline="") as stream:
    return list(csv.DictReader(stream))


def close(actual, expected, label, tail=False):
  expected, actual = float(expected), float(actual)
  assert math.isfinite(actual) and math.isfinite(expected), label
  tolerance = abs(expected) * 1e-8 if tail else 1e-9
  assert abs(actual - expected) <= tolerance, (label, actual, expected, tolerance)


results = {}
for case in ("between",):
  events = [json.loads(line) for line in (native / f"{case}.jsonl").read_text().splitlines()]
  current = None
  for event in events:
    assert event["schema_version"] == 1 and "Error" not in event["result_type"], event
    data = event["data"]
    kind = event["result_type"]
    if kind == "RegressionResult":
      current = data["covariance"]
      results[case, current, "fit"] = data
      assert data["estimator"] == "ols" and data["include_intercept"]
      assert data["sample"] == {"total_observations": 532, "dropped_observations": 0}
      assert data["covariance_parameter_names"] == ["intercept", "mean_lnwg"]
    elif kind in ("TestResult", "LincomResult"):
      section = "test" if kind == "TestResult" else "lincom"
      assert (case, current, section) not in results
      results[case, current, section] = data
      if section == "test":
        assert data["is_chi2"] is False
  assert sum(key[0] == case for key in results) == 6

scalars = records(reference / "reference.csv")
assert len(scalars) == 60
for row in scalars:
  case, covariance, section, quantity = (row[k] for k in ("case", "covariance", "section", "quantity"))
  fit = results[case, covariance, "fit"]
  if section == "coefficient":
    actual = fit["coefficients"][int(row["i"])][quantity]
  elif section == "covariance":
    actual = fit["covariance_matrix"][int(row["i"])][int(row["j"])]
  else:
    actual = results[case, covariance, section][quantity]
  close(actual, row["value"], row, tail=quantity == "p_value")

cells = 0
for case, filename in (("between", "panel_between_predictions.csv"),):
  actual = records(native / filename)
  expected = records(reference / f"{case}_predictions.csv")
  assert len(actual) == len(expected) == 532
  assert list(actual[0]) == list(expected[0]), (actual[0].keys(), expected[0].keys())
  for a, e in zip(actual, expected):
    for key in e:
      if key in ("id", "model_rows", "first_year", "last_year"):
        assert float(a[key]) == float(e[key]), (case, key, a, e)
      else:
        close(a[key], e[key], (case, key, e["id"]))
      cells += 1

print(f"PASS: {len(scalars)} R scalars, {cells} exported cells across 532 person means")

# Printed Stata8 facts use float32 ingestion, with half-unit printed precision.
facts = records(reference / "author_printed.csv")
assert len(facts) == 7
for fact in facts:
  target = Decimal(fact["printed"])
  assert abs(Decimal(fact["value"]) - target) <= Decimal(10) ** target.as_tuple().exponent / 2, fact
print("PASS: 7 separate author float32 printed facts")
