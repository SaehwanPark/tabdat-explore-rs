#!/usr/bin/env python3
"""Audit exact samples, right-censoring risk sets, independent R and native plots."""
import math
from pathlib import Path
import sys

# Reuse the already-tested step/SVG audit; no application/Python runtime coupling.
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "mma17p1km"))
from check_reference import close, number, rows, check_steps_and_svg


def check(directory, input_csv, reference_csv):
  directory = Path(directory)
  original, source = rows(input_csv), rows(directory / "censored_source.csv")
  assert original and len(original) == len(source)
  assert list(original[0]) == list(source[0]) == ["source_row", "duration", "failed"]
  for a, b in zip(source, original):
    for name in a:
      close(number(a[name]), number(b[name]))
  durations = [float(row["duration"]) for row in original]
  flags = [float(row["failed"]) for row in original]
  assert all(math.isfinite(t) and t > 0 for t in durations)
  assert all(d in (0, 1) for d in flags) and sum(flags) > 0
  table, reference = rows(directory / "censored_survival.csv"), rows(reference_csv)
  assert len(table) == len(reference) == len(set(durations))
  assert list(table[0]) == list(reference[0])
  survival, greenwood, hazard, variance = 1.0, 0.0, 0.0, 0.0
  checks = 0
  for a, b, time in zip(table, reference, sorted(set(durations))):
    for name in a:
      close(number(a[name]), number(b[name]))
      checks += 1
    risk = sum(t >= time for t in durations)
    events = sum(d for t, d in zip(durations, flags) if t == time)
    censored = durations.count(time) - events
    survival *= 1 - events / risk
    hazard += events / risk
    variance += events / risk**2
    if risk > events:
      greenwood += events / (risk * (risk - events))
    expected = dict(time=time, at_risk=risk, events=events, censored=censored,
      survival=survival, cumhaz=hazard, cumhaz_se=math.sqrt(variance))
    if survival > 0:
      expected["survival_se"] = survival * math.sqrt(greenwood)
      if survival == 1:
        expected.update(survival_lower=1.0, survival_upper=1.0)
      else:
        z = 1.959963984540054 * math.sqrt(greenwood) / abs(math.log(survival))
        expected["survival_lower"] = math.exp(-math.exp(math.log(-math.log(survival)) + z))
        expected["survival_upper"] = math.exp(-math.exp(math.log(-math.log(survival)) - z))
    else:
      expected.update(survival_se=None, survival_lower=None, survival_upper=None)
    if hazard > 0:
      z = 1.959963984540054 * math.sqrt(variance) / hazard
      expected.update(cumhaz_lower=hazard*math.exp(-z), cumhaz_upper=hazard*math.exp(z))
    else:
      expected.update(cumhaz_lower=0.0, cumhaz_upper=0.0)
    for name, value in expected.items():
      close(number(a[name]), value)
  return checks, check_steps_and_svg(directory, reference, prefix="censored", hazard_bands=True)


if __name__ == "__main__":
  assert len(sys.argv) == 4, "usage: check_censored.py output_directory censored.csv reference.csv"
  checks, vertices = check(*sys.argv[1:])
  print(f"Validated {checks} R table cells, censoring risk sets, steps and {vertices} SVG vertices")
