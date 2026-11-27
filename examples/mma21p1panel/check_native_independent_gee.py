#!/usr/bin/env python3
"""Audit native independent Gaussian GEE JSON/CSV against dense Base R."""

import argparse
import csv
import hashlib
import json
import math
from pathlib import Path

SOURCE_FIELDS = ("lnhr", "lnwg", "kids", "ageh", "agesq", "disab", "id", "year")
SOURCE_SHA256 = "2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77"


def rows(path):
  with path.open(newline="") as handle:
    return list(csv.DictReader(handle))


def close(actual, expected, label, tolerance=1e-9):
  actual = float(actual)
  expected = float(expected)
  if not (math.isfinite(actual) and math.isfinite(expected)):
    raise AssertionError((label, actual, expected))
  if label.endswith(".p_value") or label.endswith(".wald_p_value"):
    valid = actual == 0 if expected == 0 else abs(actual - expected) <= abs(expected) * 1e-8
  else:
    valid = abs(actual - expected) <= tolerance
  if not valid:
    raise AssertionError((label, actual, expected, abs(actual - expected)))


def compare_csv(actual, expected, label):
  if len(actual) != len(expected):
    raise AssertionError((label, "row count", len(actual), len(expected)))
  count = 0
  for index, (left, right) in enumerate(zip(actual, expected)):
    for key, value in right.items():
      if key not in left:
        raise AssertionError((label, index, "missing field", key))
      if value == "":
        if left[key] != "":
          raise AssertionError((label, index, key, left[key], "NULL"))
      elif key in {"kind", "row", "column", "covariance", "parameter"}:
        if left[key] != value:
          raise AssertionError((label, index, key, left[key], value))
      else:
        close(left[key], value, f"{label}[{index}].{key}")
        count += 1
  print(f"{label}: {len(actual)} rows and {count} numerical comparisons passed")


def main():
  parser = argparse.ArgumentParser()
  parser.add_argument("json", type=Path)
  parser.add_argument("predictions", type=Path)
  parser.add_argument("reference", type=Path)
  parser.add_argument("mom", type=Path)
  args = parser.parse_args()

  raw = args.mom.read_bytes()
  if hashlib.sha256(raw).hexdigest() != SOURCE_SHA256:
    raise AssertionError("unexpected publisher MOM.dat SHA256")
  if len(raw.split()) != 5320 * len(SOURCE_FIELDS):
    raise AssertionError("publisher MOM.dat source shape differs")
  if (args.reference / "MOM.md5").read_text().strip() != hashlib.md5(raw).hexdigest():
    raise AssertionError("Base-R source MD5 differs")

  events = [json.loads(line) for line in args.json.read_text().splitlines() if line.strip()]
  if not events or any(event.get("schema_version") != 1 or "Error" in event.get("result_type", "") for event in events):
    raise AssertionError("JSON workflow contains errors or an unsupported schema")
  fits = [event["data"] for event in events if event.get("result_type") == "GaussianGeeResult"]
  if len(fits) != 1:
    raise AssertionError(("native Gaussian GEE result count", len(fits)))
  fit = fits[0]
  assert fit["estimator"] == "gaussian_identity_independent_gee"
  assert fit["correlation_structure"] == "independent"
  assert fit["correlation"] == "fixed_identity"
  assert fit["working_covariance"] == "rss_over_n_times_inverse_x_prime_x"
  assert fit["sandwich_covariance"] == "uncorrected_unit_sandwich"
  assert fit["reference_distribution"] == "asymptotic_standard_normal"
  assert fit["sample"] == {"total_observations": 5320, "dropped_observations": 0}
  assert (fit["observation_count"], fit["group_count"], fit["minimum_group_size"], fit["maximum_group_size"]) == (5320, 532, 10, 10)
  assert fit["singleton_group_count"] == 0 and fit["rho"] == 0 and fit["ordered_pair_count"] == 0
  assert fit["iterations"] == 1 and fit["max_abs_scaled_score"] <= 1e-10

  ref_fit = rows(args.reference / "fit.csv")
  assert len(ref_fit) == 1
  actual_fit = {
    "observations": fit["observation_count"],
    "groups": fit["group_count"],
    "scale": fit["phi"],
    "intercept": fit["working"]["coefficients"][0]["value"],
    "slope": fit["working"]["coefficients"][1]["value"],
    "working_correlation": fit["rho"],
    "inverse_bread_scaled_score": fit["max_abs_scaled_score"],
  }
  for key, actual in actual_fit.items():
    close(actual, ref_fit[0][key], f"fit.{key}")

  reference_covariance = rows(args.reference / "covariance.csv")
  actual_covariance = []
  for kind, section in (("working", fit["working"]), ("sandwich", fit["sandwich"])):
    names = [coefficient["name"] for coefficient in section["coefficients"]]
    for i, row_name in enumerate(names):
      for j, column_name in enumerate(names):
        actual_covariance.append({"kind": kind, "row": row_name, "column": column_name, "value": section["covariance"][i][j]})
  compare_csv(actual_covariance, reference_covariance, "full covariance matrices")

  reference_coefficients = rows(args.reference / "coefficients.csv")
  actual_coefficients = []
  for kind in ("working", "sandwich"):
    for coefficient in fit[kind]["coefficients"]:
      actual_coefficients.append({
        "covariance": kind,
        "parameter": coefficient["name"],
        "estimate": coefficient["value"],
        "standard_error": coefficient["standard_error"],
        "statistic": coefficient["statistic"],
        "p_value": coefficient["p_value"],
        "confidence_lower": coefficient["confidence_lower"],
        "confidence_upper": coefficient["confidence_upper"],
        "wald_statistic": "" if coefficient["name"] == "_cons" else fit[kind]["wald_statistic"],
        "wald_p_value": "" if coefficient["name"] == "_cons" else fit[kind]["wald_p_value"],
      })
  compare_csv(actual_coefficients, reference_coefficients, "coefficient inference")

  tables = [event["data"] for event in events if event.get("result_type") == "EstimateTableResult"]
  assert len(tables) == 1
  table = tables[0]
  assert [model["name"] for model in table["models"]] == ["p4_independent_working", "p4_independent_sandwich"]
  for model, covariance_kind, covariance_name in zip(
    table["models"], ("working", "sandwich"), ("working_gaussian_independent", "unit_sandwich_uncorrected")
  ):
    assert model["estimator"] == "gaussian_independent_gee"
    assert model["covariance"] == covariance_name
    assert model["observation_count"] == 5320
    assert model["panel"]["group_count"] == 532
    for coefficient, expected in zip(model["coefficients"], fit[covariance_kind]["coefficients"]):
      assert coefficient["name"] == ("intercept" if expected["name"] == "_cons" else expected["name"])
      close(coefficient["value"], expected["value"], f"table.{model['name']}.{coefficient['name']}.estimate")
      close(coefficient["standard_error"], expected["standard_error"], f"table.{model['name']}.{coefficient['name']}.standard_error")
    reference_matrix = {
      row["row"]: {}
      for row in reference_covariance
      if row["kind"] == covariance_kind
    }
    for row in reference_covariance:
      if row["kind"] == covariance_kind:
        reference_matrix[row["row"]][row["column"]] = float(row["value"])
    for i, row_name in enumerate(("intercept", "lnwg")):
      for j, column_name in enumerate(("intercept", "lnwg")):
        reference_row = "_cons" if row_name == "intercept" else row_name
        reference_column = "_cons" if column_name == "intercept" else column_name
        close(
          model["covariance_matrix"][i][j],
          reference_matrix[reference_row][reference_column],
          f"table.{model['name']}.covariance.{row_name}.{column_name}",
        )
  print("named working/sandwich snapshots preserve method identity, sample and inference")

  source_lines = raw.decode("ascii").splitlines()
  predictions = rows(args.predictions)
  reference_predictions = rows(args.reference / "predictions.csv")
  assert len(predictions) == 5320
  normalized_predictions = [
    {
      "source_row": row["source_row"],
      "lnhr": row["lnhr"],
      "lnwg": row["lnwg"],
      "id": row["id"],
      "year": row["year"],
      "fitted_mean": row["native_independent_mean"],
      "residual": row["native_independent_residual"],
    }
    for row in predictions
  ]
  compare_csv(normalized_predictions, reference_predictions, "source-ordered predictions")
  for index, (actual, expected, source_line) in enumerate(zip(predictions, reference_predictions, source_lines), 1):
    tokens = source_line.split()
    if float(actual["source_row"]) != index or [float(actual[field]) for field in SOURCE_FIELDS] != [float(value) for value in tokens]:
      raise AssertionError(("source identity/order", index))
  print("publisher source identity/order: 5,320 rows and 42,560 original values preserved")


if __name__ == "__main__":
  main()
