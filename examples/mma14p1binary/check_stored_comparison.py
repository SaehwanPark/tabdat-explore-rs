"""Development-only native stored Table14.2 comparison against the base-R reference.

python3 check_stored_comparison.py R_comparison.csv R_predictions.csv native.jsonl native_predictions.csv
"""

import json
import sys

sys.dont_write_bytecode = True
from check_comparison import check as check_fits, close, read_csv


def check(reference, predictions, native, native_predictions):
  # Existing independent checks still validate all sequential fits and records.
  check_fits(reference, predictions, native, native_predictions)
  with open(native, encoding="utf-8") as source:
    envelopes = [json.loads(line) for line in source]
  tables = [e for e in envelopes if e["result_type"] == "EstimateTableResult"]
  assert len(tables) == 1 and tables[0]["schema_version"] == 1
  table = tables[0]["data"]
  assert table["parameter_names"] == ["intercept", "lnrelp"]
  names = ["blogit", "bprobit", "bOLS", "bloghet", "bprobhet", "bOLShet"]
  models = table["models"]
  assert [m["name"] for m in models] == names
  assert [e["data"]["name"] for e in envelopes if e["result_type"] == "EstimateStoreResult"] == names
  order = [(family, mode) for mode in ("classical", "robust") for family in ("logit", "probit", "ols")]
  fits = dict(zip(order, models))
  for (family, mode), model in fits.items():
    assert model["estimator"] == family
    assert model["outcome"] == "charter" and model["predictors"] == ["lnrelp"]
    assert model["include_intercept"] is True and model["observation_count"] == 630
    assert model["covariance_parameter_names"] == ["intercept", "lnrelp"]
    assert [c["name"] for c in model["coefficients"]] == ["intercept", "lnrelp"]
    assert model["cluster_variable"] is None and model["cluster_count"] is None
    assert model["sample"] == {"retained_indices": list(range(630)), "total_observations": 630,
      "dropped_observations": 0, "weights": None, "cluster_groups": None}
    if family == "ols":
      assert model["reference_distribution"] == "student_t"
      assert model["inference_degrees_of_freedom"] == 628
      assert model["event_count"] is None and model["mcfadden_r_squared"] is None
      assert model["covariance"] == ("nonrobust" if mode == "classical" else "robust")
      assert model["log_likelihood_convention"] == "gaussian_working_model_ml_variance_rss_over_n"
    else:
      assert model["reference_distribution"] == "asymptotic_standard_normal"
      assert model["inference_degrees_of_freedom"] is None
      assert model["event_count"] == 452 and model["r_squared"] is None
      assert model["log_likelihood_convention"] == "bernoulli"
      assert model["covariance"] == ("robust_sandwich_n_over_n_minus_1" if mode == "robust" else
        "model_based_fisher_information" if family == "logit" else "model_based_observed_information")
  references = read_csv(reference)
  assert len(references) == 94
  for row in references:
    model = fits[row["family"], row["mode"]]
    i, j, quantity = int(row["i"]), int(row["j"]), row["quantity"]
    if quantity == "covariance":
      actual = model["covariance_matrix"][i][j]
    elif quantity in ("coefficient", "standard_error", "statistic", "p_value"):
      actual = model["coefficients"][i]["value" if quantity == "coefficient" else quantity]
    else:
      actual = model[quantity]
    close(actual, row["value"], tail=quantity == "p_value")
  print("Native stored table independently matches all 94 R quantities, selected covariance/reference/likelihood/sample metadata and requested six-column order; 630 prediction records remain checked.")


if __name__ == "__main__":
  assert len(sys.argv) == 5, __doc__
  check(*sys.argv[1:])
