#!/usr/bin/env python3
"""Development-only independent R refits from raw records and verified draws.

No native coefficients/means/covariances are inputs. Each copy gets a distinct
R matrix row/effect, even when its original firm is repeated. A failed fit
leaves diagnostics but never a partial-success fits.csv/complete manifest.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
import csv
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path
import shutil
import subprocess
import time

from check_native_gamma_re import raw_source

HERE = Path(__file__).resolve().parent
FIELDS = ["estimate_intercept", "estimate_slope", "estimate_lnvariance",
          "log_likelihood", "poisson_log_likelihood", "score_intercept",
          "score_slope", "score_lnvariance"]


def verified_draws(path, groups):
  location = HERE.parents[1] / "crates/tabdat-stats/tests/fixtures/bootstrap/draws.py"
  spec = importlib.util.spec_from_file_location("bootstrap_draw_oracle", location)
  oracle = importlib.util.module_from_spec(spec)
  spec.loader.exec_module(oracle)
  stream = oracle.stream(10001)
  zone = oracle.MASK - oracle.MASK % groups
  expected = []
  draws = []
  for replicate in range(1, 501):
    indices = []
    step = 0
    while len(indices) < groups:
      word = next(stream)
      step += 1
      assert step <= groups + 1024
      if word < zone:
        index = word % groups
        expected.append([replicate, len(indices) + 1, step, index, index + 1])
        indices.append(index)
    draws.append(indices)
  with path.open() as source:
    reader = csv.reader(source)
    assert next(reader) == ["replicate", "position", "step", "index0", "firm_id"]
    actual = [[int(v) for v in row] for row in reader]
  assert actual == expected, "draws differ from independent seed10001 stream"
  return draws


def refit(root, reference, records, replicate, indices):
  raw = root / "inputs" / f"copy{replicate:03}.asc"
  raw.write_text("\n".join(" ".join(records[i]) for i in indices) + "\n")
  output = root / "refits" / f"copy{replicate:03}"
  environment = os.environ.copy()
  environment.update(OMP_NUM_THREADS="1", OPENBLAS_NUM_THREADS="1", VECLIB_MAXIMUM_THREADS="1")
  result = subprocess.run(["Rscript", str(reference), str(raw), str(output), "bootstrap-point"],
                          capture_output=True, text=True, env=environment, timeout=300)
  (root / "logs" / f"copy{replicate:03}.log").write_text(result.stdout + result.stderr)
  assert result.returncode == 0, f"independent R copy{replicate} failed; see logs"
  with (output / "point.csv").open() as source:
    pairs = list(csv.DictReader(source))
  assert [v["field"] for v in pairs] == FIELDS
  values = [float(v["value"]) for v in pairs]
  assert all(math.isfinite(v) for v in values)
  assert max(abs(v) for v in values[-3:]) < 1e-8
  return [replicate] + values


def main():
  parser = argparse.ArgumentParser(description=__doc__)
  parser.add_argument("raw", type=Path)
  parser.add_argument("draws", type=Path)
  parser.add_argument("output", type=Path)
  parser.add_argument("--control", action="store_true", help="unhashed artificial source, not publisher validation")
  parser.add_argument("--jobs", type=int, default=4)
  parser.add_argument("--reps", type=int, default=500, help="2..500 prefix; only500 is the source acceptance run")
  args = parser.parse_args()
  assert 1 <= args.jobs <= 16 and 2 <= args.reps <= 500
  raw = args.raw.resolve()
  expected = raw_source(raw, original=not args.control)
  tokens = raw.read_text().split()
  assert len(tokens) % 25 == 0
  records = [tokens[i:i+25] for i in range(0, len(tokens), 25)]
  groups = len(records)
  assert 2 <= groups <= 346 and len(expected) == groups * 5
  draws = verified_draws(args.draws.resolve(), groups)
  root = args.output.resolve()
  root.mkdir(parents=True, exist_ok=False)
  for name in ["inputs", "refits", "logs"]:
    (root / name).mkdir()
  reference = root / "reference.R"
  shutil.copyfile(HERE / "normal_re_poisson_reference.R", reference)
  summary = root / "summary.R"
  shutil.copyfile(HERE / "native_normal_re_bootstrap_summary.R", summary)
  provenance = {"status": "in_progress", "source_sha256": hashlib.sha256(raw.read_bytes()).hexdigest(),
                "reference_sha256": hashlib.sha256(reference.read_bytes()).hexdigest(),
                "summary_sha256": hashlib.sha256(summary.read_bytes()).hexdigest(),
                "replications": args.reps, "groups": groups, "rows": groups * 5,
                "seed": 10001, "control": args.control,
                "draws": "independent_xoshiro256++_verified", "initializer": "independent_full_X_R_glm_t0",
                "information": "full_integrated_observed_PD_for_Newton_and_final",
                "per_copy_covariance": "not_exported;bootstrap_uses_empirical_joint_refits"}
  (root / "provenance.json").write_text(json.dumps(provenance, indent=2) + "\n")
  start = time.monotonic()
  results = []
  try:
    refit(root, reference, records, 0, list(range(groups)))
    with ThreadPoolExecutor(max_workers=args.jobs) as executor:
      futures = [executor.submit(refit, root, reference, records, rep + 1, indices)
                 for rep, indices in enumerate(draws[:args.reps])]
      try:
        for future in as_completed(futures):
          results.append(future.result())
      except BaseException:
        for future in futures:
          future.cancel()
        raise
    assert len(results) == args.reps
    pending = root / "fits.pending.csv"
    with pending.open("x", newline="") as target:
      writer = csv.writer(target)
      writer.writerow(["replicate"] + FIELDS)
      writer.writerows(sorted(results))
    result = subprocess.run(["Rscript", str(summary), str(pending),
                             str(root / "refits/copy000/point.csv"), str(raw),
                             str(root / "reference.pending.csv")], capture_output=True, text=True, timeout=90)
    (root / "logs/summary.log").write_text(result.stdout + result.stderr)
    assert result.returncode == 0, "independent R empirical summary failed"
    pending.rename(root / "fits.csv")
    (root / "reference.pending.csv").rename(root / "reference.csv")
  except BaseException as error:
    provenance.update(status="failed", error=str(error), elapsed_seconds=time.monotonic() - start)
    (root / "provenance.json").write_text(json.dumps(provenance, indent=2) + "\n")
    raise
  provenance.update(status="complete", elapsed_seconds=time.monotonic() - start)
  (root / "provenance.json").write_text(json.dumps(provenance, indent=2) + "\n")
  print(f"PASS{args.reps} independent R joint refits/{args.reps*groups} verified draws in{provenance['elapsed_seconds']:.3f}s")


if __name__ == "__main__":
  main()
