"""Compare the independent original-person Base R reference with native JSON.

python3 check_cluster_count_gof_author_reference.py reference.csv native.jsonl
"""
import csv
import json
import math
import sys


def read_records(path):
  with open(path, encoding="utf-8") as source:
    return [json.loads(line) for line in source]


def reference_rows(path):
  with open(path, encoding="utf-8", newline="") as source:
    return list(csv.DictReader(source))


def close(actual, expected, quantity):
  if quantity in {"observation_count", "tail_from", "group_count", "degrees_of_freedom", "observed_count"}:
    assert actual == expected, (quantity, actual, expected)
    return abs(actual-expected)
  tolerance = (abs(expected)*1e-10 if quantity == "p_value" else
    1e-7 if quantity == "log_likelihood" else
    1e-10 if quantity in {"statistic", "moment_covariance", "fitted_fraction", "coefficient", "alpha"} else
    1e-12)
  assert math.isfinite(actual) and abs(actual-expected) <= tolerance, (
    quantity, actual, expected, tolerance)
  return abs(actual-expected)


def check(reference_path, native_path):
  envelopes = read_records(native_path)
  fits = [row["data"] for row in envelopes if row["result_type"] == "NbregResult"]
  tests = [row["data"] for row in envelopes if row["result_type"] == "CountGoodnessOfFitResult"]
  assert len(fits) == len(tests) == 1
  fit, test = fits[0], tests[0]
  assert fit["converged"] and test["cluster"]["variable"] == "person_id"
  assert fit["observation_count"] == test["observation_count"] == 20186
  assert test["cluster"]["group_count"] == 5908
  assert len(test["table"]["bins"]) == 11
  assert len(test["moment_covariance"]) == 10

  maximum_error = 0.0
  count = 0
  for row in reference_rows(reference_path):
    assert row["case"] == "mma20p1_person"
    quantity, i, j = row["quantity"], int(row["i"]), int(row["j"])
    expected = float(row["value"])
    if quantity == "coefficient":
      actual = fit["coefficients"][i]["value"]
    elif quantity == "alpha":
      actual = fit["dispersion"]["alpha"]
    elif quantity == "log_likelihood":
      actual = fit["log_likelihood"]
    elif quantity == "group_count":
      actual = test["cluster"]["group_count"]
    elif quantity == "covariance_correction":
      actual = test["cluster"]["covariance_correction"]
    elif quantity == "moment_covariance":
      actual = test["moment_covariance"][i][j]
    elif quantity == "observed_count":
      actual = test["table"]["bins"][i]["observed_count"]
    elif quantity == "fitted_fraction":
      actual = test["table"]["bins"][i]["fitted_fraction"]
    else:
      actual = test[quantity]
    maximum_error = max(maximum_error, close(actual, expected, quantity))
    count += 1
  print(
    f"Matched {count} original-person clustered NB2/GOF reference values; "
    f"max abs error {maximum_error:.6g}."
  )


if __name__ == "__main__":
  assert len(sys.argv) == 3, __doc__
  check(sys.argv[1], sys.argv[2])
