#!/usr/bin/env python3
"""Development-only full original-source RE audit; Python stdlib only."""
import csv
from decimal import Decimal
import hashlib
import json
import math
from pathlib import Path
import sys

native, reference, raw = map(Path, sys.argv[1:])
assert hashlib.sha256(raw.read_bytes()).hexdigest() == "2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77"


def records(path):
  with path.open(newline="") as stream:
    return list(csv.DictReader(stream))


def close(actual, expected, label, tail=False):
  a, e = float(actual), float(expected)
  tolerance = abs(e)*1e-8 if tail else 1e-9
  assert math.isfinite(a) and math.isfinite(e) and abs(a-e) <= tolerance, (label,a,e,tolerance)


events = [json.loads(line) for line in (native / "random_effects.jsonl").read_text().splitlines()]
assert all(e["schema_version"] == 1 and "error" not in e for e in events)
fits = [e["data"] for e in events if e["result_type"] == "RandomEffectsResult"]
assert len(fits) == 1
fit = fits[0]
assert fit["estimator"] == "balanced_swamy_arora_random_effects_gls"
assert fit["reference_distribution"] == "normal" and fit["wald_reference"] == "chi_square"
assert fit["covariance_convention"] == "quasi_demeaned_rss_over_n_minus_k"
assert fit["sample"] == {"total_observations":5320,"dropped_observations":0}
assert fit["covariance_parameter_names"] == ["intercept","lnwg"]
assert "log_likelihood" not in fit and "r_squared" not in fit
scalars = records(reference / "reference.csv")
assert len(scalars) == 30
for row in scalars:
  value = fit
  for part in row["path"].strip("/").split("/"):
    value = value[int(part)] if isinstance(value,list) else value[part]
  close(value,row["value"],row,tail=row["path"].endswith("p_value"))
actual = records(native / "panel_re_gls_predictions.csv")
expected = records(reference / "predictions.csv")
assert len(actual) == len(expected) == 5320
assert list(actual[0]) == list(expected[0])
cells = 0
for a,e in zip(actual,expected):
  for key in e:
    if key in ("source_row","id","year"):
      assert float(a[key]) == float(e[key]), (key,a,e)
    else:
      close(a[key],e[key],(key,e["source_row"]))
    cells += 1
facts = records(reference / "author_printed.csv")
assert len(facts) == 8
for fact in facts:
  target = Decimal(fact["printed"])
  assert abs(Decimal(fact["value"])-target) <= Decimal(10)**target.as_tuple().exponent/2, fact
print(f"PASS: {len(scalars)} R/plm scalars, {cells} exported cells across 5320 rows, 8 author float32 printed facts")
