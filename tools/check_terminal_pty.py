"""Unix controlling-PTY acceptance check; external test tool, not runtime code.

Run after `cargo build --locked`:
  python3 tools/check_terminal_pty.py target/debug/tabdat-explore-rs

Checks real completion/editing/recall, input-only Ctrl-C, retained data/model,
EOF and canonical/echo terminal restoration. Does not test in-flight cancellation.
"""
import errno
import os
from pathlib import Path
import pty
import re
import select
import signal
import struct
import sys
import tempfile
import termios
import time
import fcntl

binary = str(Path(sys.argv[1]).resolve())
with tempfile.TemporaryDirectory(prefix="tabdat-editor-pty-") as directory:
    Path(directory, "data.csv").write_text("x,y\n0,1\n1,2\n2,4\n3,6\n")
    pid, fd = pty.fork()
    if pid == 0:
        os.chdir(directory)
        os.environ["TERM"] = "xterm-256color"
        os.execv(binary, [binary])
    status = None
    transcript = bytearray()
    # Provide a real window size instead of the zero-sized fresh PTY default.
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))

    def expect(marker):
        start = len(transcript)
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            if select.select([fd], [], [], 0.2)[0]:
                chunk = os.read(fd, 65536)
                assert chunk, "EOF before expected output"
                transcript.extend(chunk)
                if b"\x1b[6n" in chunk:
                    os.write(fd, b"\x1b[1;1R")
            text = re.sub(rb"\x1b\[[0-?]*[ -/]*[@-~]", b"", bytes(transcript[start:]))
            position = text.find(marker)
            if position >= 0 and b"tabdat> " in text[position + len(marker):]:
                return
        raise AssertionError(f"missing output/prompt after {marker!r}: {bytes(transcript)!r}")

    def command(text, marker):
        os.write(fd, text)
        expect(marker)

    try:
        expect(b"TabDat")
        command(b"use data.csv\n", b"Loaded 4 rows, 2 columns")
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
        assert status is not None and os.waitstatus_to_exitcode(status) == 0, "EOF did not exit cleanly"
        flags = termios.tcgetattr(fd)[3]
        assert flags & termios.ICANON and flags & termios.ECHO, "terminal mode not restored"
        assert "cancelled.csv" not in os.listdir(directory)
        assert b"Error:" not in transcript, bytes(transcript)
        print("PTY passed: command/help completion, editing, recall, input cancellation, retained data/model, EOF, terminal restoration")
    finally:
        os.close(fd)
        if status is None:
            try:
                os.kill(pid, signal.SIGKILL)
                os.waitpid(pid, 0)
            except ProcessLookupError:
                pass
