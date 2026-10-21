#!/usr/bin/env python3
"""Development-only R/plm and source-identity audit for unbalanced RE-GLS."""
import csv
import hashlib
import json
import math
from pathlib import Path
import sys

json_dir, text_dir, reference, raw = map(Path, sys.argv[1:])
assert hashlib.sha256(raw.read_bytes()).hexdigest() == "2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77"


def records(path):
  with path.open(newline="") as stream:
    return list(csv.DictReader(stream))


def close(actual, expected, label, tail=False):
  a, e = float(actual), float(expected)
  tolerance = abs(e)*1e-8 if tail else 1e-9
  assert math.isfinite(a) and math.isfinite(e) and abs(a-e) <= tolerance, (label,a,e,tolerance)


events = [json.loads(line) for line in (json_dir / "random_effects_gls_unbalanced.jsonl").read_text().splitlines()]
assert all(event["schema_version"] == 1 and "error" not in event for event in events)
fits = [event["data"] for event in events if event["result_type"] == "UnbalancedRandomEffectsGlsResult"]
assert len(fits) == 3
classical = [fit for fit in fits if fit["covariance"] == "nonrobust"]
clustered = [fit for fit in fits if fit["covariance"] == "cluster(id)"]
assert len(classical) == 1 and len(clustered) == 2 and clustered[0] == clustered[1]

inputs = records(reference / "inputs.csv")
assert len(inputs) == 4738 and all(row["case"] == "mom_sensitivity" for row in inputs)
assert len({row["source_row"] for row in inputs}) == len(inputs)
minimum = min(sum(row["id"] == unit for row in inputs) for unit in {row["id"] for row in inputs})
maximum = max(sum(row["id"] == unit for row in inputs) for unit in {row["id"] for row in inputs})
assert (minimum, maximum) == (5, 10)
scalars = [row for row in records(reference / "reference.csv") if row["case"] == "mom_sensitivity"]
assert len(scalars) == 9515
groups = {}
for row in inputs:
  groups.setdefault(row["id"], []).append(row)
beta0, beta1 = (float(coef["value"]) for coef in classical[0]["coefficients"])
sigma_e = float(classical[0]["sigma_e_squared"])
sigma_u = float(classical[0]["sigma_u_squared"])
transformed_fitted = []
transformed_residual = []
for row in inputs:
  unit = groups[row["id"]]
  size = len(unit)
  mean_y = sum(float(member["y"]) for member in unit)/size
  mean_x = sum(float(member["x"]) for member in unit)/size
  theta = 1-math.sqrt(sigma_e/(sigma_e+size*sigma_u))
  y_star = float(row["y"])-theta*mean_y
  fitted_star = beta0*(1-theta)+beta1*(float(row["x"])-theta*mean_x)
  transformed_fitted.append(fitted_star)
  transformed_residual.append(y_star-fitted_star)
compared = 0
for row in scalars:
  path = row["path"]
  fit = classical[0]
  if path.startswith("/cluster_covariance/"):
    fit, path = clustered[0], "/covariance_matrix/" + path.removeprefix("/cluster_covariance/")
  elif path.startswith("/cluster_coefficients/"):
    fit, path = clustered[0], "/coefficients/" + path.removeprefix("/cluster_coefficients/")
  elif path.startswith("/cluster_confidence_intervals/"):
    fit, path = clustered[0], "/coefficients/" + path.removeprefix("/cluster_confidence_intervals/")
    index, side = path.removeprefix("/coefficients/").split("/")
    path = f"/coefficients/{index}/{'ci_lower' if side == '0' else 'ci_upper'}"
  elif path in ("/cluster_wald_statistic", "/cluster_wald_p_value"):
    fit, path = clustered[0], path.removeprefix("/cluster_")
  elif path.startswith("/confidence_intervals/"):
    index, side = path.removeprefix("/confidence_intervals/").split("/")
    path = f"/coefficients/{index}/{'ci_lower' if side == '0' else 'ci_upper'}"
  elif path.startswith("/gls/coefficients/"):
    path = path.replace("/gls/coefficients/", "/coefficients/")
  elif path.startswith("/gls/covariance/matrix/"):
    path = path.replace("/gls/covariance/matrix/", "/covariance_matrix/")
  elif path.startswith("/gls/fitted_values/"):
    index = int(path.rsplit("/",1)[1])
    close(transformed_fitted[index],row["value"],row)
    compared += 1
    continue
  elif path.startswith("/gls/residuals/"):
    index = int(path.rsplit("/",1)[1])
    close(transformed_residual[index],row["value"],row)
    compared += 1
    continue
  elif path == "/group_size":
    assert classical[0]["group_size"] is None
    compared += 1
    continue
  elif path == "/theta":
    path = "/theta_at_minimum_group_size"
  else:
    path = path.replace("/cluster_wald_", "/wald_")
  value = fit
  for part in path.strip("/").split("/"):
    value = value[int(part)] if isinstance(value, list) else value[part]
  close(value, row["value"], row, tail=row["path"].endswith("p_value"))
  compared += 1

for fit in fits:
  assert fit["sample"] == {"total_observations":4738,"dropped_observations":0}
  assert fit["estimator"] == "unbalanced_swamy_arora_random_effects_gls"
  assert fit["group_count"] == 532 and fit["group_size"] is None
  assert (fit["minimum_group_size"],fit["maximum_group_size"]) == (5,10)
  assert fit["theta"] is None
  assert fit["reference_distribution"] == "normal" and fit["wald_reference"] == "chi_square"
  assert fit["variance_component_method"] == "unbalanced_swamy_arora"
  assert fit["covariance_convention"] in ("quasi_demeaned_rss_over_n_minus_k","unit_grouped_quasi_demeaned_scores")
  theta_values = [1-math.sqrt(sigma_e/(sigma_e+len(unit)*sigma_u)) for unit in groups.values()]
  close(fit["theta_at_minimum_group_size"],min(theta_values),"theta at minimum count")
  close(fit["theta_min"],min(theta_values),"minimum theta")
  close(fit["theta_max"],max(theta_values),"maximum theta")
for fit in clustered:
  close(fit["cluster"]["finite_sample_multiplier"],532/531*4737/4736,"cluster correction")

native = records(json_dir / "panel_re_gls_unbalanced_predictions.csv")
text = records(text_dir / "panel_re_gls_unbalanced_predictions.csv")
assert (len(native),len(text)) == (4738,4738)
assert (json_dir / "panel_re_gls_unbalanced_predictions.csv").read_bytes() == (text_dir / "panel_re_gls_unbalanced_predictions.csv").read_bytes()
source_fields = {"source_row":"source_row","id":"id","time":"year","y":"lnhr","x":"lnwg","kids":"kids","ageh":"ageh","agesq":"agesq","disab":"disab"}
cells = 0
for actual,expected in zip(native,inputs):
  for source,target in source_fields.items():
    assert float(actual[target]) == float(expected[source]), (source,target,actual,expected)
    cells += 1
  xb = float(classical[0]["coefficients"][0]["value"]) + float(classical[0]["coefficients"][1]["value"])*float(expected["x"])
  residual = float(expected["y"]) - xb
  for column in ("gls_xb","cluster_xb","robust_xb"):
    close(actual[column],xb,(column,expected["source_row"]))
    cells += 1
  for column in ("gls_residual","cluster_residual","robust_residual"):
    close(actual[column],residual,(column,expected["source_row"]))
    cells += 1
assert compared == 9515
print(f"PASS: {compared} R/plm/manual scalar comparisons and {cells} source/prediction cells over 4,738 rows; JSON/text exports match byte-for-byte")
