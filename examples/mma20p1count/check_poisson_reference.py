"""Development-only first-two-Poisson-columns reference, not a production dependency.

python3 check_poisson_reference.py R_reference.csv R_predictions.csv native.jsonl poisson_predictions.csv
"""

import csv
from decimal import Decimal
import json
import math
import sys

PREDICTORS = "LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK".split()
TOTALS = {"log_likelihood", "null_log_likelihood", "likelihood_ratio", "residual_deviance", "pearson_chi_square"}


def read_csv(path):
  with open(path, encoding="utf-8", newline="") as source:
    return list(csv.DictReader(source))


def close(actual, expected, quantity=""):
  if expected == "NA":
    assert actual is None, (quantity, actual)
    return
  actual, expected = float(actual), float(expected)
  tolerance = (abs(expected)*1e-10 + sys.float_info.min if quantity.endswith("p_value") else
    1e-7 if quantity in TOTALS else 1e-12 if quantity == "design" else 1e-10)
  assert math.isfinite(actual) and abs(actual-expected) <= tolerance, (quantity, actual, expected, tolerance)


def check(reference, predictions, native, native_predictions):
  with open(native, encoding="utf-8") as source:
    envelopes = [json.loads(line) for line in source]
  assert all(e["schema_version"] == 1 and e.get("error") is None for e in envelopes)
  assert [e["result_type"] for e in envelopes] == [
    "LoadResult", "IsidResult", "SummarizeResult", "PoissonResult", "PoissonMarginalEffectsResult",
    "PredictionResult", "PredictionResult", "PoissonResult", "PoissonMarginalEffectsResult",
    "PredictionResult", "PredictionResult", "ExportResult"]
  fits = [e["data"] for e in envelopes if e["result_type"] == "PoissonResult"]
  margins = [e["data"] for e in envelopes if e["result_type"] == "PoissonMarginalEffectsResult"]
  for mode, (fit, effects) in enumerate(zip(fits, margins)):
    assert fit["estimator"] == effects["estimator"] == "poisson"
    assert fit["outcome"] == "MDU" and fit["predictors"] == PREDICTORS
    assert fit["include_intercept"] is True and fit["observation_count"] == effects["observation_count"] == 20186
    assert fit["zero_count"] == 6308 and fit["count_sum"] == 57746
    assert fit["covariance"] == effects["covariance"] == ("robust_sandwich_n_over_n_minus_1" if mode else "model_based_poisson_information")
    assert fit["reference_distribution"] == effects["reference_distribution"] == "asymptotic_standard_normal"
    assert fit["confidence_level"] == effects["confidence_level"] == 95
    assert fit["covariance_parameter_names"] == [c["name"] for c in fit["coefficients"]] == ["intercept", *PREDICTORS]
    assert [e["name"] for e in effects["effects"]] == PREDICTORS
    assert effects["sample"] == "retained_estimation_design"
    assert effects["effect"] == "average_continuous_predictor_column_derivative"
    assert effects["uncertainty"] == "conditional_on_observed_design_delta_method"
    assert fit["sample"] == {"retained_indices": list(range(20186)), "total_observations": 20186,
      "dropped_observations": 0, "weights": None, "cluster_groups": None}
    assert fit["null_model"] == "intercept_only_estimated_mean"
    assert fit["log_likelihood_convention"] == "poisson_count_pmf"
    assert fit["dispersion_inference"] == "descriptive_not_a_panel_or_specification_test"
    assert fit["df_residual"] == 20168
    assert fit["converged"] is True and 1 <= fit["iterations"] <= 100
    assert fit["convergence_coordinate_system"] == "unit_column_norm" and fit["convergence_absolute_tolerance"] == 1e-10
    assert fit["max_abs_scaled_score"] <= 1e-10 and fit["max_abs_scaled_newton_step"] <= 1e-10
    if mode:
      assert fit["likelihood_ratio_p_value"] is None and fit["likelihood_ratio_reference_distribution"] is None
      assert fit["omnibus_wald"]["degrees_of_freedom"] == 17
      assert fit["omnibus_wald"]["reference_distribution"] == "asymptotic_chi_square"
      assert fit["omnibus_wald"]["unavailable_reason"] is None
    else:
      assert fit["likelihood_ratio_df"] == 17 and fit["likelihood_ratio_reference_distribution"] == "asymptotic_chi_square"
      assert fit["omnibus_wald"] is None
  refs = read_csv(reference)
  assert len(refs) == 1095
  for r in refs:
    assert r["case"] == "mma20p1count"
    mode, i, j = int(r["robust"]), int(r["i"]), int(r["j"])
    q = r["quantity"]
    if q.startswith("coefficient_"):
      actual = fits[mode]["coefficients"][i][q.removeprefix("coefficient_")]
    elif q.startswith("effect_"):
      actual = margins[mode]["effects"][i][q.removeprefix("effect_")]
    elif q == "covariance":
      actual = fits[mode]["covariance_matrix"][i][j]
    elif q.startswith("wald_"):
      actual = fits[mode]["omnibus_wald"][q.removeprefix("wald_")]
    else:
      actual = fits[mode][q]
    close(actual, r["value"], q)
  expected, actual = read_csv(predictions), read_csv(native_predictions)
  assert len(expected) == len(actual) == 20186
  assert list(actual[0]) == ["prepared_row", "MDU", *PREDICTORS,
    "poisson_classical_mu", "poisson_classical_xb", "poisson_robust_mu", "poisson_robust_xb"]
  for index, (r, n) in enumerate(zip(expected, actual), 1):
    assert Decimal(r["prepared_row"]) == Decimal(n["prepared_row"]) == index
    assert Decimal(r["MDU"]) == Decimal(n["MDU"]) and Decimal(n["MDU"]) >= 0
    for field in PREDICTORS:
      close(n[field], r[field], "design")
    for kind in ("mu", "xb"):
      assert n["poisson_classical_"+kind] == n["poisson_robust_"+kind]
      close(n["poisson_classical_"+kind], r[kind])
    close(n["poisson_classical_mu"], math.exp(float(n["poisson_classical_xb"])))
  assert max(float(r["poisson_classical_mu"]) for r in actual) > 1
  print("Matched all 1,095 model-based/robust Poisson coefficient/covariance/normal-CI/tail/LR/Wald/diagnostic/continuous-delta quantities and every field/mean/xb of all 20,186 prepared records, in original export order.")
  print("Count total 57,746; 6,308 zeros; 17 predictors + intercept. Means are not probabilities. Independent-row robust is not panel/cluster inference; prepared positions are not person IDs.")


if __name__ == "__main__":
  assert len(sys.argv) == 5, __doc__
  check(*sys.argv[1:])
