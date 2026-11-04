#!/usr/bin/env python3
"""Audit the bounded manual FE working-covariance comparison, not its error law."""
import argparse
import csv
import hashlib
import json
import math
from pathlib import Path

RAW = "source_row id year lnhr lnwg kids ageh agesq disab".split()
SOURCE = "lnhr lnwg kids ageh agesq disab id year".split()
NAMES = ["p2_fe_iid_working", "p2_fe_hc1_working", "p2_fe_person_cluster"]


def close(actual, expected, path):
  assert actual is not None
  tolerance = abs(expected) * 1e-8 if path.endswith("p_value") else 1e-9
  assert math.isfinite(actual) and abs(actual - expected) <= tolerance, (path, actual, expected)


def pointer(data, path):
  for key in path.split("/")[1:]:
    data = data[int(key)] if isinstance(data, list) else data[key]
  return data


def audit(native, reference, raw, synthetic=False):
  if not synthetic:
    assert hashlib.sha256(raw.read_bytes()).hexdigest() == "2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77"
  values = [float(v) for v in raw.read_text().split()]
  assert len(values) % 8 == 0 and all(math.isfinite(v) for v in values)
  source = [dict(zip(SOURCE, values[i:i+8])) for i in range(0, len(values), 8)]
  n, g = len(source), len({r["id"] for r in source})
  assert 3 <= g <= 532 and n == 10 * g
  events = [json.loads(l) for l in (native / "manual_fe_covariance.jsonl").read_text().splitlines()]
  select = lambda kind: [e["data"] for e in events if e["result_type"] == kind]
  canonical, = select("FixedEffectsResult")
  fits, combinations = select("RegressionResult"), select("LincomResult")
  assert len(fits) == len(combinations) == 3
  data = {"canonical": canonical, **{k: dict(fit=f, lincom=l) for k, f, l in zip(["iid", "hc1", "cluster"], fits, combinations)}}
  rows = list(csv.DictReader((reference / "reference.csv").open()))
  assert len(rows) == len({(r["model"], r["path"]) for r in rows}) == 68
  for row in rows:
    close(pointer(data[row["model"]], row["path"]), float(row["value"]), row["path"])
  table, = select("EstimateTableResult")
  assert table["parameter_names"] == ["intercept", "within_lnwg"]
  assert len(table["models"]) == 3
  for i, (fit, combination, snapshot) in enumerate(zip(fits, combinations, table["models"])):
    assert fit["outcome"] == "within_lnhr" and fit["predictors"] == ["within_lnwg"]
    assert fit["include_intercept"] and fit["estimator"] == "ols"
    assert fit["covariance_parameter_names"] == ["intercept", "within_lnwg"]
    assert fit["covariance"] == ["nonrobust", "robust", "cluster(id)"][i]
    assert fit["sample"] == dict(total_observations=n, dropped_observations=0)
    assert abs(fit["coefficients"][0]["value"]) <= 1e-12
    assert snapshot["name"] == NAMES[i]
    for key in ["coefficients", "covariance_matrix", "covariance_parameter_names", "covariance", "observation_count", "df_residual", "outcome", "predictors", "include_intercept", "estimator"]:
      assert snapshot[key] == fit[key], key
    sample = snapshot["sample"]
    assert sample["total_observations"] == n and sample["dropped_observations"] == 0
    assert sample["retained_indices"] == list(range(n)) and sample["weights"] is None
    expected_clusters = [str(int(row["id"])) for row in source] if i == 2 else None
    assert sample["cluster_groups"] == expected_clusters
    assert snapshot["reference_distribution"] == "student_t"
    assert snapshot["inference_degrees_of_freedom"] == combination["df_residual"] == (g-1 if i == 2 else n-2)
    assert snapshot["cluster_count"] == (g if i == 2 else None)
    assert snapshot["cluster_variable"] == ("id" if i == 2 else None)
  clustered = fits[2]
  assert clustered["coefficients"][0]["standard_error"] <= 1e-12
  assert max(abs(v) for v in clustered["covariance_matrix"][0]) <= 1e-12
  assert "floating-point artifacts" in json.dumps(events) and "adequate cluster asymptotics" in json.dumps(events)
  export, = select("ExportResult")
  kinds = [e["result_type"] for e in events]
  assert kinds[-3:] == ["SummarizeResult", "LincomResult", "ExportResult"]
  assert kinds.count("PredictionResult") == 2
  assert max(i for i, kind in enumerate(kinds) if kind == "PredictionResult") < len(kinds)-2
  assert export["path"] == export["dataset"]["path"] == "panel_manual_fe_covariance_predictions.csv"
  assert export["dataset"]["row_count"] == n
  actual = list(csv.DictReader((native / "panel_manual_fe_covariance_predictions.csv").open()))
  expected = list(csv.DictReader((reference / "predictions.csv").open()))
  assert len(actual) == len(expected) == n
  assert list(actual[0]) == list(expected[0]) and len(actual[0]) == 15
  for i, (a, b, original) in enumerate(zip(actual, expected, source), 1):
    for key in RAW:
      assert float(a[key]) == float(b[key]) == (i if key == "source_row" else original[key]), (i, key)
    for key in a.keys() - set(RAW):
      close(float(a[key]), float(b[key]), key)
  print(f"PASS {'synthetic' if synthetic else 'original MOM'}:68 QR fields,3 full covariances/slopes/intervals/named reports;{n}x15 source/mean/prediction cells;G={g};not centered-intercept t/p validation or nominal coverage")


if __name__ == "__main__":
  parser = argparse.ArgumentParser(description=__doc__)
  parser.add_argument("native", type=Path)
  parser.add_argument("reference", type=Path)
  parser.add_argument("raw", type=Path)
  parser.add_argument("--synthetic", action="store_true")
  args = parser.parse_args()
  audit(args.native, args.reference, args.raw, args.synthetic)
