"""Development-only original-source/CLI audit against independent R/nnet evidence."""
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
records = [json.loads(line) for line in (work / "mlogit.jsonl").read_text().splitlines()]
assert all(row.get("schema_version") == 1 and "error" not in row for row in records), records
fit, = [row["data"] for row in records if row["result_type"] == "MultinomialLogitResult"]
effects, = [row["data"] for row in records if row["result_type"] == "MultinomialMarginalEffectsResult"]
assert fit["category_levels"] == effects["category_levels"] == [1, 2, 3, 4]
assert fit["category_counts"] == [134, 178, 418, 452]
assert fit["base_outcome"] == effects["base_outcome"] == 1
assert fit["sample"]["retained_indices"] == list(range(1182))
assert fit["sample"]["total_observations"] == 1182
assert fit["sample"]["dropped_observations"] == 0
assert fit["predictors"] == ["ydiv1000"] and fit["include_intercept"]
assert [(c["category"], c["name"]) for c in fit["coefficients"]] == [
  (category, name) for category in (2, 3, 4) for name in ("intercept", "ydiv1000")
]
assert [(c["category"], c["name"]) for c in effects["effects"]] == [
  (category, "ydiv1000") for category in (1, 2, 3, 4)
]
for result in (fit, effects):
  assert result["covariance"] == "model_based_joint_fisher_information"
  assert result["covariance_correction"] == 1.0
  assert result["reference_distribution"] == "asymptotic_standard_normal"
  assert result["confidence_level"] == 95.0
assert fit["converged"] and fit["max_abs_scaled_score"] <= 1e-10
assert fit["max_abs_scaled_newton_step"] <= 1e-10
assert fit["null_model"] == "intercept_only_estimated_category_shares"
assert fit["likelihood_ratio_reference_distribution"] == "asymptotic_chi_square"
assert effects["uncertainty"] == "conditional_on_observed_design_delta_method"
assert effects["sample"] == "retained_estimation_design"
count = 0
with (work / "R_quantities.csv").open() as source:
  for row in csv.DictReader(source):
    key, i, j = row["quantity"], int(row["i"]), int(row["j"])
    if key.startswith("coefficient_"):
      actual = fit["coefficients"][i][key.removeprefix("coefficient_")]
    elif key.startswith("effect_"):
      actual = effects["effects"][i][key.removeprefix("effect_")]
    elif key == "covariance":
      actual = fit["covariance_matrix"][i][j]
    elif key == "category_count":
      actual = fit["category_counts"][i]
    else:
      actual = fit[key]
    expected = float(row["value"])
    # Absolute 1e-9 covers rounded R CSV and independent numerical delta gradients;
    # observed maxima are reported in README, not inferred from this tolerance.
    assert math.isfinite(actual) and abs(actual - expected) < 1e-9, (key, i, j, actual, expected)
    count += 1
assert count == 107
with (work / "fishing_mnl_predictions.csv").open() as source:
  native = list(csv.DictReader(source))
with (work / "R_predictions.csv").open() as source:
  reference = list(csv.DictReader(source))
assert len(native) == len(reference) == 1182
assert list(native[0]) == list(reference[0])
source_names = list(native[0])[1:17]
assert source_names == ["mode", "price", "crate", "dbeach", "dpier", "dprivate", "dcharter",
  "pbeach", "ppier", "pprivate", "pcharter", "qbeach", "qpier", "qprivate", "qcharter", "income"]
original = (work / "Nldata.asc").read_text().split()
assert len(original) == 1182 * 16
source_cells = probability_cells = index_cells = 0
for index, (a, b) in enumerate(zip(native, reference), 1):
  # Bind both prepared exports to the hashed original token stream, not merely
  # to each other: identically altered native/R inputs must not pass this audit.
  tokens = original[(index-1)*16:index*16]
  for name, token in zip(source_names, tokens):
    assert float(a[name]) == float(token), (index, name, a[name], token)
  assert float(a["ydiv1000"]) == float(tokens[-1]) / 1000
  assert int(a["source_row"]) == int(b["source_row"]) == index
  assert float(a["mode"]) == float(b["mode"])
  for key in a:
    actual, expected = float(a[key]), float(b[key])
    tolerance = 2e-10 if key in {f"p{i}" for i in range(1, 5)} | {f"xb{i}" for i in range(1, 5)} else 1e-10
    assert math.isfinite(actual) and abs(actual - expected) < tolerance, (index, key, actual, expected)
    if key.startswith("xb"):
      index_cells += 1
    elif key in {"p1", "p2", "p3", "p4"}:
      probability_cells += 1
    else:
      source_cells += 1
  assert abs(sum(float(a[f"p{i}"]) for i in range(1, 5)) - 1) < 1e-14
  assert float(a["xb1"]) == 0
assert abs(sum(e["value"] for e in effects["effects"])) < 1e-14
# Author Table15.2 printed digits are a separate, coarser historical check.
for coefficient, value, se in zip(fit["coefficients"],
    [.8141503, -.1434029, .7389208, .0919064, 1.341291, -.0316399],
    [.2286316, .0532882, .1967309, .0406638, .1945167, .0418463]):
  assert abs(coefficient["value"] - value) < 1e-6
  assert abs(coefficient["standard_error"] - se) < 1e-6
assert abs(fit["log_likelihood"] - (-1477.1506)) < .00005
print(f"Validated source SHA256,1182 ordered people,{count} R scalar quantities,{source_cells} source/design cells,{probability_cells} probabilities,{index_cells} indices and historical Table15.2 comparison (coefficient/SE absolute tolerance1e-6).")
