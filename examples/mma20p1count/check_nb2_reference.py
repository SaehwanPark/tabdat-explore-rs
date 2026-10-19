"""Development-only NB2 Table20.5 reference, not a production dependency.

python3 check_nb2_reference.py R_nb2_reference.csv R_nb2_predictions.csv native.jsonl nb2_predictions.csv
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
  tolerance = (abs(expected)*1e-10+sys.float_info.min if quantity.endswith("p_value") else
    1e-7 if quantity in TOTALS else 1e-12 if quantity == "design" else 1e-10)
  assert math.isfinite(actual) and abs(actual-expected) <= tolerance, (quantity, actual, expected, tolerance)


def check(reference, predictions, native, native_predictions):
  with open(native, encoding="utf-8") as source:
    envelopes = [json.loads(line) for line in source]
  assert all(e["schema_version"] == 1 and e.get("error") is None for e in envelopes)
  assert [e["result_type"] for e in envelopes] == [
    "LoadResult", "IsidResult", "SummarizeResult", "NbregResult", "NbregMarginalEffectsResult",
    "PredictionResult", "PredictionResult", "NbregResult", "NbregMarginalEffectsResult",
    "PredictionResult", "PredictionResult", "ExportResult"]
  fits = [e["data"] for e in envelopes if e["result_type"] == "NbregResult"]
  margins = [e["data"] for e in envelopes if e["result_type"] == "NbregMarginalEffectsResult"]
  for mode, (fit, effects) in enumerate(zip(fits, margins)):
    assert fit["estimator"] == effects["estimator"] == "nb2"
    assert fit["outcome"] == "MDU" and fit["predictors"] == PREDICTORS and fit["include_intercept"] is True
    assert fit["observation_count"] == effects["observation_count"] == 20186
    assert fit["count_sum"] == 57746 and fit["zero_count"] == 6308
    assert fit["mean_parameter_count"] == 18 and fit["parameter_count"] == 19 and fit["df_residual"] == 20167
    assert fit["covariance"] == effects["covariance"] == ("robust_sandwich_n_over_n_minus_1" if mode else "model_based_joint_observed_information")
    assert fit["reference_distribution"] == effects["reference_distribution"] == "asymptotic_standard_normal"
    assert fit["confidence_level"] == effects["confidence_level"] == 95
    assert [c["name"] for c in fit["coefficients"]] == ["intercept", *PREDICTORS]
    assert fit["covariance_parameter_names"] == ["intercept", *PREDICTORS, "lnalpha"]
    assert [e["name"] for e in effects["effects"]] == PREDICTORS
    assert effects["sample"] == "retained_estimation_design"
    assert effects["effect"] == "average_continuous_predictor_column_derivative"
    assert effects["uncertainty"] == "conditional_on_observed_design_delta_method"
    assert effects["covariance_role"] == "beta_block_of_full_joint_beta_lnalpha_covariance"
    assert fit["sample"] == {"retained_indices": list(range(20186)), "total_observations": 20186,
      "dropped_observations": 0, "weights": None, "cluster_groups": None}
    assert fit["variance_function"] == "mu_plus_alpha_mu_squared"
    assert fit["dispersion_parameterization"] == "alpha_exp_lnalpha_size_inverse_alpha"
    assert fit["dispersion_boundary_test"] == "not_implemented_no_normal_alpha_zero_test"
    assert "p_value" not in fit["dispersion"] and "statistic" not in fit["dispersion"]
    assert fit["null_model"] == "intercept_only_mean_and_dispersion_reestimated"
    assert fit["log_likelihood_convention"] == "nb2_count_pmf"
    assert fit["dispersion_inference"] == "descriptive_not_a_panel_or_specification_test"
    assert fit["converged"] is True and 1 <= fit["iterations"] <= 100
    assert fit["iterations_role"] == "full_model_information_evaluations"
    assert fit["convergence_coordinate_system"] == "unit_mean_design_column_norm_and_log_alpha_sqrt_n"
    assert fit["convergence_absolute_tolerance"] == 1e-10
    assert 0 <= fit["max_abs_scaled_score"] <= 1e-10 and 0 <= fit["max_abs_scaled_newton_step"] <= 1e-10
    assert fit["supported_maximum_count"] == 100000 and fit["supported_log_alpha_open_bounds"] == [-12, 12]
    assert fit["likelihood_ratio_df"] == 17
    if mode:
      assert fit["likelihood_ratio_p_value"] is None and fit["likelihood_ratio_reference_distribution"] is None
      assert fit["omnibus_wald"]["degrees_of_freedom"] == 17
      assert fit["omnibus_wald"]["reference_distribution"] == "asymptotic_chi_square"
      assert fit["omnibus_wald"]["unavailable_reason"] is None
    else:
      assert fit["likelihood_ratio_reference_distribution"] == "asymptotic_chi_square" and fit["omnibus_wald"] is None
    d = fit["dispersion"]
    close(d["alpha"], math.exp(d["log_alpha"]))
    close(d["alpha_standard_error"], d["alpha"]*d["log_alpha_standard_error"])
    for bound in ("lower", "upper"):
      close(d["alpha_confidence_"+bound], math.exp(d["log_alpha_confidence_"+bound]))
  refs = read_csv(reference)
  assert len(refs) == 1187
  for r in refs:
    assert r["case"] == "mma20p1count"
    mode, i, j = int(r["robust"]), int(r["i"]), int(r["j"])
    q = r["quantity"]
    if q.startswith("coefficient_"):
      actual = fits[mode]["coefficients"][i][q.removeprefix("coefficient_")]
    elif q.startswith("effect_"):
      actual = margins[mode]["effects"][i][q.removeprefix("effect_")]
    elif q in fits[mode]["dispersion"]:
      actual = fits[mode]["dispersion"][q]
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
    "nb2_classical_mu", "nb2_classical_xb", "nb2_robust_mu", "nb2_robust_xb"]
  for index, (r, n) in enumerate(zip(expected, actual), 1):
    assert Decimal(r["prepared_row"]) == Decimal(n["prepared_row"]) == index
    assert Decimal(r["MDU"]) == Decimal(n["MDU"]) and Decimal(n["MDU"]) >= 0
    for field in PREDICTORS:
      close(n[field], r[field], "design")
    for kind in ("mu", "xb"):
      assert n["nb2_classical_"+kind] == n["nb2_robust_"+kind]
      close(n["nb2_classical_"+kind], r[kind])
    close(n["nb2_classical_mu"], math.exp(float(n["nb2_classical_xb"])))
  assert max(float(r["nb2_classical_mu"]) for r in actual) > 1
  print("Matched all 1,187 joint observed/model-based and independent-row robust NB2 coefficient/covariance/normal-CI/tail/dispersion/LR/Wald/diagnostic/delta quantities and every field/mean/xb of all 20,186 prepared records.")
  print("Count total57,746; 6,308 zeros; 17 predictors+intercept+lnalpha. Means are not probabilities. Independent-row robust is not panel-robust; prepared positions are not person IDs.")


if __name__ == "__main__":
  assert len(sys.argv) == 5, __doc__
  check(*sys.argv[1:])
