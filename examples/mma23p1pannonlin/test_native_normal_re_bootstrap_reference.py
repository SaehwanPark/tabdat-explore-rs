"""Reference-input and no-partial-publication controls; no mocked statistical claims."""
import csv
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import native_normal_re_bootstrap_reference as reference

ROOT = Path(__file__).resolve().parents[2]


def draws(path, groups):
  spec = importlib.util.spec_from_file_location("test_draws", ROOT / "crates/tabdat-stats/tests/fixtures/bootstrap/draws.py")
  module = importlib.util.module_from_spec(spec)
  spec.loader.exec_module(module)
  stream = module.stream(10001)
  zone = module.MASK - module.MASK % groups
  with path.open("w", newline="") as output:
    writer = csv.writer(output)
    writer.writerow(["replicate", "position", "step", "index0", "firm_id"])
    for rep in range(1, 501):
      position = step = 0
      while position < groups:
        word = next(stream)
        step += 1
        if word < zone:
          position += 1
          writer.writerow([rep, position, step, word % groups, word % groups + 1])


class ReferenceControls(unittest.TestCase):
  def setUp(self):
    self.temp = tempfile.TemporaryDirectory()
    self.addCleanup(self.temp.cleanup)
    self.root = Path(self.temp.name)
    self.draws = self.root / "draws.csv"
    draws(self.draws, 40)
    self.output = self.root / "output"
    self.args = ["reference", str(ROOT / "tests/fixtures/mma23_gamma_re_bootstrap/source.asc"),
                 str(self.draws), str(self.output), "--control", "--reps", "2", "--jobs", "1"]

  def test_entire_stream_and_tampered_draws(self):
    self.assertEqual([len(v) for v in reference.verified_draws(self.draws, 40)], [40] * 500)
    original = self.draws.read_text()
    rows = list(csv.reader(original.splitlines()))
    for column in range(5):
      changed = [row.copy() for row in rows]
      changed[2][column] = str(int(changed[2][column]) + 1)
      with self.draws.open("w", newline="") as output:
        csv.writer(output).writerows(changed)
      with self.assertRaises(AssertionError):
        reference.verified_draws(self.draws, 40)
    self.draws.write_text(original)
    self.assertEqual(len(reference.verified_draws(self.draws, 40)), 500)

  def test_output_no_clobber(self):
    self.output.mkdir()
    sentinel = self.output / "sentinel"
    sentinel.write_bytes(b"unchanged\n")
    with patch("sys.argv", self.args), self.assertRaises(FileExistsError):
      reference.main()
    self.assertEqual(list(self.output.iterdir()), [sentinel])
    self.assertEqual(sentinel.read_bytes(), b"unchanged\n")

  def test_failed_copy_never_publishes_partial_success(self):
    def fail_copy(root, script, records, rep, indices):
      if rep == 1:
        raise RuntimeError("failed copied fit")
      return [rep] + [0.] * 8
    with patch("sys.argv", self.args), patch.object(reference, "refit", side_effect=fail_copy):
      with self.assertRaisesRegex(RuntimeError, "failed copied fit"):
        reference.main()
    self.assert_failed()

  def test_failed_summary_never_publishes_fit_or_reference(self):
    def fake_fit(root, script, records, rep, indices):
      return [rep] + [0.] * 8
    failed = subprocess.CompletedProcess(["Rscript"], 1, "", "failed summary")
    with patch("sys.argv", self.args), patch.object(reference, "refit", side_effect=fake_fit), patch.object(reference.subprocess, "run", return_value=failed):
      with self.assertRaisesRegex(AssertionError, "empirical summary failed"):
        reference.main()
    self.assert_failed()
    self.assertTrue((self.output / "fits.pending.csv").exists())

  def assert_failed(self):
    self.assertFalse((self.output / "fits.csv").exists())
    self.assertFalse((self.output / "reference.csv").exists())
    provenance = json.loads((self.output / "provenance.json").read_text())
    self.assertEqual(provenance["status"], "failed")
    self.assertEqual(provenance["replications"], 2)


if __name__ == "__main__":
  unittest.main()
