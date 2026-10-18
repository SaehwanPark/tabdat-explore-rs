"""Development-only Table14.1 reference check; not needed to execute TabDat.

python3 check_summary.py R_summary.csv R_summary_sample.csv native.jsonl fishing_summary_sample.csv
"""

from decimal import Decimal
import json
import math
import sys

sys.dont_write_bytecode = True
from check_comparison import close, read_csv


VARIABLES = ("charter", "pcharter", "ppier", "lnrelp")
COHORTS = (("overall", 630), ("pier", 178), ("charter", 452))


def check(reference, sample, native, native_sample):
  expected, actual = read_csv(sample), read_csv(native_sample)
  assert len(expected) == len(actual) == 630
  assert list(actual[0]) == ["source_row", "mode", "charter", "pcharter", "ppier", "lnrelp"]
  ids = [int(row["source_row"]) for row in expected]
  assert ids == sorted(set(ids)) and all(1 <= i <= 1182 for i in ids)
  assert sum(int(row["mode"]) == 2 for row in expected) == 178
  assert sum(int(row["charter"]) for row in expected) == 452
  for r, n in zip(expected, actual):
    assert int(r["mode"]) in (2, 4)
    assert int(r["charter"]) == (int(r["mode"]) == 4)
    for field in ("source_row", "mode", "charter"):
      assert Decimal(n[field]) == Decimal(r[field]), (field, n, r)
    for field in ("pcharter", "ppier", "lnrelp"):
      close(n[field], r[field], design=True)
    assert float(r["pcharter"]) > 0 and float(r["ppier"]) > 0
    close(r["lnrelp"], math.log(float(r["pcharter"])/float(r["ppier"])), design=True)
  with open(native, encoding="utf-8") as source:
    envelopes = [json.loads(line) for line in source]
  assert all(e["schema_version"] == 1 for e in envelopes)
  assert [e["result_type"] for e in envelopes] == [
    "LoadResult", "SqlCreateResult", "IsidResult", "SummarizeResult",
    "SummarizeResult", "SummarizeResult", "ExportResult",
  ]
  summaries = [e["data"]["rows"] for e in envelopes if e["result_type"] == "SummarizeResult"]
  refs = read_csv(reference)
  assert len(refs) == 12
  assert [(r["cohort"], r["variable"]) for r in refs] == [
    (name, variable) for name, _ in COHORTS for variable in VARIABLES
  ]
  for i, ((name, size), rows) in enumerate(zip(COHORTS, summaries)):
    assert [r["variable"] for r in rows] == list(VARIABLES)
    for j, row in enumerate(rows):
      r = refs[4*i+j]
      assert row["count"] == int(r["count"]) == size, (name, row)
      for field in ("mean", "std_dev", "minimum", "maximum"):
        close(row[field], r[field])
    if name != "overall":
      assert rows[0]["std_dev"] == 0 and rows[0]["mean"] == (name == "charter")
  print("Matched all 60 Table14.1 summary quantities and 630 source/outcome/design records in original order; SD uses n-1, not n.")
  print("Native outputs are three ordered summary blocks, not a joined-table command or historical float32 formatting parity.")


if __name__ == "__main__":
  assert len(sys.argv) == 5, __doc__
  check(*sys.argv[1:])
