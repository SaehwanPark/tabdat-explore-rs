#!/usr/bin/env python3
"""Audit fixed original Swamy-Arora theta, quasi covariance and owned reports."""
import argparse
import csv
import hashlib
import json
import math
from pathlib import Path

SOURCE = "lnhr lnwg kids ageh agesq disab id year".split()
RAW = ["source_row"]+SOURCE
NAMES = ["p2_gls_iid_working", "p2_gls_hc1_working", "p2_gls_person_cluster"]


def close(actual, expected, path):
  assert actual is not None and not isinstance(actual, bool)
  tolerance = abs(expected)*1e-8 if path.endswith("p_value") else 1e-9
  assert math.isfinite(actual) and abs(actual-expected) <= tolerance, (path, actual, expected)


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
  assert 3 <= g <= 532 and n == 10*g
  events = [json.loads(l) for l in (native/"manual_gls_covariance.jsonl").read_text().splitlines()]
  select = lambda kind: [e["data"] for e in events if e["result_type"] == kind]
  canonical, = select("RandomEffectsResult")
  assert canonical["sigma_e_squared"] > 0 and canonical["sigma_u_squared"] >= 0
  assert 0 <= canonical["theta"] < 1
  if canonical["raw_sigma_u_squared"] <= 0:
    assert canonical["sigma_u_squared"] == canonical["theta"] == 0
  fits, intervals = select("RegressionResult"), select("LincomResult")
  assert len(fits) == 3 and len(intervals) == 6
  data = {"canonical": canonical, **{k: dict(fit=fits[i], lincom0=intervals[2*i], lincom1=intervals[2*i+1]) for i, k in enumerate(["iid", "hc1", "cluster"])}}
  reference_rows = list(csv.DictReader((reference/"reference.csv").open()))
  assert len(reference_rows) == len({(r["model"], r["path"]) for r in reference_rows}) == 120
  for row in reference_rows:
    close(pointer(data[row["model"]], row["path"]), float(row["value"]), row["path"])
  table, = select("EstimateTableResult")
  assert table["parameter_names"] == ["intercept", "quasi_lnwg"] and len(table["models"]) == 3
  for i, (fit, snapshot) in enumerate(zip(fits, table["models"])):
    assert fit["outcome"] == "quasi_lnhr" and fit["predictors"] == ["quasi_lnwg"]
    assert fit["include_intercept"] and fit["estimator"] == "ols"
    assert fit["covariance_parameter_names"] == ["intercept", "quasi_lnwg"]
    assert fit["covariance"] == ["nonrobust", "robust", "cluster(id)"][i]
    assert fit["sample"] == dict(total_observations=n, dropped_observations=0)
    assert snapshot["name"] == NAMES[i]
    for key in ["coefficients", "covariance_matrix", "covariance_parameter_names", "covariance", "observation_count", "df_residual", "outcome", "predictors", "include_intercept", "estimator"]:
      assert snapshot[key] == fit[key], key
    sample = snapshot["sample"]
    assert sample["total_observations"] == n and sample["dropped_observations"] == 0
    assert sample["retained_indices"] == list(range(n)) and sample["weights"] is None
    assert sample["cluster_groups"] == ([str(int(r["id"])) for r in source] if i == 2 else None)
    assert snapshot["reference_distribution"] == "student_t"
    df = g-1 if i == 2 else n-2
    assert snapshot["inference_degrees_of_freedom"] == intervals[2*i]["df_residual"] == intervals[2*i+1]["df_residual"] == df
    assert snapshot["cluster_count"] == (g if i == 2 else None)
    assert snapshot["cluster_variable"] == ("id" if i == 2 else None)
  scale = 1-canonical["theta"]
  for i in range(2):
    close(fits[0]["coefficients"][i]["value"], canonical["coefficients"][i]["value"]*(scale if i == 0 else 1), "parameter scale")
    for j in range(2):
      close(fits[0]["covariance_matrix"][i][j], canonical["covariance_matrix"][i][j]*(scale if i == 0 else 1)*(scale if j == 0 else 1), "covariance scale")
  notes = json.dumps(events)
  assert "normal inference" in notes and "Student-t" in notes and "original Swamy-Arora theta" in notes and "(1-theta)" in notes
  kinds = [e["result_type"] for e in events]
  assert kinds[-4:] == ["SummarizeResult", "LincomResult", "LincomResult", "ExportResult"]
  assert kinds.count("PredictionResult") == 4
  export, = select("ExportResult")
  assert export["path"] == export["dataset"]["path"] == "panel_manual_gls_covariance_predictions.csv"
  assert export["dataset"]["row_count"] == n
  actual = list(csv.DictReader((native/"panel_manual_gls_covariance_predictions.csv").open()))
  expected = list(csv.DictReader((reference/"predictions.csv").open()))
  assert len(actual) == len(expected) == n and list(actual[0]) == list(expected[0]) and len(actual[0]) == 23
  for i, (a, b, original) in enumerate(zip(actual, expected, source), 1):
    for key in RAW:
      assert float(a[key]) == float(b[key]) == (i if key == "source_row" else original[key]), (i, key)
    for key in a.keys()-set(RAW):
      close(float(a[key]), float(b[key]), key)
    if float(a["raw_sigma_u_squared"]) <= 0:
      assert float(a["sigma_u_squared"]) == float(a["theta"]) == 0
    for key in ["theta", "sigma_e_squared", "raw_sigma_u_squared", "sigma_u_squared"]:
      close(float(a[key]), canonical[key], key)
  print(f"PASS {'synthetic' if synthetic else 'original MOM'}:120 plm/QR fields,3 full quasi covariances/6 intervals/named exact samples;{n}x23 source/component/population/quasi cells;raw_u={canonical['raw_sigma_u_squared']};theta={canonical['theta']}")


if __name__ == "__main__":
  parser = argparse.ArgumentParser(description=__doc__)
  parser.add_argument("native", type=Path)
  parser.add_argument("reference", type=Path)
  parser.add_argument("raw", type=Path)
  parser.add_argument("--synthetic", action="store_true")
  args = parser.parse_args()
  audit(args.native, args.reference, args.raw, args.synthetic)
