"""Development-only six-model/reference checker; Python is not needed by TabDat.

python3 check_comparison.py R_comparison.csv R_predictions.csv native.jsonl native_predictions.csv
"""

import csv
from decimal import Decimal
import json
import math
import sys


def read_csv(path):
  with open(path, encoding="utf-8", newline="") as source:
    return list(csv.DictReader(source))


def close(actual, expected, *, tail=False, design=False):
  actual, expected = float(actual), float(expected)
  tolerance = (abs(expected)*1e-10 + sys.float_info.min if tail else
    1e-12 if design else 1e-10)
  assert math.isfinite(actual) and abs(actual-expected) < tolerance, (actual, expected, tolerance)


def check(reference, predictions, native, native_predictions):
  fits = {}
  order = []
  covariance_keys = {
    "model_based_fisher_information": ("logit", "classical"),
    "model_based_observed_information": ("probit", "classical"),
  }
  with open(native, encoding="utf-8") as source:
    for line in source:
      envelope = json.loads(line)
      assert envelope["schema_version"] == 1
      if envelope["result_type"] not in ("BinaryLogitResult", "BinaryProbitResult", "RegressionResult"):
        continue
      fit = envelope["data"]
      family = fit["estimator"]
      if family == "ols":
        assert fit["covariance"] in ("nonrobust", "robust")
        mode = "classical" if fit["covariance"] == "nonrobust" else "robust"
        assert fit["log_likelihood_convention"] == "gaussian_working_model_ml_variance_rss_over_n"
        assert fit["covariance_parameter_names"] == ["intercept", "lnrelp"]
        assert fit["sample"] == {"total_observations": 630, "dropped_observations": 0}
      else:
        assert family in ("logit", "probit")
        mode = "robust" if fit["covariance"] == "robust_sandwich_n_over_n_minus_1" else "classical"
        if mode == "classical":
          assert covariance_keys[fit["covariance"]] == (family, mode)
        else:
          assert fit["likelihood_ratio_p_value"] is None
          assert fit["omnibus_wald"]["degrees_of_freedom"] == 1
        assert fit["event_count"] == 452
        assert fit["sample"]["retained_indices"] == list(range(630))
        assert fit["sample"]["dropped_observations"] == 0
      assert fit["outcome"] == "charter" and fit["predictors"] == ["lnrelp"]
      assert fit["include_intercept"] is True and fit["observation_count"] == 630
      assert [c["name"] for c in fit["coefficients"]] == ["intercept", "lnrelp"]
      key = family, mode
      assert key not in fits
      fits[key] = fit
      order.append(key)
  assert order == [(family, mode) for mode in ("classical", "robust") for family in ("logit", "probit", "ols")]
  references = read_csv(reference)
  assert len(references) == 94
  for row in references:
    fit = fits[row["family"], row["mode"]]
    i, j = int(row["i"]), int(row["j"])
    quantity = row["quantity"]
    if quantity == "covariance":
      actual = fit["covariance_matrix"][i][j]
    elif quantity in ("coefficient", "standard_error", "statistic", "p_value"):
      actual = fit["coefficients"][i]["value" if quantity == "coefficient" else quantity]
    else:
      actual = fit[quantity]
    close(actual, row["value"], tail=quantity == "p_value")
  expected_rows, actual_rows = read_csv(predictions), read_csv(native_predictions)
  assert len(expected_rows) == len(actual_rows) == 630
  assert sum(int(row["charter"]) for row in expected_rows) == 452
  for expected, actual in zip(expected_rows, actual_rows):
    for field in ("source_row", "mode", "charter"):
      assert Decimal(expected[field]) == Decimal(actual[field]), (field, expected, actual)
    for field in expected.keys() - {"source_row", "mode", "charter"}:
      close(actual[field], expected[field], design=field in ("pcharter", "ppier", "lnrelp"))
    for family in ("logit", "probit", "ols"):
      for kind in (("pr", "xb") if family != "ols" else ("xb",)):
        assert actual[f"{family}_classical_{kind}"] == actual[f"{family}_robust_{kind}"]
  linear = [float(row["ols_classical_xb"]) for row in actual_rows]
  assert min(linear) < 0 and max(linear) > 1
  print(f"Matched 94 six-model quantities and all 630 source/outcome/design/prediction records; OLS range [{min(linear):.9f}, {max(linear):.9f}], not clipped.")
  print("\nNative-value comparison (development-only checker, not a stored-fit table command):")
  print("Quantity\t" + "\t".join(f"{f}/{m}" for f, m in order))
  for i, name in enumerate(("intercept", "lnrelp")):
    for field in ("value", "statistic"):
      print(f"{name} {field}\t" + "\t".join(f"{fits[key]['coefficients'][i][field]:.3f}" for key in order))
  for field in ("observation_count", "log_likelihood", "r_squared", "mcfadden_r_squared"):
    print(field + "\t" + "\t".join("NA" if fits[key].get(field) is None else f"{fits[key][field]:.3f}" for key in order))
  print("Binary statistics are normal z; OLS statistics are t(628). Gaussian working-model and Bernoulli likelihoods are not a cross-family LR test.")


if __name__ == "__main__":
  assert len(sys.argv) == 5, __doc__
  check(*sys.argv[1:])
