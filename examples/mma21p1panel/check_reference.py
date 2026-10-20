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
for case in ("pooled", "fd"):
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
      assert data["sample"] == {"total_observations": 5320 if case == "pooled" else 4788, "dropped_observations": 0}
      assert data["covariance_parameter_names"] == ["intercept", "lnwg" if case == "pooled" else "dlnwg"]
    elif kind in ("TestResult", "LincomResult"):
      section = "test" if kind == "TestResult" else "lincom"
      assert (case, current, section) not in results
      results[case, current, section] = data
      if section == "test":
        assert data["is_chi2"] is False
  assert sum(key[0] == case for key in results) == 9

scalars = records(reference / "reference.csv")
assert len(scalars) == 180
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
for case, filename in (("pooled", "panel_pooled_predictions.csv"), ("fd", "panel_fd_predictions.csv")):
  actual = records(native / filename)
  expected = records(reference / f"{case}_predictions.csv")
  assert len(actual) == len(expected) == (5320 if case == "pooled" else 4788)
  assert list(actual[0]) == list(expected[0]), (actual[0].keys(), expected[0].keys())
  for a, e in zip(actual, expected):
    for key in e:
      if key in ("source_row", "previous_source_row", "id", "year", "previous_year"):
        assert float(a[key]) == float(e[key]), (case, key, a, e)
      else:
        close(a[key], e[key], (case, key, e["source_row"]))
      cells += 1

# Author Stata8 float32 facts are NOT the f64 native numerical oracle.
author = records(reference / "author_float32.csv")
printed = records(Path(__file__).with_name("author_printed.csv"))
assert len(author) == len(printed) == 12
for actual, expected in zip(author, printed):
  for key in ("case", "covariance", "parameter"):
    assert actual[key] == expected[key]
  for key in ("value", "standard_error"):
    target = Decimal(expected[key])
    half_unit = Decimal(10) ** target.as_tuple().exponent / 2
    assert abs(Decimal(actual[key]) - target) <= half_unit, (actual, expected, key)
print(f"PASS: {len(scalars)} f64 R scalars, {cells} exported cells across 10,108 rows; 24 author float32 printed facts")
