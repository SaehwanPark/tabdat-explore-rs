#!/usr/bin/env python3
"""Check the patents panel xtsum script against raw-token R and the author log."""
import csv
import hashlib
import json
import math
import re
import sys
from pathlib import Path

DATA_SHA256 = "87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8"
DO_SHA256 = "e4b90cc60b52585fba6485d0cc9ed79ce0b9c9fc6fd0f390a435b15ef7dd98aa"
LOG_SHA256 = "92f68c76063cf083217724f1b1b45cfaf8ae7a8c7ce0b4a1d5f62accd86dc208"
FIELDS = [
  "id", "year", "cusip", "ardssic", "scisect", "logk", "sumpat", "logr",
  "pat", "lpat", "dpat", "randd", "logrl1", "logrl2", "logrl3", "logrl4",
  "logrl5", "dyear2", "dyear3", "dyear4", "dyear5",
]
XT_FIELDS = [
  "overall_N", "overall_mean", "overall_sd", "overall_min", "overall_max",
  "between_n", "between_sd", "between_min", "between_max", "within_N",
  "within_sd", "within_min", "within_max", "mean_T",
]
XT_VARIABLES = FIELDS


def require(condition, message):
  if not condition:
    raise AssertionError(message)


def sha256(path):
  digest = hashlib.sha256()
  with open(path, "rb") as stream:
    for chunk in iter(lambda: stream.read(1024 * 1024), b""):
      digest.update(chunk)
  return digest.hexdigest()


def read_csv(path):
  with open(path, newline="", encoding="utf-8") as stream:
    return list(csv.DictReader(stream))


def close(actual, expected, label, tolerance=None):
  actual = float(actual)
  expected = float(expected)
  require(math.isfinite(actual) and math.isfinite(expected), f"{label}: nonfinite value")
  if tolerance is None:
    tolerance = 1e-10 + 1e-12 * abs(expected)
  require(abs(actual - expected) <= tolerance, f"{label}: {actual} != {expected}")


def printed_close(actual, token, label):
  expected = float(token)
  decimals = len(token.partition(".")[2]) if "." in token else 0
  half_unit = 0.500001 * (10.0 ** -decimals)
  close(actual, expected, label, half_unit)


def author_xtsum(lines):
  values = {}
  for i, line in enumerate(lines):
    match = re.match(r"^\s*(\w+)\s+overall\s*\|", line)
    if not match:
      continue
    variable = match.group(1).lower()
    parts = line.split("|")
    overall = parts[1].split()
    n = int(re.search(r"N\s*=\s*(\d+)", parts[2]).group(1))
    between_line = lines[i + 1].split("|")
    within_line = lines[i + 2].split("|")
    require("between" in between_line[0] and "within" in within_line[0], f"{variable}: malformed xtsum block")
    between = between_line[1].split()
    panels = int(re.search(r"n\s*=\s*(\d+)", between_line[2]).group(1))
    within = within_line[1].split()
    periods = int(re.search(r"T\s*=\s*(\d+)", within_line[2]).group(1))
    require(len(overall) == 4 and len(between) == 3 and len(within) == 3, f"{variable}: unexpected xtsum fields")
    values[variable] = {
      "overall_N": n, "overall": overall, "between_n": panels,
      "between": between, "mean_T": periods, "within": within,
    }
  require(list(values) == XT_VARIABLES, "author xtsum rows or variable order changed")
  return values


def check(native_dir, reference_dir, raw_path, author_do, author_log):
  for path, expected in [
    (raw_path, DATA_SHA256), (author_do, DO_SHA256), (author_log, LOG_SHA256),
  ]:
    require(sha256(path) == expected, f"source checksum mismatch: {path}")

  reference_dir = Path(reference_dir)
  raw_reference = read_csv(reference_dir / "full.csv")
  reference = read_csv(reference_dir / "xtsum.csv")
  author_reference = read_csv(reference_dir / "author_float32_xtsum.csv")
  require(len(raw_reference) == 3460, "raw-token R reference must contain all full-decade rows")
  require([row["variable"] for row in reference] == XT_VARIABLES, "R xtsum variable order changed")
  require([row["variable"] for row in author_reference] == XT_VARIABLES, "float32 R xtsum variable order changed")

  prepared = read_csv(Path(native_dir) / "patents_decade_prepared.csv")
  require(len(prepared) == 3460, "prepared full-decade file must contain 3,460 rows")
  require(list(prepared[0]) == FIELDS, "prepared full-decade schema changed")
  for actual, expected in zip(prepared, raw_reference):
    for field in FIELDS:
      if expected[field] == "":
        require(actual[field] == "", f"prepared {field}: expected source missingness")
      elif field in FIELDS[:9]:
        require(float(actual[field]) == float(expected[field]), f"prepared source {field} changed")
      else:
        close(actual[field], expected[field], f"prepared derived {field}")

  with open(Path(native_dir) / "xtsum.jsonl", encoding="utf-8") as stream:
    events = [json.loads(line) for line in stream if line.strip()]
  require(
    [event.get("result_type") for event in events]
    == ["LoadResult", "IsidResult", "DescribeResult", "TableResult", "TableResult"],
    "xtsum event sequence changed",
  )
  require(all(event.get("schema_version") == 1 and "error" not in event for event in events), "native xtsum script failed")
  columns = events[2]["data"]["dataset"]["columns"]
  require([column["name"] for column in columns] == FIELDS, "native source schema changed")
  require(events[3]["data"]["headers"] == ["validated_rows"], "source guard schema changed")
  require(events[3]["data"]["rows"] == [[3460]], "source guard did not validate all rows")
  table = events[4]["data"]
  require(table["headers"] == ["variable", *XT_FIELDS], "native xtsum output schema changed")
  require(len(table["rows"]) == len(reference) == 21, "native xtsum must return all 21 variables")

  max_error = 0.0
  for actual, expected in zip(table["rows"], reference):
    require(actual[0] == expected["variable"], "native xtsum order differs from Base R")
    for index, field in enumerate(XT_FIELDS, 1):
      if field in ("overall_N", "between_n", "within_N"):
        require(int(actual[index]) == int(expected[field]), f"{actual[0]} {field} differs from R")
      else:
        error = abs(float(actual[index]) - float(expected[field]))
        max_error = max(max_error, error)
        close(actual[index], expected[field], f"{actual[0]} {field}")
    require(int(actual[1]) == int(actual[10]), f"{actual[0]} overall/within counts differ")

  expected_counts = {"ardssic": (3360, 336, 10)}
  expected_counts.update({f"logrl{j}": (346 * (10 - j), 346, 10 - j) for j in range(1, 6)})
  for row in table["rows"]:
    variable = row[0]
    expected_n, expected_panels, expected_t = expected_counts.get(variable, (3460, 346, 10))
    require((int(row[1]), int(row[6]), int(row[14])) == (expected_n, expected_panels, expected_t), f"{variable}: expected source N/n/T")

  log_lines = Path(author_log).read_text(encoding="utf-8", errors="strict").splitlines()
  logged = author_xtsum(log_lines)
  for expected in author_reference:
    author = logged[expected["variable"]]
    require(int(expected["overall_N"]) == author["overall_N"], f"{expected['variable']}: author N")
    require(int(expected["between_n"]) == author["between_n"], f"{expected['variable']}: author n")
    require(int(expected["mean_T"]) == author["mean_T"], f"{expected['variable']}: author T")
    for token, field in zip(author["overall"], ["overall_mean", "overall_sd", "overall_min", "overall_max"]):
      printed_close(expected[field], token, f"author overall {expected['variable']} {field}")
    for token, field in zip(author["between"], ["between_sd", "between_min", "between_max"]):
      printed_close(expected[field], token, f"author between {expected['variable']} {field}")
    for token, field in zip(author["within"], ["within_sd", "within_min", "within_max"]):
      printed_close(expected[field], token, f"author within {expected['variable']} {field}")

  print(
    "Matched checksum-pinned full-decade patents source, all 21 variable-specific "
    "overall/between/within rows against raw-token Base R and the author log; "
    f"max native/R error={max_error:.3g}."
  )


if __name__ == "__main__":
  if len(sys.argv) != 6:
    raise SystemExit("usage: check_xtsum.py native_dir reference_dir patr7079.asc author.do author.txt")
  try:
    check(*sys.argv[1:])
  except (AssertionError, OSError, StopIteration, ValueError, KeyError, IndexError) as error:
    raise SystemExit(f"FAIL: {error}")
