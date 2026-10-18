"""Development-only Figure14.1 SVG data/geometry check against the R predictions.

python3 check_figure.py R_predictions.csv fishing_binary_comparison.svg native.jsonl
Run check_comparison.py as well to validate the complete sample/design/fit export.
No Python, R, XML library or browser runtime is required by TabDat.
"""

import json
import math
import re
import sys
import xml.etree.ElementTree as ET

# Keep development checks from leaving bytecode beside the repository sources.
sys.dont_write_bytecode = True
from check_comparison import close, read_csv


def pixel(actual, expected):
  # Absolute screen-coordinate tolerance, not an estimator/reference tolerance.
  assert math.isfinite(float(actual)) and abs(float(actual)-expected) < 1e-8, (actual, expected)


def check(reference, svg, native):
  rows = read_csv(reference)
  assert len(rows) == 630
  assert sum(int(row["charter"]) for row in rows) == 452
  ids = [int(row["source_row"]) for row in rows]
  assert ids == sorted(set(ids))
  assert all(int(row["mode"]) in (2, 4) and int(row["charter"]) == (int(row["mode"]) == 4) for row in rows)
  x = [float(row["lnrelp"]) for row in rows]
  y = [float(row["charter"]) for row in rows]
  # Independently written equivalent ordinal convention; not an R RNG reference.
  display_y = [value + 0.03*(2*((i*73) % 251)/250-1) for i, value in enumerate(y)]
  variables = ["logit_classical_pr", "probit_classical_pr", "ols_classical_xb"]
  predictions = [[float(row[variable]) for row in rows] for variable in variables]
  assert all(0 <= v <= 1 for series in predictions[:2] for v in series)
  assert min(predictions[2]) < 0 and max(predictions[2]) > 1
  values = y + display_y + [v for series in predictions for v in series]
  xmin, xmax, ymin, ymax = min(x), max(x), min(values), max(values)
  xspan, yspan = xmax-xmin, ymax-ymin
  xmin, xmax = xmin-0.05*xspan, xmax+0.05*xspan
  ymin, ymax = ymin-0.05*yspan, ymax+0.05*yspan
  root = ET.parse(svg).getroot()
  ns = "{http://www.w3.org/2000/svg}"
  assert root.tag == ns+"svg"
  assert root.attrib["width"] == "900" and root.attrib["height"] == "660"
  assert root.attrib["role"] == "img" and root.attrib["aria-labelledby"] == "plot-title plot-description"
  assert root.attrib["data-candidate-rows"] == "630"
  assert root.attrib["data-coordinate-precision"] == "float64"
  assert float(root.attrib["data-jitter-y"]) == 0.03
  for key, expected in [("x-min", xmin), ("x-max", xmax), ("y-min", ymin), ("y-max", ymax)]:
    close(root.attrib["data-"+key], expected)
  for key, expected in [("left", 90), ("top", 75), ("width", 730), ("height", 400)]:
    assert float(root.attrib["data-plot-"+key]) == expected
  def position(xvalue, yvalue):
    return 90+730*(xvalue-xmin)/(xmax-xmin), 75+400*(ymax-yvalue)/(ymax-ymin)
  allowed = {ns+tag for tag in ("svg", "title", "desc", "rect", "g", "text", "path", "circle")}
  for element in root.iter():
    assert element.tag in allowed
    assert all(not key.lower().startswith("on") and "href" not in key.lower() for key in element.attrib)
    assert not any("url(" in value.lower() for value in element.attrib.values())
  title = root.find(ns+"title")
  assert title is not None and title.text == "Predicted probabilities and OLS fitted values"
  point_marks = [e for e in root.iter(ns+"circle") if e.attrib.get("class") == "scatter-point"]
  assert len(point_marks) == 630
  assert [int(e.attrib["data-row-index"]) for e in point_marks] == list(range(630))
  for i, point in enumerate(point_marks):
    a = point.attrib
    assert a["data-x-variable"] == "lnrelp" and a["data-y-variable"] == "charter"
    close(a["data-x"], x[i], design=True)
    assert float(a["data-y"]) == y[i]
    close(a["data-display-y"], display_y[i], design=True)
    expected_x, expected_y = position(x[i], display_y[i])
    pixel(a["cx"], expected_x)
    pixel(a["cy"], expected_y)
  line_marks = [e for e in root.iter(ns+"path") if e.attrib.get("class") == "line-series"]
  assert [e.attrib["data-variable"] for e in line_marks] == variables
  assert [e.attrib["stroke"] for e in line_marks] == ["#0072B2", "#D55E00", "#009E73"]
  assert [e.attrib["stroke-dasharray"] for e in line_marks] == ["none", "8 4", "3 3"]
  order = sorted(range(630), key=lambda i: (x[i], i))
  for line, predicted in zip(line_marks, predictions):
    a = line.attrib
    assert a["fill"] == "none" and a["stroke-width"] == "2"
    indices = list(map(int, a["data-row-indices"].split(",")))
    assert indices == order
    xs, ys = (list(map(float, a[key].split(","))) for key in ("data-x-values", "data-y-values"))
    assert len(xs) == len(ys) == 630
    tokens = re.findall(r"([ML])([^ML\s]+)\s+([^ML\s]+)", a["d"])
    assert len(tokens) == 630
    assert "".join(mode+px+" "+py for mode, px, py in tokens) == a["d"]
    for j, (i, (mode, actual_x, actual_y)) in enumerate(zip(indices, tokens)):
      assert mode == ("M" if j == 0 else "L")
      close(xs[j], x[i], design=True)
      close(ys[j], predicted[i])
      expected_x, expected_y = position(x[i], predicted[i])
      pixel(actual_x, expected_x)
      pixel(actual_y, expected_y)
  with open(native, encoding="utf-8") as source:
    envelopes = [json.loads(line) for line in source]
  plots = [e for e in envelopes if e["result_type"] == "PlotArtifactResult"]
  assert len(plots) == 1 and plots[0]["schema_version"] == 1
  artifact = plots[0]["data"]
  assert artifact == {
    "path": "fishing_binary_comparison.svg", "format": "svg", "width": 900, "height": 660,
    "candidate_rows": 630, "x_variable": "lnrelp",
    "scatter": {"variable": "charter", "point_count": 630, "omitted_rows": 0},
    "lines": [{"variable": v, "point_count": 630, "omitted_rows": 0} for v in variables],
    "jitter_y": 0.03, "jitter_convention": "deterministic_row_index_pattern_outcome_units",
    "coordinate_precision": "float64",
  }
  print("Matched all 630 outcomes and 1,890 R prediction vertices, stable x/row order and actual SVG geometry (1e-8 pixel absolute tolerance).")
  print(f"OLS range [{min(predictions[2]):.9f}, {max(predictions[2]):.9f}] remains visible; deterministic display-only y jitter is not Stata RNG/pixel parity or anonymization.")


if __name__ == "__main__":
  assert len(sys.argv) == 4, __doc__
  check(*sys.argv[1:])
