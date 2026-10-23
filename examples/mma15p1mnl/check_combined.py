"""Development-only original-source/independent-R audit; no runtime dependency."""
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
records = [json.loads(line) for line in (work / "combined.jsonl").read_text().splitlines()]
assert all(r.get("schema_version") == 1 and "error" not in r for r in records)
encoded_names = ["dummy2", "dummy3", "dummy4", "d2y", "d3y", "d4y"]
combined_names = ["p", "q"] + encoded_names
conditional = [r["data"] for r in records if r["result_type"] == "ConditionalLogitResult"]
assert len(conditional) == 2
fits = dict(zip(["encoded", "combined"], conditional))
for model, names in [("encoded", encoded_names), ("combined", combined_names)]:
  fit = fits[model]
  assert fit["predictors"] == names == fit["covariance_parameter_names"]
  assert [c["name"] for c in fit["coefficients"]] == names
  assert fit["alternative_levels"] == [1, 2, 3, 4] and fit["chosen_counts"] == [134, 178, 418, 452]
  assert fit["group_variable"] == "id" and fit["alternative_variable"] == "alternative"
  assert not fit["include_intercept"] and fit["independent_unit"] == "choice_set"
  assert fit["choice_set_count"] == fit["candidate_choice_set_count"] == 1182
  assert fit["excluded_choice_set_count"] == 0
  assert fit["sample"]["retained_indices"] == list(range(4728))
  assert fit["sample"]["total_observations"] == 4728 and fit["sample"]["dropped_observations"] == 0
  assert fit["covariance"] == "model_based_joint_conditional_fisher_information" and fit["covariance_correction"] == 1.0
  assert fit["reference_distribution"] == "asymptotic_standard_normal" and fit["confidence_level"] == 95.0
  assert fit["null_model"] == "equal_probabilities_within_complete_choice_sets_all_coefficients_zero"
  assert fit["likelihood_ratio_df"] == len(names) and fit["likelihood_ratio_reference_distribution"] == "asymptotic_chi_square"
  assert fit["converged"] and fit["max_abs_scaled_score"] <= 1e-10 and fit["max_abs_scaled_newton_step"] <= 1e-10

def read_csv(name):
  with (work / name).open() as source:
    return list(csv.DictReader(source))

def close(actual, expected, key):
  assert math.isfinite(actual) and abs(actual-expected) < 1e-9, (key, actual, expected)
  if key.endswith("p_value") and expected > 0:
    assert abs(actual-expected) < 1e-9*expected, (key, actual, expected)

contrasts = read_csv("fishing_combined_contrasts.csv")
assert [int(r["alternative"]) for r in contrasts] == [1, 2, 3, 4]
maxima = {}
count = 0
for row in read_csv("R_combined_quantities.csv"):
  model, key, i, j = row["model"], row["quantity"], int(row["i"]), int(row["j"])
  fit = fits[model]
  if key.startswith("coefficient_"):
    actual = fit["coefficients"][i][key.removeprefix("coefficient_")]
  elif key == "covariance":
    actual = fit["covariance_matrix"][i][j]
  elif key.endswith("_contrast") or key == "mean_probability":
    name = {"income_contrast": "income1000", "price_contrast": "charter_price1", "catch_contrast": "charter_catch001"}.get(key, key)
    actual = float(contrasts[i][name])
  else:
    actual = fit[key]
  expected = float(row["value"])
  close(actual, expected, key)
  maxima[(model, key)] = max(maxima.get((model, key), 0), abs(actual-expected))
  count += 1
assert count == 218
mnl, = [r["data"] for r in records if r["result_type"] == "MultinomialLogitResult"]
assert mnl["base_outcome"] == 1 and mnl["predictors"] == ["ydiv1000"]
assert mnl["category_levels"] == [1, 2, 3, 4] and mnl["category_counts"] == [134, 178, 418, 452]
assert mnl["sample"]["retained_indices"] == list(range(1182))
# Same full model/likelihood, DIFFERENT null: conditional tests all six utility
# columns zero, ordinary MNL retains estimated shares and tests three slopes.
assert mnl["null_model"] == "intercept_only_estimated_category_shares" and mnl["likelihood_ratio_df"] == 3
close(mnl["log_likelihood"], fits["encoded"]["log_likelihood"], "encoded_full_likelihood")
assert abs(mnl["null_log_likelihood"]-fits["encoded"]["null_log_likelihood"]) > 1
permutation = [0, 2, 4, 1, 3, 5]
for i, p in enumerate(permutation):
  for name in ["value", "standard_error", "statistic", "p_value", "confidence_lower", "confidence_upper"]:
    close(mnl["coefficients"][p][name], fits["encoded"]["coefficients"][i][name], "encoding_"+name)
  for j, q in enumerate(permutation):
    close(mnl["covariance_matrix"][p][q], fits["encoded"]["covariance_matrix"][i][j], "encoding_covariance")
native = read_csv("fishing_combined_predictions.csv")
reference = read_csv("R_combined_predictions.csv")
assert len(native) == len(reference) == 4728 and list(native[0]) == list(reference[0])
tokens = (work / "Nldata.asc").read_text().split()
assert len(tokens) == 1182*16
probabilities = ["mnl_probability", "encoded_mnl", "baseline", "income", "price", "catch"]
max_probability = 0
for index, (a, b) in enumerate(zip(native, reference)):
  person, alternative = index//4+1, index%4+1
  source = tokens[(person-1)*16:person*16]
  for data in (a, b):
    assert int(data["id"]) == person and int(data["alternative"]) == alternative
    assert float(data["chosen"]) == float(source[2+alternative]) == float(float(source[0]) == alternative)
    assert float(data["p"]) == float(data["pbase"]) == float(source[6+alternative])
    assert float(data["q"]) == float(data["qbase"]) == float(source[10+alternative])
    assert float(data["ydiv1000"]) == float(data["ybase"]) == float(source[15])/1000
    for j in range(2, 5):
      assert float(data[f"dummy{j}"]) == float(alternative == j)
      assert float(data[f"d{j}y"]) == float(alternative == j)*(float(source[15])/1000)
  for name in probabilities:
    actual, expected = float(a[name]), float(b[name])
    assert math.isfinite(actual) and 0 <= actual <= 1 and abs(actual-expected) < 1e-10, (index, name, actual, expected)
    max_probability = max(max_probability, abs(actual-expected))
  assert abs(float(a["mnl_probability"])-float(a["encoded_mnl"])) < 1e-10
for start in range(0, 4728, 4):
  for name in probabilities:
    assert abs(sum(float(r[name]) for r in native[start:start+4])-1) < 1e-14
for name in ["income1000", "charter_price1", "charter_catch001"]:
  assert abs(sum(float(r[name]) for r in contrasts)) < 1e-11
# Separate coarse historical check: the section4 eight-column fit, not its
# five-column alternative-constant/price/catch prelude or a random-coefficient fit.
for coefficient, value, se in zip(fits["combined"]["coefficients"],
    [-.0251166,.357782,.7779594,.5272788,1.694366,-.1275771,.0894398,-.0332917],
    [.0017317,.1097733,.2204939,.2227927,.2240506,.0506395,.0500671,.0503409]):
  assert abs(coefficient["value"]-value) < 1e-6
  assert abs(coefficient["standard_error"]-se) < 1e-6
assert abs(fits["combined"]["log_likelihood"]-(-1215.1376)) < .00005
print(f"Validated218 R scalars,source-bound4728-row eight-column design,full MNL encoding permutation and28368 probabilities; max p error={max_probability:.3g}")
print(maxima)
