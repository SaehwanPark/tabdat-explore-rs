"""Development-only full §14.2 summary/preparation check; not needed by TabDat.

python3 check_analysis.py R_analysis_summary.csv R_source_sample.csv R_prepared_sample.csv native.jsonl fishing_source_sample.csv fishing_prepared_sample.csv
Also run check_stored_comparison.py and check_figure.py on this same native output.
"""

from decimal import Decimal
import json
import math
import sys

sys.dont_write_bytecode = True
from check_comparison import close, read_csv


ORIGINAL = ("mode", "price", "crate", "dbeach", "dpier", "dprivate", "dcharter",
  "pbeach", "ppier", "pprivate", "pcharter", "qbeach", "qpier", "qprivate", "qcharter", "income")
SOURCE = ORIGINAL + ("ydiv1000",)
PREPARED = SOURCE + ("charter", "pratio", "lnrelp")
COHORTS = (("overall", 630), ("pier", 178), ("charter", 452))
LAYOUT = [("source", "overall", SOURCE, 1182)] + [
  ("full", name, PREPARED, size) for name, size in COHORTS] + [
  ("table14_1", name, ("charter", "pcharter", "ppier", "lnrelp"), size) for name, size in COHORTS] + [
  ("fitted", "overall", ("charter", "logit_classical_pr", "probit_classical_pr", "ols_classical_xb"), 630)]


def records(reference, native, variables, size):
  expected, actual = read_csv(reference), read_csv(native)
  assert len(expected) == len(actual) == size
  columns = ["source_row", *variables]
  assert list(expected[0]) == list(actual[0]) == columns
  integral = {"source_row", "mode", "dbeach", "dpier", "dprivate", "dcharter", "charter"}
  for r, n in zip(expected, actual):
    for field in columns:
      if field in integral:
        assert Decimal(n[field]) == Decimal(r[field]), (field, n, r)
      else:
        close(n[field], r[field], design=True)
    close(n["ydiv1000"], float(n["income"])/1000, design=True)
    if "charter" in variables:
      assert Decimal(n["charter"]) == (Decimal(n["mode"]) == 4)
      assert float(n["pcharter"]) > 0 and float(n["ppier"]) > 0
      x = math.log(float(n["pcharter"])/float(n["ppier"]))
      close(n["lnrelp"], x, design=True)
      close(n["pratio"], 100*x, design=True)
  ids = [int(Decimal(row["source_row"])) for row in actual]
  assert ids == sorted(set(ids))
  return actual


def check(reference, source_reference, prepared_reference, native, source_native, prepared_native):
  raw = records(source_reference, source_native, SOURCE, 1182)
  assert [int(Decimal(row["source_row"])) for row in raw] == list(range(1, 1183))
  prepared = records(prepared_reference, prepared_native, PREPARED, 630)
  selected = [row for row in raw if Decimal(row["mode"]) in (2, 4)]
  assert len(selected) == 630
  assert sum(Decimal(row["mode"]) == 2 for row in selected) == 178
  assert sum(Decimal(row["charter"]) for row in prepared) == 452
  for r, p in zip(selected, prepared):
    for field in ("source_row", *SOURCE):
      assert Decimal(r[field]) == Decimal(p[field]), (field, r, p)
  with open(native, encoding="utf-8") as stream:
    envelopes = [json.loads(line) for line in stream]
  assert all(e["schema_version"] == 1 and e.get("error") is None for e in envelopes)
  summaries = [e["data"]["rows"] for e in envelopes if e["result_type"] == "SummarizeResult"]
  refs = read_csv(reference)
  assert len(summaries) == len(LAYOUT) == 8
  assert len(refs) == 93
  assert [(r["block"], r["cohort"], r["variable"]) for r in refs] == [
    (block, cohort, variable) for block, cohort, variables, _ in LAYOUT for variable in variables]
  offset = 0
  for (block, cohort, variables, size), rows in zip(LAYOUT, summaries):
    assert [row["variable"] for row in rows] == list(variables), (block, cohort)
    for row, r in zip(rows, refs[offset:offset+len(variables)]):
      assert row["count"] == int(r["count"]) == size, (block, cohort, row)
      for field in ("mean", "std_dev", "minimum", "maximum"):
        close(row[field], r[field])
    offset += len(variables)
  assert summaries[-1][-1]["minimum"] < 0 and summaries[-1][-1]["maximum"] > 1
  print("Matched all 465 quantities across 8 source/full-cohort/Table14.1/fitted summary blocks (93 rows), plus every field of 1,182 source and 630 prepared records in original order; SD uses n-1.")
  print("Float64 income/1000, charter=(mode4), 100*ln(pcharter/ppier) and lnrelp checked; OLS remains unbounded. This is not historical float32/format/RNG parity.")


if __name__ == "__main__":
  assert len(sys.argv) == 7, __doc__
  check(*sys.argv[1:])
