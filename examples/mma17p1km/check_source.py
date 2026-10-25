#!/usr/bin/env python3
"""Source-bound audit: pin retrieved originals and check every printed sts-list row."""
import csv
import hashlib
from pathlib import Path
import re
import sys
from check_reference import check, rows

assert len(sys.argv) == 6, "usage: check_source.py strkdur.asc author.do author.txt output_directory reference.csv"
raw, program, log, output, reference = map(Path, sys.argv[1:])
for path, digest in [(raw, "cf5ed7b73d0c08c5295493720bc2b8d2f271ad752b922acfc893f6fc4a3f8059"),
    (program, "4e27dde09e230f369f8fb17f7e7b459ce63e39a40153431e58900700767c253f"),
    (log, "e0621037ec861ad4cf891e8bce77263e6f9aaa9413e60b3412b1edea4e115d50")]:
  assert hashlib.sha256(path.read_bytes()).hexdigest() == digest, f"Unexpected source bytes: {path}"
original = [line.split() for line in raw.read_text().splitlines() if line.strip()]
converted = rows(output / "strikes.csv")
assert len(original) == len(converted) == 566
for i, (a, b) in enumerate(zip(original, converted), 1):
  assert float(b["source_row"]) == i
  assert float(b["dur"]) == float(a[0]) and float(b["gdp"]) == float(a[1])
assert sum(float(a[0]) for a in original) == 24691
checks, vertices = check(output, output / "strikes.csv", reference)
# Integer counts are exact; four-decimal Stata quantities get half a printed unit.
printed = []
for line in log.read_text().splitlines():
  fields = line.split()
  if len(fields) == 8 and all(re.fullmatch(r"\d+", f) for f in fields[:4]):
    if all(re.fullmatch(r"\d+\.\d+|\.", f) for f in fields[4:]):
      printed.append(fields)
table = rows(output / "strikes_survival.csv")
assert len(printed) == len(table) == 141
facts = 0
for native, author in zip(table, printed):
  assert [int(float(native[k])) for k in ["time", "at_risk", "events"]] == list(map(int, author[:3]))
  assert author[3] == "0"  # no lost/censored spells
  for key, text in zip(["survival", "survival_se", "survival_lower", "survival_upper"], author[4:]):
    if text == ".":
      assert native[key] == ""
    else:
      assert abs(float(native[key]) - float(text)) <= .000050000001, (key, native[key], text)
    facts += 1
  facts += 4
print(f"Pinned566-row original audit: {checks} R cells, {facts} printed facts, {vertices} SVG vertices")
