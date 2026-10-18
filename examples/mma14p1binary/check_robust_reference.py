"""Development-only standard-library checker for the independent R reference.

python3 check_robust_reference.py R_reference.csv logit.jsonl probit.jsonl
R_reference.csv comes from crates/tabdat-stats/tests/fixtures/binary_robust_reference.R.
Prediction/source-row validation uses reference_logit.R and reference_probit.R.
Clustered external example:
  python3 check_robust_reference.py --cluster R_predictions.csv native_predictions.csv R_reference.csv native.jsonl
"""

import csv
import json
import math
import sys


def check(reference_path, native_paths, cluster_predictions=None):
  results = {}
  n, events, group_count = 630, 452, None
  if cluster_predictions:
    with open(cluster_predictions[0], encoding="utf-8", newline="") as source:
      reference_rows = list(csv.DictReader(source))
    with open(cluster_predictions[1], encoding="utf-8", newline="") as source:
      native_rows = list(csv.DictReader(source))
    n = len(reference_rows)
    assert n == 1934 and len(native_rows) == n
    events = sum(int(row["y"]) for row in reference_rows)
    group_count = len({row["district"] for row in reference_rows})
    assert events == 759 and group_count == 60
    for expected, actual in zip(reference_rows, native_rows):
      for field in ("woman", "district", "y"):
        assert int(expected[field]) == int(actual[field]), (field, expected, actual)
      for field in ("age", "logit_pr", "logit_xb", "probit_pr", "probit_xb"):
        tolerance = 1e-12 if field == "age" else 1e-10
        assert abs(float(expected[field])-float(actual[field])) < tolerance, (field, expected, actual)
    print("Matched all 1934 person/district/outcome/design/prediction identities and values.")
  covariance = "cluster_sandwich_g_over_g_minus_1" if group_count else "robust_sandwich_n_over_n_minus_1"
  for path in native_paths:
    with open(path, encoding="utf-8") as source:
      for line in source:
        envelope = json.loads(line)
        data = envelope["data"]
        kind = envelope["result_type"]
        if kind in ("BinaryLogitResult", "BinaryProbitResult"):
          assert data["covariance"] == covariance
          assert data["reference_distribution"] == "asymptotic_standard_normal"
          if group_count:
            assert data["cluster"] == {"variable": "district", "group_count": group_count}
            assert data["sample"]["cluster_groups"] == [str(int(row["district"])) for row in reference_rows]
          assert data["likelihood_ratio_p_value"] is None
          assert data["likelihood_ratio_reference_distribution"] is None
          assert data["sample"]["retained_indices"] == list(range(n))
          assert data["sample"]["dropped_observations"] == 0
          assert data["observation_count"] == n and data["event_count"] == events
          assert data["include_intercept"] is True
          results[data["estimator"], "fit"] = data
        elif kind in ("LogitMarginalEffectsResult", "ProbitMarginalEffectsResult"):
          assert data["covariance"] == covariance
          if group_count:
            assert data["cluster"] == {"variable": "district", "group_count": group_count}
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
      if quantity.startswith("wald_"):
        wald = results[family, "fit"]["omnibus_wald"]
        assert wald["null_hypothesis"] == "all_non_intercept_coefficients_zero"
        assert wald["reference_distribution"] == "asymptotic_chi_square"
        assert wald["unavailable_reason"] is None
        actual = wald[{"wald_statistic": "statistic", "wald_df": "degrees_of_freedom", "wald_p": "p_value"}[quantity]]
      elif quantity == "covariance":
        actual = results[family, "fit"]["covariance_matrix"][i][j]
      elif quantity.startswith("ame"):
        field = "value" if quantity == "ame" else fields[quantity[4:]]
        actual = results[family, "ame"]["effects"][i][field]
      else:
        actual = results[family, "fit"]["coefficients"][i][fields[quantity]]
      expected = float(row["value"])
      tolerance = abs(expected)*1e-10 + sys.float_info.min if quantity in ("p", "ame_p", "wald_p") else 1e-10
      assert math.isfinite(actual) and abs(actual-expected) < tolerance, (row, actual)
      count += 1
  assert count == 50, count
  print(f"Matched {count} coefficient/covariance/delta-AME/omnibus-Wald quantities against independent R.")


if __name__ == "__main__":
  if len(sys.argv) == 6 and sys.argv[1] == "--cluster":
    check(sys.argv[4], [sys.argv[5]], cluster_predictions=sys.argv[2:4])
  else:
    assert len(sys.argv) == 4, __doc__
    check(sys.argv[1], sys.argv[2:])
