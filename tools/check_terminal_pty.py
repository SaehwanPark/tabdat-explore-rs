"""Unix controlling-PTY acceptance and opt-in local latency measurements.

Run after `cargo build --locked`:
  python3 tools/check_terminal_pty.py target/debug/tabdat-explore-rs
  python3 tools/check_terminal_pty.py target/debug/tabdat-explore-rs --measure

External development tool, not runtime code or a performance threshold.
Does not test in-flight cancellation or measure physical display-paint latency.
"""
import argparse
import errno
import fcntl
import json
import math
import os
from pathlib import Path
import platform
import pty
import re
import select
import signal
import statistics
import struct
import tempfile
import termios
import time


def options(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("--measure", action="store_true", help="emit validated local timing JSON instead of the acceptance message")
    parser.add_argument("--repetitions", type=int, help="1..50 measured cycles after one excluded warmup (default 7)")
    parser.add_argument("--label", help="user-supplied source/profile context, not verified")
    args = parser.parse_args(argv)
    if not args.measure and (args.repetitions is not None or args.label is not None):
        parser.error("--repetitions and --label require --measure")
    args.repetitions = 7 if args.repetitions is None else args.repetitions
    args.label = "unspecified" if args.label is None else args.label
    if not 1 <= args.repetitions <= 50:
        parser.error("--repetitions must be 1..50")
    args.binary = args.binary.resolve()
    if not args.binary.is_file() or not os.access(args.binary, os.X_OK):
        parser.error("binary must be an existing executable file")
    return args


def timing_summary(samples):
    if not samples or any(not math.isfinite(value) or value < 0 for value in samples):
        raise ValueError("timings must be nonempty, finite and nonnegative")
    return {"samples_ms": list(samples), "min_ms": min(samples),
            "median_ms": statistics.median(samples), "max_ms": max(samples)}


def numeric_rows(text, header, count):
    lines = text.decode("utf-8").splitlines()
    start = lines.index(header) + 1
    rows = [line.split("\t") for line in lines[start:start + count]]
    if len(rows) != count:
        raise ValueError("missing result rows")
    return rows


def close(actual, expected):
    value = float(actual)
    # Exact four-row OLS algebra gives intercept=7/10 and slope=17/10.
    # This is a fixture health check, not a new statistical tolerance contract.
    if not math.isfinite(value) or abs(value - expected) > 1e-10:
        raise ValueError(f"fixture value {actual} != {expected}; absolute tolerance 1e-10")


def validate_measurement(case, text):
    if b"Error:" in text:
        raise ValueError("command emitted an error")
    if case == "load":
        if b"Loaded 4 rows, 2 columns from data.csv\n" not in text:
            raise ValueError("wrong load result")
    elif case == "count":
        if b"\n4 rows\n" not in text:
            raise ValueError("wrong count")
    elif case == "summarize":
        rows = numeric_rows(text, "Variable\tN\tMean\tStd. dev.\tMin\tMax", 2)
        for row, name, mean in zip(rows, ["x", "y"], [1.5, 3.25]):
            if row[:2] != [name, "4"]:
                raise ValueError("wrong finite sample/order")
            close(row[2], mean)
    elif case == "regress":
        if b"N = 4; residual df = 2\n" not in text or b"Estimator: ols; covariance: nonrobust; intercept: true\n" not in text:
            raise ValueError("wrong model sample/design")
        rows = numeric_rows(text, "Parameter\tCoefficient\tStd. error\tStatistic\tP(two-sided)", 2)
        for row, name, value in zip(rows, ["intercept", "x"], [0.7, 1.7]):
            if row[0] != name:
                raise ValueError("wrong coefficient order")
            close(row[1], value)
    elif case == "predict":
        if b"Predicted: 4 rows, 3 columns\n" not in text:
            raise ValueError("wrong prediction result")
    elif case == "preview":
        rows = numeric_rows(text, "x\ty\tfitted", 4)
        for index, (row, y) in enumerate(zip(rows, [1, 2, 4, 6])):
            if len(row) != 3:
                raise ValueError("wrong prediction schema")
            close(row[0], index)
            close(row[1], y)
            close(row[2], 0.7 + 1.7 * index)
    else:
        raise ValueError(f"unknown measurement case: {case}")


def measure_cycles(command, repetitions):
    cases = [
        ("load", b"use data.csv\n", b"Loaded 4 rows, 2 columns"),
        ("count", b"count\n", b"4 rows"),
        ("summarize", b"summarize x y\n", b"y\t4\t3.25\t"),
        ("regress", b"regress y x\n", b"Parameter\tCoefficient"),
        ("predict", b"predict fitted\n", b"Predicted: 4 rows, 3 columns"),
    ]
    samples = {case: [] for case, _, _ in cases}
    for cycle in range(repetitions + 1):
        for case, source, marker in cases:
            start = time.monotonic_ns()
            text = command(source, marker)
            elapsed = (time.monotonic_ns() - start) / 1_000_000
            validate_measurement(case, text)
            if cycle:
                samples[case].append(elapsed)
        # Validate actual current predictions outside all command timers. The
        # next cycle reloads the same two-column fixture, avoiding schema growth.
        validate_measurement("preview", command(b"head 4\n", b"(4 rows shown)"))
    return {case: timing_summary(values) for case, values in samples.items()}


def validate_acceptance_errors(transcript):
    errors = re.findall(rb"(?:^|\n)Error: ([^\r\n]*)", bytes(transcript))
    if (len(errors) != 2 or errors[0] != b"rename unknown variable: absent"
            or not errors[1].endswith(b"/analysis files/partial schema.td:3: unknown command: unknown_command")):
        raise AssertionError(f"unexpected acceptance diagnostics: {errors!r}")


def run(args):
    binary = str(args.binary)
    with tempfile.TemporaryDirectory(prefix="tabdat-editor-pty-") as directory:
        Path(directory, "data.csv").write_text("x,y\n0,1\n1,2\n2,4\n3,6\n")
        Path(directory, "data-extra.csv").write_text("x,y\n0,1\n1,2\n2,4\n3,6\n")
        Path(directory, "quoted.csv").write_text("annual income,annual cost,变量 名\n0,1,0\n1,2,1\n2,4,2\n3,6,3\n", encoding="utf-8")
        Path(directory, "cohort_probe").write_text("a same-name file must not override the registered table")
        Path(directory, "cohort_dir").mkdir()
        Path(directory, "cohort_dir", "input.csv").write_text("x,y\n0,1\n1,2\n2,4\n3,6\n")
        Path(directory, "analysis files").mkdir()
        Path(directory, "analysis files", "semi, 雪.csv").write_text("x;y\n0;1\n1;2\n2;4\n3;6\n")
        Path(directory, "analysis files", "child script.td").write_text("count\n")
        Path(directory, "analysis files", "partial schema.td").write_text("rename refreshed_column scripted_column\nsql SELECT * FROM active into scripted_cohort\nunknown_command\n")
        pid, fd = pty.fork()
        if pid == 0:
            os.chdir(directory)
            os.environ["TERM"] = "xterm-256color"
            try:
                os.execv(binary, [binary])
            except OSError:
                os._exit(127)
        status = None
        transcript = bytearray()
        timings = None
        def expect(marker):
            start = len(transcript)
            deadline = time.monotonic() + 15
            while time.monotonic() < deadline:
                if select.select([fd], [], [], 0.2)[0]:
                    chunk = os.read(fd, 65536)
                    if not chunk:
                        raise AssertionError("EOF before expected output")
                    transcript.extend(chunk)
                    if b"\x1b[6n" in chunk:
                        os.write(fd, b"\x1b[1;1R")
                text = re.sub(rb"\x1b\[[0-?]*[ -/]*[@-~]", b"", bytes(transcript[start:])).replace(b"\r", b"")
                position = text.find(marker)
                if position >= 0 and b"tabdat> " in text[position + len(marker):]:
                    return text
            raise AssertionError(f"missing output/prompt after {marker!r}: {bytes(transcript)!r}")

        def command(text, marker):
            os.write(fd, text)
            return expect(marker)

        try:
            # A fresh PTY otherwise has a zero-sized window, unlike an actual terminal.
            fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
            expect(b"TabDat")
            command(b"use data.c\t\n", b"Loaded 4 rows, 2 columns")
            command(b"sql SELECT * FROM active WHERE x >= 2 into cohort_probe\n", b'Created and activated table "cohort_probe": 2 rows, 2 columns')
            command(b"use data.csv\n", b"Loaded 4 rows, 2 columns")
            command(b"use cohort_pro\t\n", b'Activated table "cohort_probe": 2 rows, 2 columns')
            command(b"count\n", b"2 rows")
            command(b"sql SELECT * FROM active into cohort_dir\n", b'Created and activated table "cohort_dir": 2 rows, 2 columns')
            command(b"use data.csv\n", b"Loaded 4 rows, 2 columns")
            command(b"use cohort_d\t\n", b'Activated table "cohort_dir": 2 rows, 2 columns')
            command(b"use ./cohort_dir/in\t\n", b"Loaded 4 rows, 2 columns")
            command(b"use data\t\t", b"data-extra.csv")
            command(b"\x03", b"Input cancelled.")
            command(b"use 'ana\tsemi\t, delimiter(';')\n", b"Loaded 4 rows, 2 columns")
            # Complete before a preexisting closing quote and option suffix.
            line = b'use "analysis files/se", delimiter(";")'
            suffix_length = len(b'", delimiter(";")')
            command(line + b"\x1b[D" * suffix_length + b"\t" + b"\x1b[C" * suffix_length + b"\n", b"Loaded 4 rows, 2 columns")
            command(b"run 'ana\tchild\t\n", b"Executed 1 commands")
            command(b"generate probe_column = x\n", b"Generated: 4 rows, 3 columns")
            command(b"summarize probe_c\t\n", b"probe_column\t4\t1.5\t")
            fit = command(b"regress y probe_c\t\n", b"probe_column\t")
            rows = numeric_rows(fit, "Parameter\tCoefficient\tStd. error\tStatistic\tP(two-sided)", 2)
            for row, name, value in zip(rows, ["intercept", "probe_column"], [0.7, 1.7]):
                if row[0] != name:
                    raise ValueError("wrong completed predictor order")
                close(row[1], value)
            command(b"rename probe_column refreshed_column\n", b"Renamed: 4 rows, 3 columns")
            command(b"rename absent ignored\n", b"Error:")
            command(b"summarize refreshed_c\t\n", b"refreshed_column\t4\t1.5\t")
            command(b"run 'analysis files/partial schema.td'\n", b"partial schema.td:3:")
            command(b"use scripted_c\t\n", b'Activated table "scripted_cohort": 4 rows, 3 columns')
            command(b"summarize scripted_c\t\n", b"scripted_column\t4\t1.5\t")
            command(b"drop scripted_c\t\n", b"Dropped: 4 rows, 2 columns")
            command(b"re\t\t", b"reshape")  # ambiguous prefix lists alternatives
            command(b"\x03", b"Input cancelled.")
            command(b"re\tg\t y x\n", b"Parameter\tCoefficient")
            command(b"  SuMm y\x1b[D\x1b[D\t\n", b"y\t4\t3.25\t")
            command(b"help summ\t\n", b"How to invoke:")
            command(b"?summ\t\n", b"How to invoke:")
            command(b"count\n", b"4 rows")
            command(b"\x1b[A\n", b"4 rows")  # recall last submitted command
            command(b"counx\x7ft\n", b"4 rows")  # edit a typo before submission
            command(b"replace y = 0\x03", b"Input cancelled.")
            command(b"export cancelled.csv\x03", b"Input cancelled.")
            command(b"\x1b[A\n", b"4 rows")  # cancelled lines must not enter history
            command(b"predict fitted\n", b"Predicted: 4 rows, 3 columns")
            command(b"summarize y\n", b"y\t4\t3.25\t")
            # Registered append/join targets complete without filesystem candidates,
            # so the same-name file must not override the registered table here.
            command(b"use data.csv\n", b"Loaded 4 rows, 2 columns")
            command(b"sql SELECT x, y AS lookup_y FROM active WHERE x <= 1 into lookup_probe\n", b'Created and activated table "lookup_probe": 2 rows, 2 columns')
            command(b"use data.csv\n", b"Loaded 4 rows, 2 columns")
            command(b"append cohort_pro\t\n", b"Appended: 6 rows, 2 columns")
            appended = command(b"head 6\n", b"(6 rows shown)")
            rows = numeric_rows(appended, "x\ty", 6)
            for row, values in zip(rows, [[0, 1], [1, 2], [2, 4], [3, 6], [2, 4], [3, 6]]):
                if len(row) != 2:
                    raise ValueError("wrong appended schema")
                for cell, value in zip(row, values):
                    close(cell, value)
            # A typed join suffix after the target survives completion and executes.
            command(b"join lookup_pro\t on x, how=left suffix(_s)\n", b"Joined: 6 rows, 3 columns")
            command(b"summarize lookup_y\n", b"lookup_y\t2\t1.5\t")
            command(b"use quoted.csv\n", b"Loaded 4 rows, 3 columns")
            command(b"summarize `annual i\t\n", b"annual income\t4\t1.5\t")
            command("summarize `变量\t\n".encode(), "变量 名\t4\t1.5\t".encode())
            line = b"summarize `annual i`, detail"
            suffix_length = len(b"`, detail")
            command(line + b"\x1b[D" * suffix_length + b"\t" + b"\x1b[C" * suffix_length + b"\n", b"annual income\t4\t1.5\t")
            fit = command(b"regress `annual c\t `annual i\t, robust\n", b"annual income\t")
            rows = numeric_rows(fit, "Parameter\tCoefficient\tStd. error\tStatistic\tP(two-sided)", 2)
            for row, name, value in zip(rows, ["intercept", "annual income"], [0.7, 1.7]):
                if row[0] != name:
                    raise ValueError("wrong quoted predictor order")
                close(row[1], value)
            command(b"rename `annual income` `updated income`\n", b"Renamed: 4 rows, 3 columns")
            command(b"summarize `updated i\t\n", b"updated income\t4\t1.5\t")
            command(b"rename `updated income` `annual income`\n", b"Renamed: 4 rows, 3 columns")
            command(b"predict fitted\n", b"Predicted: 4 rows, 4 columns")
            command(b"summarize fitted\n", b"fitted\t4\t3.25\t")
            if args.measure:
                timings = measure_cycles(command, args.repetitions)
            os.write(fd, b"\x04")
            deadline = time.monotonic() + 15
            while time.monotonic() < deadline:
                ended, current = os.waitpid(pid, os.WNOHANG)
                if ended:
                    status = current
                    break
                if select.select([fd], [], [], 0.2)[0]:
                    try:
                        transcript.extend(os.read(fd, 65536))
                    except OSError as error:
                        if error.errno != errno.EIO:
                            raise
            if status is None or os.waitstatus_to_exitcode(status) != 0:
                raise AssertionError("EOF did not exit cleanly")
            flags = termios.tcgetattr(fd)[3]
            if not (flags & termios.ICANON and flags & termios.ECHO):
                raise AssertionError("terminal mode not restored")
            if "cancelled.csv" in os.listdir(directory):
                raise AssertionError("cancelled export created a file")
            validate_acceptance_errors(transcript)
        finally:
            os.close(fd)
            if status is None:
                try:
                    os.kill(pid, signal.SIGKILL)
                    os.waitpid(pid, 0)
                except ProcessLookupError:
                    pass
    return timings


def main():
    args = options()
    timings = run(args)
    if args.measure:
        report = {
            "measurement_schema_version": 1,
            "tool": "check_terminal_pty.py",
            "binary": str(args.binary),
            "label": args.label,
            "host": {"system": platform.system(), "release": platform.release(),
                     "machine": platform.machine(), "python": platform.python_version()},
            "terminal": {"TERM": "xterm-256color", "rows": 24, "columns": 80},
            "fixture": {"rows": 4, "csv": "x,y\n0,1\n1,2\n2,4\n3,6\n"},
            "repetitions": args.repetitions,
            "excluded_warmup_cycles": 1,
            "preconditioning": "existing completion/editing/cancellation/model-retention acceptance flow runs first in the same process",
            "timing_scope": "parent monotonic submit-to-result-and-prompt observation in a warmed persistent controlling PTY; includes parsing/execution/rendering/transport/scheduling and observer ANSI/marker processing, excludes numeric validation and preview",
            "cases": timings,
        }
        print(json.dumps(report, allow_nan=False))
    else:
        print("PTY passed: command/help/read-path/registered-name/append-join-target/bare-and-backtick-column completion, editing, recall, input cancellation, retained data/model, EOF, terminal restoration")


if __name__ == "__main__":
    main()
