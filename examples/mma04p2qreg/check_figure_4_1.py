"""Audit native Figure 4.1 results and SVG geometry against independent R output."""
import csv
import json
import math
import re
import sys
import xml.etree.ElementTree as ET

jsonl, reference_csv, svg_path = sys.argv[1:]
events = [json.loads(line) for line in open(jsonl, encoding="utf-8") if line.strip()]
matches = [event for event in events if event.get("result_type") == "QuantileSlopePlotResult"]
assert len(matches) == 1, len(matches)
result = matches[0]["data"]
analysis = result["analysis"]
assert analysis["estimator"] == "iid_pairs_bootstrap_quantile_slope_bands"
assert analysis["observation_count"] == 5006
assert analysis["sample"] == {"total_observations": 5999, "dropped_observations": 993}
assert analysis["quantiles"] == [i / 20 for i in range(1, 20)]
assert analysis["replications"] == 200
assert analysis["seed"] == "10101"
assert analysis["covariance_convention"] == "common_iid_pairs_b_minus_1"
assert analysis["band_method"] == "pointwise_normal_fixed_1.96"
assert analysis["reference_distribution"] == "asymptotic_standard_normal"
assert analysis["critical_value"] == 1.96
assert analysis["simultaneous_coverage"] is False

with open(reference_csv, newline="", encoding="utf-8") as file:
  expected = list(csv.DictReader(file))
assert len(expected) == 19
assert len(analysis["points"]) == 19

def close(actual, wanted, label):
  actual, wanted = float(actual), float(wanted)
  assert math.isfinite(actual) and abs(actual - wanted) <= 1e-8, (label, actual, wanted)

for index, (actual, reference) in enumerate(zip(analysis["points"], expected)):
  for key in ("quantile", "estimate", "standard_error", "lower", "upper"):
    close(actual[key], reference[key], f"point[{index}].{key}")
  close(analysis["ols_coefficient"], reference["ols_coefficient"], "OLS slope")
  close(actual["lower"], float(reference["estimate"]) - 1.96 * float(reference["standard_error"]), "lower formula")
  close(actual["upper"], float(reference["estimate"]) + 1.96 * float(reference["standard_error"]), "upper formula")

namespace = "{http://www.w3.org/2000/svg}"
root = ET.parse(svg_path).getroot()
assert root.attrib["width"] == "900" and root.attrib["height"] == "660"
assert root.attrib["data-point-count"] == "19"
assert root.attrib["data-band-method"] == "pointwise_normal_fixed_1.96"
assert root.attrib["data-simultaneous-coverage"] == "false"
assert root.find(namespace + "title").text == "Slope Estimates as Quantile Varies"
texts = [element.text for element in root.iter(namespace + "text")]
for label in ["Quantile", "Slope and confidence bands", "Upper 95% confidence band", "Quantile slope coefficient", "Lower 95% confidence band", "OLS slope coefficient"]:
  assert label in texts, label

xs = [float(row["quantile"]) for row in expected]
ols = float(analysis["ols_coefficient"])
ys = [value for row in expected for value in (float(row["lower"]), float(row["estimate"]), float(row["upper"]))] + [ols]
x_min, x_max = min(xs), max(xs)
y_min, y_max = min(ys), max(ys)
x_padding = .05 * (x_max - x_min)
y_padding = .05 * (y_max - y_min)
x_min -= x_padding; x_max += x_padding
y_min -= y_padding; y_max += y_padding
for name, wanted in [("x-min", x_min), ("x-max", x_max), ("y-min", y_min), ("y-max", y_max)]:
  close(root.attrib["data-" + name], wanted, name)

def position(x, y):
  return 90 + 730 * (x - x_min) / (x_max - x_min), 75 + 400 * (y_max - y) / (y_max - y_min)

series = [element for element in root.iter(namespace + "path") if element.attrib.get("class") == "quantile-slope-series"]
assert [element.attrib["data-series"] for element in series] == ["upper_95_band", "quantile_slope", "lower_95_band", "ols_slope"]
series_values = {
  "upper_95_band": [float(row["upper"]) for row in expected],
  "quantile_slope": [float(row["estimate"]) for row in expected],
  "lower_95_band": [float(row["lower"]) for row in expected],
  "ols_slope": [ols, ols],
}
series_quantiles = {name: xs for name in ("upper_95_band", "quantile_slope", "lower_95_band")}
series_quantiles["ols_slope"] = [xs[0], xs[-1]]
for path in series:
  name = path.attrib["data-series"]
  q_values = [float(value) for value in path.attrib["data-quantile-values"].split(",")]
  b_values = [float(value) for value in path.attrib["data-coefficient-values"].split(",")]
  assert q_values == series_quantiles[name]
  assert len(b_values) == len(series_values[name])
  for index, (actual, wanted) in enumerate(zip(b_values, series_values[name])):
    close(actual, wanted, f"{name}[{index}] raw coordinate")
  tokens = re.findall(r"([ML])([^ML\s]+)\s+([^ML\s]+)", path.attrib["d"])
  assert len(tokens) == len(q_values) == len(b_values)
  for index, (mode, px, py) in enumerate(tokens):
    assert mode == ("M" if index == 0 else "L")
    ex, ey = position(q_values[index], b_values[index])
    close(px, ex, f"{name}[{index}] SVG x")
    close(py, ey, f"{name}[{index}] SVG y")

print("PASS: 19 R quantile slopes, B-1 SEs, fixed-1.96 pointwise endpoints, same-sample OLS, sample counts, series order, SVG bounds and all rendered path vertices (1e-8 absolute)")
