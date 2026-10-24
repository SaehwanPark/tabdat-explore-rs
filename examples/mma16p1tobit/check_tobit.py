"""Development-only original-source Tobit/R audit; standard library only.

Usage: python3 check_tobit.py WORK_DIRECTORY
Scalar gates:1e-9 absolute+1e-8 relative; positive tails:1e-8 relative;
probabilities:1e-10 absolute. Solver certificates are gated, not equated across
Rust's concave scaled coordinates and R's natural beta/lnsigma coordinates.
"""
import csv
import hashlib
import json
import math
import pathlib
import sys

FIELDS = ["value", "standard_error", "statistic", "p_value", "confidence_lower", "confidence_upper"]
SOURCE = ["e", "lnwage", "ystar", "ytrunc", "ycens", "dy"]
PREDICTIONS = ["latent", "uncensored_probability", "observed_mean"]
NORMAL_95 = 1.959963984540054


def finite(value):
  value = float(value)
  assert math.isfinite(value), value
  return value


def close(actual, expected, key):
  actual, expected = finite(actual), finite(expected)
  assert abs(actual - expected) <= 1e-9 + 1e-8 * abs(expected), (key, actual, expected)
  if key.endswith("p_value"):
    assert actual > 0 and expected > 0 and abs(actual / expected - 1) <= 1e-8, (key, actual, expected)


def probability(actual, expected):
  actual, expected = finite(actual), finite(expected)
  assert 0 <= actual <= 1 and 0 <= expected <= 1 and abs(actual - expected) <= 1e-10


def inference(row):
  value, se, statistic, tail, low, high = [finite(row[k]) for k in FIELDS]
  assert se > 0
  close(statistic, value / se, "normal statistic")
  expected_tail = math.erfc(abs(statistic) / math.sqrt(2))
  assert expected_tail > 0 and tail > 0 and abs(tail / expected_tail - 1) < 1e-8, "positive normal p-value"
  close(low, value - NORMAL_95 * se, "normal lower")
  close(high, value + NORMAL_95 * se, "normal upper")


def read_csv(work, name):
  with (work / name).open() as stream:
    return list(csv.DictReader(stream))


def likelihood(beta, sigma, source):
  """Independently differentiate Gaussian density/censor mass in beta/lnsigma."""
  information = [[0.0] * 3 for _ in range(3)]
  score = [0.0] * 3
  ll = 0.0
  for row in source:
    x, y = float(row[1]), float(row[4])
    design = [1.0, x]
    mu = beta[0] + beta[1] * x
    if y > 0:
      r = (y - mu) / sigma
      ll += -math.log(sigma) - 0.5 * r * r - 0.5 * math.log(2 * math.pi)
      location, log_scale = r / sigma, r * r - 1
      bb, be, ee = 1 / sigma**2, 2 * r / sigma, 2 * r * r
    else:
      q = -mu / sigma
      mass = 0.5 * math.erfc(-q / math.sqrt(2))
      assert mass > 0
      mills = math.exp(-0.5 * q * q) / math.sqrt(2 * math.pi) / mass
      curvature = mills * (q + mills)
      ll += math.log(mass)
      location, log_scale = -mills / sigma, -q * mills
      bb, be, ee = curvature / sigma**2, (q * curvature - mills) / sigma, q * q * curvature - q * mills
    for a in range(2):
      score[a] += design[a] * location
      for b in range(2):
        information[a][b] += design[a] * design[b] * bb
      information[a][2] += design[a] * be
      information[2][a] += design[a] * be
    score[2] += log_scale
    information[2][2] += ee
  return ll, score, information


def audit(work):
  raw = (work / "mma16p1tobit.asc").read_bytes()
  assert hashlib.sha256(raw).hexdigest() == "8ec7ba9cf3da6854217f873441e47fd5492e777904057c8d9ca8f8ee00244ba6", "source fingerprint"
  source = [line.split() for line in raw.decode().splitlines() if line.strip()]
  assert len(source) == 200 and all(len(row) == 6 for row in source)
  records = [json.loads(line) for line in (work / "tobit.jsonl").read_text().splitlines()]
  assert [r["result_type"] for r in records] == ["LoadResult", "TobitResult", "TobitMarginalEffectsResult", "PredictionResult", "PredictionResult", "PredictionResult", "ExportResult"]
  assert all(r.get("schema_version") == 1 and "error" not in r for r in records)
  fit, effects = records[1]["data"], records[2]["data"]
  assert fit["estimator"] == effects["estimator"] == "tobit"
  assert fit["distribution"] == "iid_homoskedastic_gaussian"
  assert fit["outcome"] == "ycens" and fit["predictors"] == ["lnwage"] and fit["include_intercept"] is True
  assert fit["lower_limit"] == 0 and fit["observation_count"] == 200
  assert fit["left_censored_count"] == 70 and fit["uncensored_count"] == 130
  assert fit["censoring"] == "recorded_response_at_or_below_lower_limit"
  assert fit["covariance_parameter_names"] == ["intercept", "lnwage", "lnsigma"]
  assert fit["covariance_parameter_order"] == "location_coefficients_then_lnsigma"
  assert fit["covariance"] == effects["covariance"] == "model_based_joint_observed_beta_lnsigma_information"
  assert fit["covariance_correction"] == effects["covariance_correction"] == 1
  assert fit["reference_distribution"] == effects["reference_distribution"] == "asymptotic_standard_normal"
  assert fit["confidence_level"] == effects["confidence_level"] == 95
  assert fit["null_model"] == "intercept_and_scale_refitted_same_sample_bound"
  assert fit["likelihood_ratio_df"] == 1 and fit["likelihood_ratio_reference_distribution"] == "asymptotic_chi_square"
  assert fit["log_likelihood_convention"] == "gaussian_uncensored_density_and_left_censor_probability"
  assert fit["scale_interval"] == "exponentiated_log_scale_normal_wald_95_not_sigma_zero_test"
  assert fit["sample"]["retained_indices"] == list(range(200)) and fit["sample"]["total_observations"] == 200 and fit["sample"]["dropped_observations"] == 0
  assert fit["converged"] is True and fit["iterations"] > 0
  for name in ["max_abs_scaled_score", "max_abs_scaled_newton_step"]:
    assert 0 <= finite(fit[name]) <= 1e-10
  assert fit["convergence_coordinate_system"] == "rms_predictors_delta_and_scaled_inverse_sigma"
  assert fit["convergence_score_normalization"] == "per_observation_mean"
  assert fit["convergence_step_normalization"] == "1_plus_abs_parameter" and fit["convergence_tolerance"] == 1e-10
  assert effects["effect"] == "average_continuous_observed_censored_mean_derivative"
  assert effects["sample"] == "retained_estimation_design" and effects["observation_count"] == 200 and effects["lower_limit"] == 0
  assert effects["uncertainty"] == "conditional_on_observed_design_full_joint_beta_lnsigma_delta_method"
  assert [c["name"] for c in fit["coefficients"]] == ["intercept", "lnwage"]
  assert [e["name"] for e in effects["effects"]] == ["lnwage"]
  for row in fit["coefficients"] + effects["effects"]:
    inference(row)
  covariance = fit["covariance_matrix"]
  assert len(covariance) == 3 and all(len(row) == 3 for row in covariance)
  beta = [finite(c["value"]) for c in fit["coefficients"]]
  scale = fit["scale"]
  sigma, eta, eta_se = finite(scale["value"]), finite(scale["log_value"]), finite(scale["log_standard_error"])
  assert sigma > 0 and eta_se > 0
  close(sigma, math.exp(eta), "positive sigma")
  close(scale["standard_error"], sigma * eta_se, "sigma delta SE")
  close(scale["confidence_lower"], math.exp(eta - NORMAL_95 * eta_se), "sigma log-Wald lower")
  close(scale["confidence_upper"], math.exp(eta + NORMAL_95 * eta_se), "sigma log-Wald upper")
  ll, score, information = likelihood(beta, sigma, source)
  close(fit["log_likelihood"], ll, "source likelihood")
  assert max(abs(s) * max(1, abs(t)) for s, t in zip(score, beta + [eta])) / 200 < 1e-10, "source score stationarity"
  determinant = math.fsum(information[0][a] * (information[1][(a+1)%3] * information[2][(a+2)%3] - information[1][(a+2)%3] * information[2][(a+1)%3]) for a in range(3))
  assert information[0][0] > 0 and information[0][0] * information[1][1] - information[0][1]**2 > 0 and determinant > 0
  for i in range(3):
    for j in range(3):
      close(covariance[i][j], covariance[j][i], "symmetric joint covariance")
      assert abs(math.fsum(information[i][k] * finite(covariance[k][j]) for k in range(3)) - (i == j)) < 1e-8, "joint information inverse"
    se = fit["coefficients"][i]["standard_error"] if i < 2 else eta_se
    close(float(se)**2, covariance[i][i], "joint SE variance")
  lr = finite(fit["likelihood_ratio"])
  assert lr > 0
  close(lr, 2 * (ll - finite(fit["null_log_likelihood"])), "same-sample LR")
  close(fit["likelihood_ratio_p_value"], math.erfc(math.sqrt(lr / 2)), "LR p_value")

  native = read_csv(work, "tobit_predictions.csv")
  reference = read_csv(work, "R_tobit_predictions.csv")
  prepared = read_csv(work, "mma16p1tobit.csv")
  assert len(native) == len(reference) == len(prepared) == 200
  assert list(native[0]) == list(reference[0]) == ["source_row"] + SOURCE + PREDICTIONS
  assert list(prepared[0]) == ["source_row"] + SOURCE
  gradient_terms = [[], [], []]
  probabilities = []
  for index, (a, b, c, tokens) in enumerate(zip(native, reference, prepared, source), 1):
    for data in [a, b, c]:
      assert int(data["source_row"]) == index, "source ordinal"
      for name, token in zip(SOURCE, tokens):
        if token == ".":
          assert data[name] == ""
        else:
          assert finite(data[name]) == float(token)
    mu = beta[0] + beta[1] * float(tokens[1])
    z = mu / sigma
    p = 0.5 * math.erfc(-z / math.sqrt(2))
    density = math.exp(-z * z / 2) / math.sqrt(2 * math.pi)
    for data in [a, b]:
      close(data["latent"], mu, "source latent mean")
      probability(data["uncensored_probability"], p)
      close(data["observed_mean"], mu * p + sigma * density, "source censored mean")
    for name in PREDICTIONS:
      (probability if name == "uncensored_probability" else lambda x, y: close(x, y, name))(a[name], b[name])
    probabilities.append(float(a["uncensored_probability"]))
    gradient_terms[0].append(density / sigma)
    gradient_terms[1].append(density * float(tokens[1]) / sigma)
    gradient_terms[2].append(-z * density)
  mean_p = math.fsum(probabilities) / 200
  probability(fit["mean_uncensored_probability"], mean_p)
  assert len(fit["mean_probability_gradient"]) == 3
  for actual, terms in zip(fit["mean_probability_gradient"], gradient_terms):
    close(actual, math.fsum(terms) / 200, "retained mean probability gradient")
  ame = effects["effects"][0]
  close(ame["value"], beta[1] * mean_p, "continuous observed-mean AME")
  gradient = [beta[1] * finite(g) + (mean_p if i == 1 else 0) for i, g in enumerate(fit["mean_probability_gradient"])]
  variance = math.fsum(gradient[i] * float(covariance[i][j]) * gradient[j] for i in range(3) for j in range(3))
  assert variance > 0
  close(float(ame["standard_error"])**2, variance, "full joint delta variance")

  values = {}
  for i, row in enumerate(fit["coefficients"], 1):
    values.update({("coefficient_" + field, i, 0): row[field] for field in FIELDS})
  values.update({("joint_covariance", i + 1, j + 1): covariance[i][j] for i in range(3) for j in range(3)})
  values.update({("mean_probability_gradient", i + 1, 0): g for i, g in enumerate(fit["mean_probability_gradient"])})
  values.update({("ame_" + field, 1, 0): ame[field] for field in FIELDS})
  summary = {"n": 200, "censored_count": 70, "uncensored_count": 130, "lower_limit": 0,
    "log_likelihood": fit["log_likelihood"], "null_log_likelihood": fit["null_log_likelihood"],
    "likelihood_ratio": lr, "likelihood_ratio_df": 1, "likelihood_ratio_p_value": fit["likelihood_ratio_p_value"],
    "sigma": sigma, "log_sigma": eta, "log_sigma_standard_error": eta_se,
    "sigma_standard_error": scale["standard_error"], "sigma_confidence_lower": scale["confidence_lower"],
    "sigma_confidence_upper": scale["confidence_upper"], "mean_uncensored_probability": fit["mean_uncensored_probability"]}
  values.update({(key, 0, 0): value for key, value in summary.items()})
  quantities = read_csv(work, "R_tobit_quantities.csv")
  expected = {(row["quantity"], int(row["i"]), int(row["j"])): finite(row["value"]) for row in quantities}
  certificates = {("scaled_score", 0, 0), ("scaled_step", 0, 0)}
  assert len(quantities) == len(expected) == 48 and set(expected) == set(values) | certificates, "reference quantity inventory"
  for key, value in values.items():
    (probability if key[0] == "mean_uncensored_probability" else lambda x, y: close(x, y, key[0]))(value, expected[key])
  for key in certificates:
    assert 0 <= expected[key] <= 1e-10
  print("Validated48 R reference gates (46 comparisons,2 independent solver certificates),9 joint covariance entries,600 source-bound predictions and1400 original cells/ordinals; full-joint delta and normal/log-Wald inference self-checks passed.")


if __name__ == "__main__":
  audit(pathlib.Path(sys.argv[1]))
