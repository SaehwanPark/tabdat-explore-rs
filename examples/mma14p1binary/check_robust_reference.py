"""Development-only standard-library checker for the independent R reference.

python3 check_robust_reference.py R_reference.csv logit.jsonl probit.jsonl
R_reference.csv comes from crates/tabdat-stats/tests/fixtures/binary_robust_reference.R.
Prediction/source-row validation uses reference_logit.R and reference_probit.R.
"""

import csv
import json
import math
import sys


def check(reference_path, native_paths):
  results = {}
  for path in native_paths:
    with open(path, encoding="utf-8") as source:
      for line in source:
        envelope = json.loads(line)
        data = envelope["data"]
        kind = envelope["result_type"]
        if kind in ("BinaryLogitResult", "BinaryProbitResult"):
          assert data["covariance"] == "robust_sandwich_n_over_n_minus_1"
          assert data["likelihood_ratio_p_value"] is None
          assert data["likelihood_ratio_reference_distribution"] is None
          assert data["sample"]["retained_indices"] == list(range(630))
          assert data["sample"]["dropped_observations"] == 0
          assert data["observation_count"] == 630 and data["event_count"] == 452
          assert data["include_intercept"] is True
          results[data["estimator"], "fit"] = data
        elif kind in ("LogitMarginalEffectsResult", "ProbitMarginalEffectsResult"):
          assert data["covariance"] == "robust_sandwich_n_over_n_minus_1"
          assert data["sample"] == "retained_estimation_design"
          results[data["estimator"], "ame"] = data
  fields = {"coefficient": "value", "se": "standard_error", "z": "statistic",
    "p": "p_value", "lower": "confidence_lower", "upper": "confidence_upper"}
  count = 0
  with open(reference_path, encoding="utf-8", newline="") as source:
    for row in csv.DictReader(source):
      family, quantity = row["family"], row["quantity"]
      assert row["intercept"] == "TRUE" and row["two"] == "FALSE"
      i, j = int(row["i"]), int(row["j"])
      if quantity == "covariance":
        actual = results[family, "fit"]["covariance_matrix"][i][j]
      elif quantity.startswith("ame"):
        field = "value" if quantity == "ame" else fields[quantity[4:]]
        actual = results[family, "ame"]["effects"][i][field]
      else:
        actual = results[family, "fit"]["coefficients"][i][fields[quantity]]
      expected = float(row["value"])
      tolerance = abs(expected)*1e-10 + sys.float_info.min if quantity in ("p", "ame_p") else 1e-10
      assert math.isfinite(actual) and abs(actual-expected) < tolerance, (row, actual)
      count += 1
  assert count == 44, count
  print(f"Matched {count} coefficient/covariance/delta-AME quantities against independent R.")


if __name__ == "__main__":
  assert len(sys.argv) == 4, __doc__
  check(sys.argv[1], sys.argv[2:])
