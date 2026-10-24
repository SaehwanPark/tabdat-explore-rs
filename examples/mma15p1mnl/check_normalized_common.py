"""Development-only source-bound preferred common-scale nested-choice/R audit."""
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
records = [json.loads(line) for line in (work / "normalized_common.jsonl").read_text().splitlines()]
assert all(r.get("schema_version") == 1 and "error" not in r for r in records)
restricted, = [r["data"] for r in records if r["result_type"] == "ConditionalLogitResult"]
normalized, = [r["data"] for r in records if r["result_type"] == "NormalizedCommonNestedLogitResult"]
fits = {"restricted": restricted, "normalized_common": normalized}
names = ["p", "q", "dshore", "dshorey"]
for fit in fits.values():
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
assert restricted["covariance_parameter_names"] == names and restricted["likelihood_ratio_df"] == 4
assert restricted["null_model"] == "equal_probabilities_within_complete_choice_sets_all_coefficients_zero"
assert normalized["estimator"] == "nlogit" and normalized["parameterization"] == "normalized_common_utility"
assert normalized["preferred_rum_normalization"] is True and normalized["shared_nest_scale"] is True
assert normalized["nest_variable"] == "nest" and normalized["lower_predictor_count"] == 2
assert normalized["parameter_count"] == 5 and normalized["nest_levels"] == [1, 2]
assert normalized["alternative_nests"] == [1, 1, 2, 2]
assert normalized["covariance"] == "model_based_joint_observed_normalized_common_nested_choice_hessian"
assert normalized["covariance_parameter_names"] == names + ["iv_common"]
assert normalized["dissimilarity"]["name"] == "iv_common" and normalized["dissimilarity"]["value"] > 1
assert normalized["coefficient_test_null"] == 0.0 and normalized["scale_test_null"] == 1.0
assert normalized["random_utility_iv_bounds_satisfied"] is False
assert normalized["independence_null"] == "shared_positive_nest_scale_equal_one_slopes_reestimated"
assert normalized["independence_reference_distribution"] == "asymptotic_chi_square"
assert normalized["independence_df"] == 1
assert not {"null_log_likelihood", "likelihood_ratio", "pseudo_r_squared", "fitted_probabilities"} & normalized.keys()
assert abs(normalized["restricted_log_likelihood"] - restricted["log_likelihood"]) < 1e-9
# Bind reported positive tails to their own statistics too: mutually corrupted
# native/reference zero p-values must not pass an absolute near-zero gate.
for model, fit in fits.items():
  parameters = fit["coefficients"] + ([fit["dissimilarity"]] if model == "normalized_common" else [])
  for i, parameter in enumerate(parameters):
    null = 0 if i < 4 else 1
    assert parameter["standard_error"] > 0
    z = (parameter["value"]-null)/parameter["standard_error"]
    assert abs(z-parameter["statistic"]) < 1e-10
    tail = math.erfc(abs(z)/math.sqrt(2))
    assert tail > 0 and abs(parameter["p_value"]/tail-1) < 1e-10
lr_tail = math.erfc(math.sqrt(normalized["independence_likelihood_ratio"]/2))
assert lr_tail > 0 and abs(normalized["independence_p_value"]/lr_tail-1) < 1e-10

def read_csv(name):
  with (work / name).open() as source:
    return list(csv.DictReader(source))

def close(actual, expected, key, model):
  assert math.isfinite(actual) and math.isfinite(expected), (model, key, actual, expected)
  tolerance = 1e-9 + (1e-8*abs(expected) if model == "normalized_common" else 0)
  assert abs(actual-expected) < tolerance, (model, key, actual, expected)
  # New-workflow positive-tail gate; existing standalone CL/MNL/earlier gates stay unchanged.
  if key.endswith("p_value") and expected > 0:
    assert actual > 0 and abs(actual-expected) < 1e-8*expected, (model, key, actual, expected)

contrasts = read_csv("fishing_normalized_common_contrasts.csv")
assert [int(r["alternative"]) for r in contrasts] == [1, 2, 3, 4]
preview, = [r["data"] for r in records if r["result_type"] == "PreviewResult"]
assert preview["columns"] == ["alternative", "mean_probability", "income1000", "charter_price1", "charter_catch001"]
assert len(preview["rows"]) == 4
for displayed, row in zip(preview["rows"], contrasts):
  for name, value in zip(preview["columns"], displayed):
    close(value, float(row[name]), "displayed_"+name, "normalized_common")
assert sum(r["result_type"] == "PredictionResult" for r in records) == 5
quantities = read_csv("R_normalized_common_quantities.csv")
assert len(quantities) == len({(r["model"], r["quantity"], r["i"], r["j"]) for r in quantities}) == 131
covariance_count = 0
maxima = {}
for row in quantities:
  model, key, i, j = row["model"], row["quantity"], int(row["i"]), int(row["j"])
  fit = fits[model]
  parameters = fit["coefficients"] + ([fit["dissimilarity"]] if model == "normalized_common" else [])
  if key == "parameter_test_null":
    actual = 0 if i <= 4 else fit["scale_test_null"]
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
assert covariance_count == 41
native = read_csv("fishing_normalized_common_predictions.csv")
reference = read_csv("R_normalized_common_predictions.csv")
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
    assert abs(math.fsum(float(r[name]) for r in native[start:start+4])-1) < 1e-14
for alt, summary in enumerate(contrasts, 1):
  selected = native[alt-1::4]
  for name, values in [
    ("mean_probability", [float(r["baseline"]) for r in selected]),
    ("income1000", [float(r["income"])-float(r["baseline"]) for r in selected]),
    ("charter_price1", [100*(float(r["price"])-float(r["baseline"])) for r in selected]),
    ("charter_catch001", [1000*(float(r["catch"])-float(r["baseline"])) for r in selected]),
  ]:
    close(float(summary[name]), math.fsum(values)/1182, "exported_"+name, "normalized_common")
for name in ["income1000", "charter_price1", "charter_catch001"]:
  assert abs(math.fsum(float(r[name]) for r in contrasts)) < 1e-11
print(f"Validated131 R quantities including41 covariance entries,source-bound4728-row normalized common-scale design and23640 probabilities; max p error={max_probability:.3g}")
print(maxima)
