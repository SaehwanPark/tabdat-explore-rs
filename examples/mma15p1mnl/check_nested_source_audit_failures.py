"""Optional source-audit corruption regressions; needs the prepared reference workdir."""
import argparse
import csv
import hashlib
import pathlib
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("work", type=pathlib.Path)
parser.add_argument("--r-check", action="store_true", help="also check the R stationary-control gate (requires development mlogit2.0.0)")
args = parser.parse_args()
source = args.work.resolve()
scripts = pathlib.Path(__file__).resolve().parent
files = ["Nldata.asc", "Nldata.csv", "mma15p2gev.do", "mma15p2gev.txt", "mma15p4gev.lim", "mma15p4gev.out", "R_nested_quantities.csv", "R_normalized_common_quantities.csv", "R_nested_source_audit.csv"]
def fingerprints():
  return {name: hashlib.sha256((source/name).read_bytes()).hexdigest() for name in files}

def read_csv(path):
  with path.open() as stream:
    return list(csv.DictReader(stream))

def write_csv(path, rows):
  with path.open("w", newline="") as stream:
    writer = csv.DictWriter(stream, fieldnames=list(rows[0]))
    writer.writeheader()
    writer.writerows(rows)

def check(directory):
  return subprocess.run(["python3", str(scripts/"check_nested_source_audit.py"), str(directory)], capture_output=True, text=True, timeout=45)

before = fingerprints()
baseline = check(source)
assert baseline.returncode == 0, baseline
cases = ["prepared_design", "shared_shifted_likelihood", "false_shore_derivative", "wrong_limdep_mapping", "omitted_float32", "duplicate_diagnostic", "zero_free_information", "nonstationary_control"]
for case in cases:
  with tempfile.TemporaryDirectory(prefix="tabdat-nested-source-audit-") as directory:
    work = pathlib.Path(directory)
    for name in files:
      shutil.copyfile(source/name, work/name)
    audit = read_csv(work/"R_nested_source_audit.csv")
    if case == "prepared_design":
      wide = read_csv(work/"Nldata.csv")
      wide[0]["pbeach"] = str(float(wide[0]["pbeach"])+1)
      write_csv(work/"Nldata.csv", wide)
    elif case == "shared_shifted_likelihood":
      for row in audit:
        if row["quantity"] == "log_likelihood" and row["section"] in ["multistart", "profile_candidate", "profile"]:
          row["value"] = str(float(row["value"])+.01)
    elif case == "false_shore_derivative":
      for row in audit:
        if row["section"] == "boundary" and row["quantity"] == "shore_score":
          row["value"] = "-3"
    elif case == "wrong_limdep_mapping":
      for row in audit:
        if (row["section"], row["quantity"], row["i"]) == ("historical_limdep", "parameter", "4"):
          row["value"] = str(32.17258445*.007377036421)
    elif case == "omitted_float32":
      original = {(row["section"], row["quantity"], row["i"]): row["value"] for row in audit if row["j"] == "0"}
      for row in audit:
        if row["j"] == "1" and row["section"] in ["historical_point", "historical_rounding_vertex"]:
          row["value"] = original[(row["section"], row["quantity"], row["i"])]
    elif case == "duplicate_diagnostic":
      audit.append(audit[0].copy())
    elif case == "zero_free_information":
      for row in audit:
        if (row["section"], row["quantity"]) == ("boundary", "minimum_free_information"):
          row["value"] = "0"
    elif case == "nonstationary_control":
      # Coherent reported/recomputed diagnostics alone are insufficient: the
      # supposed double stationary control must satisfy its own numerical gates.
      gold = read_csv(work/"R_nested_quantities.csv")
      stata = [-.0013303, .1284825, -11.40196, .1108341, 29.98591, 14.06438]
      for row in gold:
        if (row["model"], row["quantity"]) == ("nested", "parameter_value"):
          row["value"] = str(stata[int(row["i"])-1])
      write_csv(work/"R_nested_quantities.csv", gold)
      replacement = {(row["quantity"], row["j"]): row["value"] for row in audit if row["section"] == "historical_point" and row["i"] == "2"}
      for row in audit:
        if row["section"] == "historical_point" and row["i"] == "1":
          row["value"] = replacement[(row["quantity"], row["j"])]
    write_csv(work/"R_nested_source_audit.csv", audit)
    result = check(work)
    assert result.returncode != 0, (case, result)
    if case == "nonstationary_control" and args.r_check:
      result = subprocess.run(["Rscript", str(scripts/"reference_nested_source_audit.R"), str(work/"Nldata.csv"), str(work/"R_nested_quantities.csv"), str(work/"R_normalized_common_quantities.csv")], capture_output=True, text=True, timeout=600)
      assert result.returncode != 0 and not result.stdout.strip() and "scaled_score" in result.stderr, result
    print(f"Rejected {case}")
assert fingerprints() == before
print("Valid1068-diagnostic baseline passes; all eight corruptions rejected; nine original source/reference inputs preserved.")
