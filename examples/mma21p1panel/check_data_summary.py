#!/usr/bin/env python3
"""Check the Chapter 21 data-summary script against Base R and the author log."""
import csv
import hashlib
import json
import math
import re
import sys
from collections import Counter
from pathlib import Path

MOM_SHA256 = "2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77"
DO_SHA256 = "775890ef25903477e5be24027c070705910247daf46f9ed7b13d0ed0b48cc44a"
LOG_SHA256 = "3c4e65bdfcd6fe8f77fe32361bc29e041ad7caf9268ba35ecf52e2c78d5b88f6"
SOURCE_FIELDS = ["lnhr", "lnwg", "kids", "ageh", "agesq", "disab", "id", "year"]
XT_VARIABLES = ["lnhr", "lnwg", "kids", "ageh", "agesq", "disab"]
VARIABLES = SOURCE_FIELDS + [f"dyear{i}" for i in range(1, 11)]
NUMERIC_SUMMARY = ["mean", "std_dev", "minimum", "maximum"]
XT_FIELDS = [
  "overall_mean", "overall_sd", "overall_min", "overall_max", "between_sd",
  "between_min", "between_max", "within_sd", "within_min", "within_max",
]


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


def close(actual, expected, label, tolerance=1e-10):
  actual = float(actual)
  expected = float(expected)
  require(math.isfinite(actual) and math.isfinite(expected), f"{label}: nonfinite value")
  require(abs(actual - expected) <= tolerance, f"{label}: {actual} != {expected}")


def printed_close(actual, token, label):
  expected = float(token)
  decimals = len(token.partition(".")[2]) if "." in token else 0
  half_unit = 0.500001 * (10.0 ** -decimals)
  close(actual, expected, label, half_unit)


def find_one(events, result_type):
  selected = [event["data"] for event in events if event.get("result_type") == result_type]
  require(len(selected) == 1, f"expected one {result_type}, got {len(selected)}")
  return selected[0]


def author_summary(lines):
  names = set(VARIABLES)
  start = next(i for i, line in enumerate(lines) if line.strip() == ". summarize")
  end = next(i for i, line in enumerate(lines[start + 1 :], start + 1) if line.strip() == ". save mom, replace")
  values = {}
  for line in lines[start:end]:
    if "|" not in line:
      continue
    left, right = line.split("|", 1)
    variable = left.strip()
    fields = right.split()
    if variable in names and len(fields) == 5:
      values[variable] = fields
  require(list(values) == VARIABLES, "author log summarize rows or variable order changed")
  return values


def author_xtsum(lines):
  values = {}
  wanted = set(XT_VARIABLES)
  for i, line in enumerate(lines):
    match = re.match(r"^\s*(\w+)\s+overall\s*\|", line)
    if not match or match.group(1) not in wanted:
      continue
    variable = match.group(1)
    parts = line.split("|")
    overall = parts[1].split()
    n = int(re.search(r"N\s*=\s*(\d+)", parts[2]).group(1))
    between_line = lines[i + 1].split("|")
    within_line = lines[i + 2].split("|")
    require("between" in between_line[0] and "within" in within_line[0], f"{variable}: malformed xtsum block")
    between = between_line[1].split()
    people = int(re.search(r"n\s*=\s*(\d+)", between_line[2]).group(1))
    within = within_line[1].split()
    periods = int(re.search(r"T\s*=\s*(\d+)", within_line[2]).group(1))
    require(len(overall) == 4 and len(between) == 3 and len(within) == 3, f"{variable}: unexpected xtsum fields")
    values[variable] = {
      "overall_N": n,
      "overall": overall,
      "between_n": people,
      "between": between,
      "mean_T": periods,
      "within": within,
    }
  require(list(values) == XT_VARIABLES, "author log xtsum rows or variable order changed")
  return values


def check(native_path, reference_dir, mom_dat, mom_csv, author_do, author_log):
  for path, expected in [
    (mom_dat, MOM_SHA256), (author_do, DO_SHA256), (author_log, LOG_SHA256),
  ]:
    require(sha256(path) == expected, f"source checksum mismatch: {path}")

  raw_lines = Path(mom_dat).read_text(encoding="ascii").splitlines()
  tokens = [line.split() for line in raw_lines]
  require(len(tokens) == 5320 and all(len(row) == 8 for row in tokens), "MOM.dat must contain 5,320 eight-field records")
  with open(mom_csv, newline="", encoding="utf-8") as stream:
    converted = list(csv.reader(stream))
  expected_header = ["source_row", *SOURCE_FIELDS]
  require(converted[0] == expected_header, "prepared MOM.csv header changed")
  expected_rows = [[str(i), *row] for i, row in enumerate(tokens, 1)]
  require(converted[1:] == expected_rows, "prepared MOM.csv did not preserve all source tokens and order")

  with open(native_path, encoding="utf-8") as stream:
    events = [json.loads(line) for line in stream if line.strip()]
  require(events and all(event.get("schema_version") == 1 and "error" not in event for event in events), "native script did not complete cleanly")
  counts = Counter(event.get("result_type") for event in events)
  for kind, expected in {
    "LoadResult": 1, "IsidResult": 2, "TabulateResult": 1, "DropResult": 1,
    "GenerateResult": 10, "DescribeResult": 1, "SummarizeResult": 1, "TableResult": 3,
    "XtSumResult": 1,
  }.items():
    require(counts[kind] == expected, f"expected {expected} {kind}, got {counts[kind]}")

  reference_dir = Path(reference_dir)
  ref_summary = read_csv(reference_dir / "summary.csv")
  ref_author_summary = read_csv(reference_dir / "author_float32_summary.csv")
  ref_years = read_csv(reference_dir / "year_frequency.csv")
  ref_structure = read_csv(reference_dir / "panel_structure.csv")[0]
  ref_t = read_csv(reference_dir / "panel_t_distribution.csv")
  ref_xtsum = read_csv(reference_dir / "xtsum.csv")
  ref_author_xtsum = read_csv(reference_dir / "author_float32_xtsum.csv")
  require([row["variable"] for row in ref_summary] == VARIABLES, "R summary variable order changed")
  require([row["variable"] for row in ref_xtsum] == XT_VARIABLES, "R xtsum variable order changed")

  tabulate = find_one(events, "TabulateResult")
  require(tabulate["headers"] == ["year", "Count", "Percent"], "native year tabulate schema changed")
  require(len(tabulate["rows"]) == 10, "native tabulate must report ten years")
  for actual, expected in zip(tabulate["rows"], ref_years):
    require(int(actual[0]) == int(expected["year"]), "native year order differs from source")
    require(int(actual[1]) == int(expected["Count"]) == 532, "each year must contain 532 people")
    close(actual[2], expected["Percent"], "year percent")

  describe = find_one(events, "DescribeResult")["dataset"]
  require([column["name"] for column in describe["columns"]] == VARIABLES, "native 18-variable inventory changed")
  require([column["name"] for column in describe["columns"][-10:]] == [f"dyear{i}" for i in range(1, 11)], "year indicators are not in ascending order")
  require(all(column["data_type"] == "TINYINT" for column in describe["columns"][-10:]), "generated indicators must be numeric 0/1 columns")

  native_summary = find_one(events, "SummarizeResult")["rows"]
  require(len(native_summary) == 18, "native summarize must report all source and year-indicator variables")
  for actual, expected in zip(native_summary, ref_summary):
    require(actual["variable"] == expected["variable"], "native summary variable order differs from R")
    require(int(actual["count"]) == int(expected["observations"]) == 5320, f"{actual['variable']}: observation count")
    for field, output_field in zip(NUMERIC_SUMMARY, ["mean", "std_dev", "minimum", "maximum"]):
      close(actual[output_field], expected[field], f"{actual['variable']} {output_field}")

  tables = {
    tuple(event["data"]["headers"]): event["data"]["rows"]
    for event in events if event.get("result_type") == "TableResult"
  }
  source_header = ("validated_source_rows", "people", "first_year", "last_year")
  structure_header = ("observations", "people", "first_id", "last_id", "first_year", "last_year", "min_T", "mean_T", "max_T", "min_delta", "max_delta")
  t_header = ("T_i", "pattern", "people", "percent", "cumulative_percent")
  require(source_header in tables and len(tables[source_header]) == 1, "source guard result missing")
  require(structure_header in tables and len(tables[structure_header]) == 1, "panel structure table missing")
  require(t_header in tables and len(tables[t_header]) == len(ref_t), "panel T distribution missing")
  for actual, field in zip(tables[source_header][0], ["observations", "people", "first_year", "last_year"]):
    close(actual, ref_structure[field], f"source guard {field}", 0.0)
  for actual, field in zip(tables[structure_header][0], structure_header):
    close(actual, ref_structure[field], f"panel structure {field}")
  for actual, expected in zip(tables[t_header], ref_t):
    require(int(actual[0]) == int(expected["T_i"]), "T_i distribution differs from R")
    require(actual[1] == expected["pattern"], "panel pattern differs from R")
    require(int(actual[2]) == int(expected["people"]), "panel frequency differs from R")
    close(actual[3], expected["percent"], "panel percent")
    close(actual[4], expected["cumulative_percent"], "panel cumulative percent")

  native_xtsum = find_one(events, "XtSumResult")
  require(native_xtsum["id_variable"] == "id", "native xtsum panel identifier changed")
  native_xt = native_xtsum["rows"]
  require([row["variable"] for row in native_xt] == XT_VARIABLES, "native xtsum order differs from R")
  max_error = 0.0
  for actual, expected in zip(native_xt, ref_xtsum):
    variable = actual["variable"]
    for field, expected_field in [
      ("overall_n", "overall_N"),
      ("between_n", "between_n"),
      ("within_n", "within_observations"),
    ]:
      require(int(actual[field]) == int(expected[expected_field]), f"{variable} {field} differs from R")
    close(actual["mean_t"], expected["mean_T"], f"{variable} mean_t")
    close(expected["within_mean"], expected["overall_mean"], f"{variable} Base R within mean")
    for field in XT_FIELDS:
      error = abs(float(actual[field]) - float(expected[field]))
      max_error = max(max_error, error)
      close(actual[field], expected[field], f"{variable} {field}")

  log_lines = Path(author_log).read_text(encoding="utf-8", errors="strict").splitlines()
  summary_author = author_summary(log_lines)
  require([row["variable"] for row in ref_author_summary] == VARIABLES, "R float32 author summary variable order changed")
  for row in ref_author_summary:
    fields = summary_author[row["variable"]]
    require(int(fields[0]) == int(row["observations"]), f"{row['variable']}: author summary N")
    for token, field in zip(fields[1:], ["mean", "std_dev", "minimum", "maximum"]):
      printed_close(row[field], token, f"author summarize {row['variable']} {field}")
  xt_author = author_xtsum(log_lines)
  require([row["variable"] for row in ref_author_xtsum] == XT_VARIABLES, "R float32 author xtsum variable order changed")
  for row in ref_author_xtsum:
    author = xt_author[row["variable"]]
    require(int(row["overall_N"]) == author["overall_N"] == 5320, f"{row['variable']}: author xtsum N")
    require(int(row["between_n"]) == author["between_n"] == 532, f"{row['variable']}: author xtsum n")
    require(int(row["mean_T"]) == author["mean_T"] == 10, f"{row['variable']}: author xtsum T")
    for token, field in zip(author["overall"], ["overall_mean", "overall_sd", "overall_min", "overall_max"]):
      printed_close(row[field], token, f"author xtsum {row['variable']} {field}")
    for token, field in zip(author["between"], ["between_sd", "between_min", "between_max"]):
      printed_close(row[field], token, f"author xtsum {row['variable']} {field}")
    for token, field in zip(author["within"], ["within_sd", "within_min", "within_max"]):
      printed_close(row[field], token, f"author xtsum {row['variable']} {field}")

  require(any("Delta(year) = 1" in line for line in log_lines), "author xtdes time delta changed")
  require(any("1111111111" in line and "532" in line for line in log_lines), "author xtdes complete-panel pattern changed")
  require(any("XXXXXXXXXX" in line and "532" in line for line in log_lines), "author xtdes all-observed pattern changed")
  print(
    "Matched checksum-pinned 5,320-row MOM source, ten year counts, all 18 "
    "summary rows, 532 complete ten-year panels, and six overall/between/within "
    f"rows against Base R and the author log; max native/R xtsum error={max_error:.3g}."
  )


if __name__ == "__main__":
  if len(sys.argv) != 7:
    raise SystemExit(
      "usage: check_data_summary.py native.jsonl reference_dir MOM.dat MOM.csv author.do author.txt"
    )
  try:
    check(*sys.argv[1:])
  except (AssertionError, OSError, StopIteration, ValueError, KeyError, IndexError) as error:
    raise SystemExit(f"FAIL: {error}")
