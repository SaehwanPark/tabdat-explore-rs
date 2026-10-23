"""Development-only source-bound non-normalized nested-choice/R audit."""
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
records = [json.loads(line) for line in (work / "nested.jsonl").read_text().splitlines()]
assert all(r.get("schema_version") == 1 and "error" not in r for r in records)
restricted, = [r["data"] for r in records if r["result_type"] == "ConditionalLogitResult"]
nested, = [r["data"] for r in records if r["result_type"] == "NonNormalizedNestedLogitResult"]
fits = {"restricted": restricted, "nested": nested}
names = ["p", "q", "dshore", "dshorey"]
for model, fit in fits.items():
  assert fit["predictors"] == names == [c["name"] for c in fit["coefficients"]]
  assert fit["alternative_levels"] == [1, 2, 3, 4] and fit["chosen_counts"] == [134, 178, 418, 452]
  assert fit["group_variable"] == "id" and fit["alternative_variable"] == "alternative"
  assert not fit["include_intercept"] and fit["independent_unit"] == "choice_set"
  assert fit["observation_count"] == 4728 and fit["choice_set_size"] == 4
  assert fit["choice_set_count"] == fit["candidate_choice_set_count"] == 1182
  assert fit["excluded_choice_set_count"] == 0
  assert fit["sample"]["retained_indices"] == list(range(4728))
  assert fit["sample"]["total_observations"] == 4728 and fit["sample"]["dropped_observations"] == 0
  assert fit["covariance_correction"] == 1.0 and fit["confidence_level"] == 95.0
  assert fit["reference_distribution"] == "asymptotic_standard_normal"
  assert fit["converged"] and fit["max_abs_scaled_score"] <= 1e-10 and fit["max_abs_scaled_newton_step"] <= 1e-10
assert restricted["covariance"] == "model_based_joint_conditional_fisher_information"
assert restricted["covariance_parameter_names"] == names
assert restricted["null_model"] == "equal_probabilities_within_complete_choice_sets_all_coefficients_zero"
assert restricted["likelihood_ratio_df"] == 4
assert nested["estimator"] == "nlogit" and nested["parameterization"] == "non_normalized_conditional_utilities"
assert nested["preferred_rum_normalization"] is False and nested["nest_variable"] == "nest"
assert nested["lower_predictor_count"] == 2 and nested["parameter_count"] == 6
assert nested["nest_levels"] == [1, 2] and nested["alternative_nests"] == [1, 1, 2, 2]
assert nested["covariance"] == "model_based_joint_observed_nested_choice_hessian"
assert nested["covariance_parameter_names"] == names + ["iv[1]", "iv[2]"]
assert nested["coefficient_test_null"] == 0.0
assert [d["nest_code"] for d in nested["dissimilarities"]] == [1, 2]
assert all(d["test_null"] == 1.0 and d["estimate"]["value"] > 1 for d in nested["dissimilarities"])
assert nested["random_utility_iv_bounds_satisfied"] is False
assert nested["independence_null"] == "both_positive_nest_scales_equal_one_slopes_reestimated"
assert nested["independence_reference_distribution"] == "asymptotic_chi_square"
assert "null_log_likelihood" not in nested and "likelihood_ratio" not in nested
assert abs(nested["restricted_log_likelihood"] - restricted["log_likelihood"]) < 1e-9

def read_csv(name):
  with (work / name).open() as source:
    return list(csv.DictReader(source))

def close(actual, expected, key, model):
  assert math.isfinite(actual) and math.isfinite(expected), (model, key, actual, expected)
  tolerance = 1e-9 + (1e-8*abs(expected) if model == "nested" else 0)
  assert abs(actual-expected) < tolerance, (model, key, actual, expected)
  if key.endswith("p_value") and expected > 0:
    # This new nested-workflow reference (including its four-slope restriction)
    # uses the predeclared1e-8 tail contract. Standalone CL/MNL gates stay unchanged.
    # The restriction's ~1e-11 native slope stopping error amplifies to1.82e-9
    # relative error at p=4.42e-56; native tails agree with math.erfc at their z.
    assert actual > 0 and abs(actual-expected) < 1e-8*expected, (model, key, actual, expected)

contrasts = read_csv("fishing_nested_contrasts.csv")
assert [int(r["alternative"]) for r in contrasts] == [1, 2, 3, 4]
preview, = [r["data"] for r in records if r["result_type"] == "PreviewResult"]
assert preview["columns"] == ["alternative", "mean_probability", "income1000", "charter_price1", "charter_catch001"]
assert len(preview["rows"]) == 4
for displayed, row in zip(preview["rows"], contrasts):
  for name, value in zip(preview["columns"], displayed):
    close(value, float(row[name]), "displayed_"+name, "nested")
assert sum(r["result_type"] == "PredictionResult" for r in records) == 5
count, covariance_count = 0, 0
maxima = {}
for row in read_csv("R_nested_quantities.csv"):
  model, key, i, j = row["model"], row["quantity"], int(row["i"]), int(row["j"])
  fit = fits[model]
  parameters = fit["coefficients"] + ([d["estimate"] for d in fit["dissimilarities"]] if model == "nested" else [])
  if key == "parameter_test_null":
    actual = 0 if i <= 4 else fit["dissimilarities"][i-5]["test_null"]
  elif key == "parameter_count":
    actual = len(parameters)
  elif key.startswith("parameter_"):
    actual = parameters[i-1][key.removeprefix("parameter_")]
  elif key == "covariance":
    actual = fit["covariance_matrix"][i-1][j-1]
    covariance_count += 1
  elif key.endswith("_contrast") or key == "mean_probability":
    name = {"income_contrast": "income1000", "price_contrast": "charter_price1", "catch_contrast": "charter_catch001"}.get(key, key)
    actual = float(contrasts[i-1][name])
  else:
    actual = fit[key]
  expected = float(row["value"])
  close(actual, expected, key, model)
  maxima[(model, key)] = max(maxima.get((model, key), 0), abs(actual-expected))
  count += 1
assert count == 149 and covariance_count == 52
native = read_csv("fishing_nested_predictions.csv")
reference = read_csv("R_nested_predictions.csv")
assert len(native) == len(reference) == 4728
assert list(native[0]) == ["id", "alternative", "chosen", "p", "q", "ydiv1000", "nest", "dshore", "dshorey", "ybase", "pbase", "qbase", "restricted", "baseline", "income", "price", "catch"]
assert list(reference[0]) == ["id", "alternative", "nest", "chosen", "p", "q", "ydiv1000", "dshore", "dshorey", "restricted", "baseline", "income", "price", "catch"]
tokens = (work / "Nldata.asc").read_text().split()
assert len(tokens) == 1182*16
probabilities = ["restricted", "baseline", "income", "price", "catch"]
max_probability = 0
for index, (a, b) in enumerate(zip(native, reference)):
  person, alternative = index//4+1, index%4+1
  source = tokens[(person-1)*16:person*16]
  for data in (a, b):
    assert int(data["id"]) == person and int(data["alternative"]) == alternative
    assert int(data["nest"]) == (1 if alternative <= 2 else 2)
    assert float(data["chosen"]) == float(source[2+alternative]) == float(float(source[0]) == alternative)
    assert float(data["p"]) == float(source[6+alternative])
    assert float(data["q"]) == float(source[10+alternative])
    assert float(data["ydiv1000"]) == float(source[15])/1000
    assert float(data["dshore"]) == float(alternative <= 2)
    assert float(data["dshorey"]) == float(alternative <= 2)*(float(source[15])/1000)
  assert float(a["pbase"]) == float(a["p"]) and float(a["qbase"]) == float(a["q"])
  assert float(a["ybase"]) == float(a["ydiv1000"])
  for name in probabilities:
    actual, expected = float(a[name]), float(b[name])
    assert math.isfinite(actual) and 0 <= actual <= 1 and abs(actual-expected) < 1e-10, (index, name, actual, expected)
    max_probability = max(max_probability, abs(actual-expected))
for start in range(0, 4728, 4):
  for name in probabilities:
    assert abs(sum(float(r[name]) for r in native[start:start+4])-1) < 1e-14
for name in ["income1000", "charter_price1", "charter_catch001"]:
  assert abs(sum(float(r[name]) for r in contrasts)) < 1e-11
# The historical Stata8 values are NOT rounded versions of this stationary fit.
assert abs(nested["log_likelihood"]-(-1180.0156)) > .001
print(f"Validated149 R scalars including52 covariance entries,source-bound4728-row nested design and23640 probabilities; max p error={max_probability:.3g}")
print(maxima)
