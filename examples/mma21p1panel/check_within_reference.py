#!/usr/bin/env python3
"""Development-only original-source native/R within-FE audit (stdlib only)."""
import csv
from decimal import Decimal
import hashlib
import json
import math
from pathlib import Path
import sys

native, reference, raw = map(Path, sys.argv[1:])
assert hashlib.sha256(raw.read_bytes()).hexdigest() == "2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77"


def records(path):
  with path.open(newline="") as stream:
    return list(csv.DictReader(stream))


def close(actual, expected, label, tail=False):
  expected, actual = float(expected), float(actual)
  assert math.isfinite(actual) and math.isfinite(expected), label
  tolerance = abs(expected) * 1e-8 if tail else 1e-9
  assert abs(actual - expected) <= tolerance, (label, actual, expected, tolerance)


results = {}
current = None
for line in (native / "within.jsonl").read_text().splitlines():
  event = json.loads(line)
  assert event["schema_version"] == 1 and "Error" not in event["result_type"], event
  data, kind = event["data"], event["result_type"]
  if kind == "FixedEffectsResult":
    current = data["covariance"]
    assert (current, "fit") not in results
    results[current, "fit"] = data
    assert data["estimator"] == "within_fixed_effects"
    assert (data["entity_variable"], data["time_variable"]) == ("id", "year")
    assert data["sample"] == {"total_observations": 5320, "dropped_observations": 0}
    assert data["predictors"] == data["covariance_parameter_names"] == ["lnwg"]
    assert data["common_intercept_inference"] == "normalization_point_only"
    assert data["unit_effect_normalization"] == "estimation_observation_weighted_zero_mean"
    assert data["reference_distribution"] == "student_t"
    assert "log_likelihood" not in data and len(data["coefficients"]) == 1
    if current == "cluster(id)":
      close(data["cluster"]["finite_sample_multiplier"], 532 / 531, "cluster correction")
      assert data["cluster"]["group_count"] == 532
      assert data["cluster"]["correction_convention"] == "plm_arellano_sss_nested_unit_effects"
  elif kind in ("TestResult", "LincomResult"):
    section = "test" if kind == "TestResult" else "lincom"
    assert (current, section) not in results
    results[current, section] = data
    if section == "test":
      assert data["is_chi2"] is False
assert len(results) == 6
scalars = records(reference / "reference.csv")
assert len(scalars) == 64
for row in scalars:
  covariance, section, quantity = (row[k] for k in ("covariance", "section", "quantity"))
  fit = results[covariance, "fit"]
  if section == "coefficient":
    actual = fit["coefficients"][0][quantity]
  elif section == "covariance":
    actual = fit["covariance_matrix"][0][0]
  else:
    actual = results[covariance, section][quantity]
  close(actual, row["value"], row, tail=quantity == "p_value")

actual = records(native / "panel_within_fe_predictions.csv")
expected = records(reference / "within_predictions.csv")
assert len(actual) == len(expected) == 5320
assert list(actual[0]) == list(expected[0])
cells = 0
groups = {}
for a, e in zip(actual, expected):
  for key in e:
    if key in ("source_row", "id", "year"):
      assert Decimal(a[key]) == Decimal(e[key]), (key, a, e)
    else:
      close(a[key], e[key], (key, e["source_row"]))
    cells += 1
  groups.setdefault(Decimal(a["id"]), []).append(a)
  for suffix in ("xb", "xbu", "u", "residual"):
    assert a[f"fe_classical_{suffix}"] == a[f"fe_cluster_{suffix}"]
centers = records(reference / "unit_effects.csv")
assert len(groups) == len(centers) == 532
intercepts = 0
for center in centers:
  group = groups[Decimal(center["id"])]
  assert len(group) == int(center["observations"]) == 10
  close(math.fsum(float(row["lnhr"]) for row in group) / 10, center["response_mean"], center)
  close(math.fsum(float(row["lnwg"]) for row in group) / 10, center["predictor_mean"], center)
  for covariance, prefix in (("nonrobust", "fe_classical"), ("cluster(id)", "fe_cluster")):
    effect = float(group[0][f"{prefix}_u"])
    assert all(float(row[f"{prefix}_u"]) == effect for row in group)
    close(effect, center["effect"], center)
    close(results[covariance, "fit"]["common_intercept"] + effect, center["intercept"], center)
    intercepts += 1

# Printed Stata8 source facts use a separate float32 reference, not native rounding.
author = records(reference / "author_float32.csv")
printed = records(Path(__file__).with_name("within_author_printed.csv"))
assert len(author) == len(printed) == 6
for a, e in zip(author, printed):
  assert a["quantity"] == e["quantity"]
  target = Decimal(e["value"])
  half_unit = Decimal(10) ** target.as_tuple().exponent / 2
  assert abs(Decimal(a["value"]) - target) <= half_unit, (a, e)
print(f"PASS: {len(scalars)} R scalars, {cells} exported cells / 5,320 rows, {intercepts} recovered unit-intercept comparisons; 6 author float32 printed facts")
