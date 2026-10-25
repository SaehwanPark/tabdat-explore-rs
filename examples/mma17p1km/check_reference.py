#!/usr/bin/env python3
"""Audit native source identity, independent R output and actual step SVG geometry."""
import csv
import math
from pathlib import Path
import re
import sys
import xml.etree.ElementTree as ET


def rows(path):
  with open(path, newline="") as stream:
    return list(csv.DictReader(stream))


def number(value):
  return None if value == "" else float(value)


def close(actual, expected):
  if actual is None or expected is None:
    assert actual is expected, (actual, expected)
  else:
    assert math.isfinite(actual) and math.isfinite(expected)
    assert abs(actual - expected) <= 1e-11 + 1e-10 * abs(expected), (actual, expected)


def check(directory, input_csv, reference_csv):
  directory = Path(directory)
  original = rows(input_csv)
  source = rows(directory / "strikes_source.csv")
  assert original and len(source) == len(original)
  assert list(original[0]) == list(source[0]) == ["source_row", "dur", "gdp"]
  for a, b in zip(source, original):
    for name in a:
      close(number(a[name]), number(b[name]))
  durations = [float(row["dur"]) for row in original]
  assert all(math.isfinite(t) and t > 0 for t in durations)
  table = rows(directory / "strikes_survival.csv")
  reference = rows(reference_csv)
  assert len(table) == len(reference) == len(set(durations))
  assert list(table[0]) == list(reference[0])
  checks = 0
  hazard = variance = greenwood = 0.0
  for a, b, time in zip(table, reference, sorted(set(durations))):
    for name in a:
      close(number(a[name]), number(b[name]))
      checks += 1
    risk = sum(t >= time for t in durations)
    events = durations.count(time)
    survival = sum(t > time for t in durations) / len(durations)
    hazard += events / risk
    variance += events / risk**2
    expected = dict(time=time, at_risk=risk, events=events, survival=survival,
      cumhaz=hazard, cumhaz_se=math.sqrt(variance))
    if survival > 0:
      greenwood += events / (risk * (risk - events))
      expected["survival_se"] = survival * math.sqrt(greenwood)
      z = 1.959963984540054 * math.sqrt(greenwood) / abs(math.log(survival))
      expected["survival_lower"] = math.exp(-math.exp(math.log(-math.log(survival)) + z))
      expected["survival_upper"] = math.exp(-math.exp(math.log(-math.log(survival)) - z))
    else:
      expected.update(survival_se=None, survival_lower=None, survival_upper=None)
    for name, value in expected.items():
      close(number(a[name]), value)
  return checks, check_steps_and_svg(directory, reference)


def check_steps_and_svg(directory, reference, prefix="strikes", hazard_bands=False):
  """Audit the shared right-continuous step representation, including actual pixels."""
  directory = Path(directory)
  # Independently construct the staircase from the reference, not native table.
  previous = dict(time="0", survival="1", survival_lower="1", survival_upper="1", cumhaz="0")
  if hazard_bands:
    previous.update(cumhaz_lower="0", cumhaz_upper="0")
  expected_steps = [dict(previous, phase="0")]
  for row in reference:
    expected_steps.append(dict(previous, time=row["time"], phase="0"))
    previous = {name: row[name] for name in previous}
    expected_steps.append(dict(previous, phase="1"))
  steps = rows(directory / f"{prefix}_steps.csv")
  assert len(steps) == len(expected_steps)
  for a, b in zip(steps, expected_steps):
    assert set(a) == set(b)
    for name in a:
      close(number(a[name]), number(b[name]))
  vertices = 0
  hazard_variables = ["cumhaz", "cumhaz_lower", "cumhaz_upper"] if hazard_bands else ["cumhaz"]
  for filename, variables in [(f"{prefix}_survival.svg", ["survival", "survival_lower", "survival_upper"]),
      (f"{prefix}_cumhaz.svg", hazard_variables)]:
    svg = ET.parse(directory / filename).getroot()
    attrs = svg.attrib
    assert int(attrs["data-candidate-rows"]) == len(steps)
    lines = [node for node in svg.iter() if node.attrib.get("class") == "line-series"]
    assert [node.attrib["data-variable"] for node in lines] == variables
    def pixel(x, y):
      xmin, xmax = float(attrs["data-x-min"]), float(attrs["data-x-max"])
      ymin, ymax = float(attrs["data-y-min"]), float(attrs["data-y-max"])
      return (float(attrs["data-plot-left"]) + (x-xmin)/(xmax-xmin)*float(attrs["data-plot-width"]),
        float(attrs["data-plot-top"]) + (ymax-y)/(ymax-ymin)*float(attrs["data-plot-height"]))
    for node in lines:
      name = node.attrib["data-variable"]
      indices = [int(i) for i in node.attrib["data-row-indices"].split(",")]
      # NULL ordinates break the path and have no serialized vertex.
      xs = node.attrib["data-x-values"].split(",")
      ys = node.attrib["data-y-values"].split(",")
      valid_indices = [i for i, step in enumerate(expected_steps) if number(step[name]) is not None]
      assert indices == valid_indices
      assert len(xs) == len(ys) == len(valid_indices)
      expected_pixels = []
      for j, i in enumerate(valid_indices):
        step = expected_steps[i]
        x, y = number(step["time"]), number(step[name])
        close(number(xs[j]), x)
        close(number(ys[j]), y)
        expected_pixels.append(pixel(x, y))
      actual_pixels = re.findall(r"[ML]([\d.eE+\-]+)[ ,]([\d.eE+\-]+)", node.attrib["d"])
      assert len(actual_pixels) == len(expected_pixels)
      for actual, expected in zip(actual_pixels, expected_pixels):
        assert all(abs(float(a)-b) < .001 for a, b in zip(actual, expected)), (actual, expected)
      vertices += len(expected_pixels)
  return vertices


if __name__ == "__main__":
  assert len(sys.argv) == 4, "usage: check_reference.py output_directory strikes.csv reference.csv"
  checks, vertices = check(*sys.argv[1:])
  print(f"Validated {checks} R table cells, source identity, all step records and {vertices} SVG vertices")
