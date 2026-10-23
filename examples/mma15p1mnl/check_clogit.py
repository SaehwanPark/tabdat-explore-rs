"""Development-only conditional-choice source, CLI, and R/survival audit."""
import csv
import hashlib
import json
import math
import pathlib
import sys

work = pathlib.Path(sys.argv[1])
assert hashlib.sha256((work / "Nldata.asc").read_bytes()).hexdigest() == (
  "1fb6f7d5abaf790d4df2c6a13a499ca42c0db4f950d941ba92565c30b4ab6c22"
)
records = [json.loads(line) for line in (work / "clogit.jsonl").read_text().splitlines()]
assert all(r.get("schema_version") == 1 and "error" not in r for r in records)
fit, = [r["data"] for r in records if r["result_type"] == "ConditionalLogitResult"]
assert fit["estimator"] == "clogit" and fit["predictors"] == ["p", "q"]
assert fit["group_variable"] == "id" and fit["alternative_variable"] == "alternative"
assert not fit["include_intercept"] and fit["independent_unit"] == "choice_set"
assert fit["alternative_levels"] == [1, 2, 3, 4]
assert fit["chosen_counts"] == [134, 178, 418, 452]
assert fit["candidate_choice_set_count"] == fit["choice_set_count"] == 1182
assert fit["excluded_choice_set_count"] == 0
assert fit["sample"]["retained_indices"] == list(range(4728))
assert fit["sample"]["total_observations"] == 4728 and fit["sample"]["dropped_observations"] == 0
assert fit["covariance"] == "model_based_joint_conditional_fisher_information"
assert fit["covariance_correction"] == 1.0
assert fit["reference_distribution"] == "asymptotic_standard_normal" and fit["confidence_level"] == 95.0
assert fit["null_model"] == "equal_probabilities_within_complete_choice_sets_all_coefficients_zero"
assert fit["likelihood_ratio_reference_distribution"] == "asymptotic_chi_square"
assert fit["converged"] and fit["max_abs_scaled_score"] <= 1e-10 and fit["max_abs_scaled_newton_step"] <= 1e-10
assert len([r for r in records if r["result_type"] == "PredictionResult"]) == 9

def read_csv(name):
  with (work / name).open() as source:
    return list(csv.DictReader(source))

contrasts = read_csv("fishing_clogit_contrasts.csv")
assert [int(r["alternative"]) for r in contrasts] == [1, 2, 3, 4]
maxima = {}
count = 0
for row in read_csv("R_clogit_quantities.csv"):
  key, i, j = row["quantity"], int(row["i"]), int(row["j"])
  if key.startswith("coefficient_"):
    actual = fit["coefficients"][i][key.removeprefix("coefficient_")]
  elif key == "covariance":
    actual = fit["covariance_matrix"][i][j]
  elif key in {"price_contrast", "catch_contrast"}:
    actual = float(contrasts[i][key.split("_")[0] + str(j+1)])
  elif key == "mean_probability":
    actual = float(contrasts[i][key])
  else:
    actual = fit[key]
  expected = float(row["value"])
  difference = abs(actual - expected)
  assert math.isfinite(actual) and difference < 1e-9, (key, i, j, actual, expected)
  maxima[key] = max(maxima.get(key, 0), difference)
  count += 1
assert count == 61
native = read_csv("fishing_clogit_predictions.csv")
reference = read_csv("R_clogit_predictions.csv")
assert len(native) == len(reference) == 4728
assert list(native[0]) == list(reference[0])
original = (work / "Nldata.asc").read_text().split()
assert len(original) == 1182 * 16
probability_names = ["pr"] + [f"p_{kind}{j}" for kind in ("price", "catch") for j in range(1, 5)]
max_probability = 0
for index, (a, b) in enumerate(zip(native, reference)):
  person, alternative = index // 4 + 1, index % 4 + 1
  tokens = original[(person-1)*16:person*16]
  assert int(a["id"]) == int(b["id"]) == person
  assert int(a["alternative"]) == int(b["alternative"]) == alternative
  # Bind both exports directly to the hashed original tokens, not each other.
  # Identically corrupted native/R preparations must fail this audit too.
  for data in (a, b):
    assert float(data["chosen"]) == float(tokens[2+alternative]) == float(float(tokens[0]) == alternative)
    assert float(data["p"]) == float(data["pbase"]) == float(tokens[6+alternative])
    assert float(data["q"]) == float(data["qbase"]) == float(tokens[10+alternative])
  for name in probability_names:
    actual, expected = float(a[name]), float(b[name])
    max_probability = max(max_probability, abs(actual - expected))
    assert math.isfinite(actual) and abs(actual-expected) < 1e-10, (index, name, actual, expected)
for start in range(0, 4728, 4):
  for name in probability_names:
    assert abs(sum(float(r[name]) for r in native[start:start+4])-1) < 1e-14
for kind in ("price", "catch"):
  for j in range(1, 5):
    assert abs(sum(float(r[f"{kind}{j}"]) for r in contrasts)) < 1e-11
# Historical printed coefficient/SE digits are deliberately separate from the
# tight native/R check; finite-change contrasts are float64, not Stata8 storage.
for coefficient, value, se in zip(fit["coefficients"], [-.0204765, .9530985], [.0012231, .0894134]):
  assert abs(coefficient["value"]-value) < 1e-6
  assert abs(coefficient["standard_error"]-se) < 1e-6
assert abs(fit["log_likelihood"]-(-1311.9796)) < .00005
print(f"Validated61 R scalars,4728 source-bound choice rows and42552 probabilities; max probability error={max_probability:.3g}")
print(maxima)
