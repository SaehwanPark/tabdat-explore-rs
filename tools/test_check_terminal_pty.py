"""Deterministic helper/failure tests; real editor acceptance uses the PTY tool itself."""
import contextlib
import io
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import check_terminal_pty as probe


RESULTS = {
    b"use data.csv\n": b"Loaded 4 rows, 2 columns from data.csv\n",
    b"count\n": b"\n4 rows\n",
    b"summarize x y\n": b"Variable\tN\tMean\tStd. dev.\tMin\tMax\nx\t4\t1.5\t1\t0\t3\ny\t4\t3.25\t2\t1\t6\n",
    b"regress y x\n": b"Estimator: ols; covariance: nonrobust; intercept: true\nN = 4; residual df = 2\nParameter\tCoefficient\tStd. error\tStatistic\tP(two-sided)\nintercept\t0.7\t0.1\t7\t0.01\nx\t1.7\t0.1\t17\t0.01\n",
    b"predict fitted\n": b"Predicted: 4 rows, 3 columns\n",
    b"head 4\n": b"x\ty\tfitted\n0\t1\t0.7\n1\t2\t2.4\n2\t4\t4.1\n3\t6\t5.8\n",
}


class ProbeTests(unittest.TestCase):
    def test_arguments_bound_cost_and_reject_unused_or_invalid_inputs(self):
        binary = sys.executable
        self.assertFalse(probe.options([binary]).measure)
        args = probe.options([binary, "--measure", "--repetitions", "2", "--label", "smoke"])
        self.assertEqual((args.repetitions, args.label), (2, "smoke"))
        for extra in [["--measure", "--repetitions", "0"], ["--measure", "--repetitions", "51"],
                      ["--measure", "--repetitions", "bad"], ["--label", "ignored"],
                      ["--repetitions", "7"], ["--unknown"]]:
            with self.subTest(extra=extra), contextlib.redirect_stderr(io.StringIO()):
                with self.assertRaises(SystemExit) as error:
                    probe.options([binary, *extra])
                self.assertEqual(error.exception.code, 2)
        with contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit):
            probe.options(["/missing-binary-for-pty-test"])

    def test_timing_summary_retains_order_and_rejects_invalid_samples(self):
        self.assertEqual(probe.timing_summary([3.0, 0.0, 2.0, 1.0]), {
            "samples_ms": [3.0, 0.0, 2.0, 1.0], "min_ms": 0.0, "median_ms": 1.5, "max_ms": 3.0,
        })
        for values in [[], [-1.0], [float("nan")], [float("inf")]]:
            with self.subTest(values=values), self.assertRaises(ValueError):
                probe.timing_summary(values)

    def test_cycles_exclude_first_warmup_and_reset_fixture_before_each_sample(self):
        seen = []
        ticks = iter(value for index in range(15) for value in (
            index * 100_000_000, index * 100_000_000 + (index + 1) * 1_000_000))

        def command(source, _):
            seen.append(source)
            return RESULTS[source]

        with patch.object(probe.time, "monotonic_ns", side_effect=lambda: next(ticks)):
            report = probe.measure_cycles(command, 2)
        self.assertEqual(seen, list(RESULTS) * 3)
        self.assertEqual(report["load"]["samples_ms"], [6.0, 11.0])
        self.assertEqual(report["predict"]["samples_ms"], [10.0, 15.0])
        self.assertEqual(set(report), {"load", "count", "summarize", "regress", "predict"})
        self.assertTrue(all(len(value["samples_ms"]) == 2 for value in report.values()))

    def test_invalid_values_and_prediction_rows_fail_instead_of_counting_timings(self):
        for case, text in [
            ("load", b"Loaded 0 rows, 2 columns from data.csv\n"),
            ("count", b"\n3 rows\n"),
            ("summarize", RESULTS[b"summarize x y\n"].replace(b"3.25", b"NaN")),
            ("regress", RESULTS[b"regress y x\n"].replace(b"0.7", b"0.9")),
            ("regress", RESULTS[b"regress y x\n"].replace(b"N = 4", b"N = 3")),
            ("preview", RESULTS[b"head 4\n"].replace(b"5.8", b"5.9")),
            ("predict", b"Error: failed\nPredicted: 4 rows, 3 columns\n"),
        ]:
            with self.subTest(case=case), self.assertRaises(ValueError):
                probe.validate_measurement(case, text)

        def bad_preview(source, _):
            return RESULTS[source].replace(b"5.8", b"5.9")

        with self.assertRaises(ValueError):
            probe.measure_cycles(bad_preview, 1)

    def test_acceptance_allows_only_the_two_expected_recovery_errors(self):
        expected = (b"Error: rename unknown variable: absent\r\n"
                    b"Error: /temporary/analysis files/partial schema.td:2: unknown command: unknown_command\r\n")
        probe.validate_acceptance_errors(expected)
        for text in [b"", expected.splitlines(keepends=True)[0],
                     expected + b"Error: additional failure\r\n",
                     expected.replace(b"absent", b"other"),
                     expected.replace(b"schema.td:2:", b"schema.td:3:"),
                     b"\r\n".join(reversed(expected.splitlines()))]:
            with self.subTest(text=text), self.assertRaises(AssertionError):
                probe.validate_acceptance_errors(text)

    def test_failed_child_emits_no_report_and_removes_temporary_fixture(self):
        binary = shutil.which("false")
        if binary is None:
            self.skipTest("Unix false executable unavailable")
        with tempfile.TemporaryDirectory() as root:
            result = subprocess.run([
                sys.executable, str(Path(probe.__file__).resolve()), binary,
                "--measure", "--repetitions", "1",
            ], capture_output=True, timeout=20, env={**os.environ, "TMPDIR": root})
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(result.stdout, b"")
            self.assertEqual(list(Path(root).iterdir()), [])


if __name__ == "__main__":
    unittest.main()
