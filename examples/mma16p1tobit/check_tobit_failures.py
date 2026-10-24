"""Permanent adversarial checks for the source-bound Tobit audit (no R needed).

Usage: python3 check_tobit_failures.py VALIDATED_WORK_DIRECTORY
Only owned temporary copies are mutated; input artifacts remain unchanged.
"""
import contextlib
import csv
import hashlib
import io
import json
import math
import pathlib
import shutil
import sys
import tempfile

from check_tobit import audit, NORMAL_95

FILES = ["mma16p1tobit.asc", "mma16p1tobit.csv", "tobit.jsonl", "tobit_predictions.csv", "R_tobit_predictions.csv", "R_tobit_quantities.csv"]


def change_csv(work, name, change):
  with (work / name).open() as stream:
    rows = list(csv.DictReader(stream))
  fields = list(rows[0])
  change(rows)
  with (work / name).open("w") as stream:
    writer = csv.DictWriter(stream, fieldnames=fields)
    writer.writeheader()
    writer.writerows(rows)


def records(work):
  return [json.loads(line) for line in (work / "tobit.jsonl").read_text().splitlines()]


def publish(work, data):
  (work / "tobit.jsonl").write_text("".join(json.dumps(r) + "\n" for r in data))


def original_change(work):
  with (work / "mma16p1tobit.asc").open("ab") as stream:
    stream.write(b"\n")


def wrong_order(work):
  change_csv(work, "tobit_predictions.csv", lambda rows: rows.reverse())


def wrong_mean(work):
  change_csv(work, "tobit_predictions.csv", lambda rows: rows[0].update(observed_mean=rows[0]["latent"]))


def missing_quantity(work):
  change_csv(work, "R_tobit_quantities.csv", lambda rows: rows.pop())


def shared_zero_tail(work):
  data = records(work)
  data[1]["data"]["coefficients"][0]["p_value"] = 0
  publish(work, data)
  change_csv(work, "R_tobit_quantities.csv", lambda rows: [r.update(value="0") for r in rows if r["quantity"] == "coefficient_p_value" and r["i"] == "1"])


def shared_covariance(work):
  data = records(work)
  for a, b in [(0, 1), (1, 0)]:
    data[1]["data"]["covariance_matrix"][a][b] *= 1.01
  publish(work, data)
  change_csv(work, "R_tobit_quantities.csv", lambda rows: [r.update(value=str(float(r["value"]) * 1.01)) for r in rows if r["quantity"] == "joint_covariance" and (r["i"], r["j"]) in [("1", "2"), ("2", "1")]])


def shared_gradient(work):
  data = records(work)
  data[1]["data"]["mean_probability_gradient"][0] *= 1.01
  publish(work, data)
  change_csv(work, "R_tobit_quantities.csv", lambda rows: [r.update(value=str(float(r["value"]) * 1.01)) for r in rows if r["quantity"] == "mean_probability_gradient" and r["i"] == "1"])


def false_stationary_control(work):
  data = records(work)
  fit = data[1]["data"]
  row = fit["coefficients"][0]
  row["value"] += 0.001
  row["statistic"] = row["value"] / row["standard_error"]
  row["p_value"] = math.erfc(abs(row["statistic"]) / math.sqrt(2))
  row["confidence_lower"] = row["value"] - NORMAL_95 * row["standard_error"]
  row["confidence_upper"] = row["value"] + NORMAL_95 * row["standard_error"]
  fit["max_abs_scaled_score"] = fit["max_abs_scaled_newton_step"] = 0
  publish(work, data)
  def alter(rows):
    for r in rows:
      if r["quantity"] in ["scaled_score", "scaled_step"]:
        r["value"] = "0"
      elif r["quantity"].startswith("coefficient_") and r["i"] == "1":
        r["value"] = str(row[r["quantity"].removeprefix("coefficient_")])
  change_csv(work, "R_tobit_quantities.csv", alter)


def wrong_reference_cell(work):
  change_csv(work, "R_tobit_predictions.csv", lambda rows: rows[0].update(ytrunc="0"))


def main(source):
  before = {name: hashlib.sha256((source / name).read_bytes()).hexdigest() for name in FILES}
  with contextlib.redirect_stdout(io.StringIO()):
    audit(source)
  cases = [
    ("source hash", original_change, "source fingerprint"),
    ("row order", wrong_order, "source ordinal"),
    ("latent substituted for censored mean", wrong_mean, "source censored mean"),
    ("reference inventory", missing_quantity, "reference quantity inventory"),
    ("shared zero tail", shared_zero_tail, "positive normal p-value"),
    ("shared joint covariance", shared_covariance, "joint information inverse"),
    ("shared retained gradient", shared_gradient, "retained mean probability gradient"),
    ("shared false stationary control", false_stationary_control, "source score stationarity"),
    ("reference original cell", wrong_reference_cell, None),
  ]
  for name, corrupt, message in cases:
    with tempfile.TemporaryDirectory(prefix="tabdat-tobit-audit-failure-") as directory:
      work = pathlib.Path(directory)
      for filename in FILES:
        shutil.copy2(source / filename, work / filename)
      corrupt(work)
      output = io.StringIO()
      try:
        with contextlib.redirect_stdout(output):
          audit(work)
      except AssertionError as error:
        assert not output.getvalue(), name
        assert message is None or message in str(error), (name, message, str(error))
      else:
        raise AssertionError("corruption accepted: " + name)
  assert before == {name: hashlib.sha256((source / name).read_bytes()).hexdigest() for name in FILES}
  print(f"Rejected{len(cases)} targeted corruptions; all{len(FILES)} original input hashes preserved.")


if __name__ == "__main__":
  main(pathlib.Path(sys.argv[1]))
