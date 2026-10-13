"""Capture calls from the pinned parser suite without changing its checkout.

Run with the oracle's existing Python environment, from any directory:
  PYTHONDONTWRITEBYTECODE=1 ../tabdat-explore/.venv/bin/python \
    tools/capture_parser_oracle.py ../tabdat-explore \
    crates/tabdat-language/tests/fixtures/python-parser.json
"""

import dataclasses
import hashlib
import json
from pathlib import Path
import subprocess
import sys

PIN = "16b45d9b66b0d80f32d4d220e84d81bc5180bdbe"
TREE = "601b236788872323af9277d2276a236154a0f129"
LOCK = "0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239"


def git(root, *args):
    return subprocess.check_output(["git", "-C", str(root), *args], text=True).strip()


def encode(value):
    if dataclasses.is_dataclass(value):
        return {"type": type(value).__name__, "fields": {
            field.name: encode(getattr(value, field.name))
            for field in dataclasses.fields(value)
        }}
    if isinstance(value, Path):
        return str(value)
    if isinstance(value, (list, tuple)):
        return [encode(item) for item in value]
    if isinstance(value, dict):
        return {key: encode(item) for key, item in value.items()}
    return value


def main():
    root = Path(sys.argv[1]).resolve()
    output = Path(sys.argv[2]).resolve()
    assert git(root, "rev-parse", "HEAD") == PIN
    assert git(root, "rev-parse", "HEAD^{tree}") == TREE
    assert not git(root, "status", "--porcelain"), "oracle must be clean"
    assert hashlib.sha256((root / "uv.lock").read_bytes()).hexdigest() == LOCK
    sys.path.insert(0, str(root / "src"))
    import pytest
    import tabdat.parser as parser
    from tabdat.errors import ParseError

    cases = {}
    original = parser.parse_command

    def capture(text):
        try:
            command = original(text)
        except ParseError as error:
            cases[text] = {"input": text, "error": str(error)}
            raise
        cases[text] = {"input": text, "command": encode(command)}
        return command

    parser.parse_command = capture
    status = pytest.main([str(root / "tests/test_parser.py"), "-q", "-p", "no:cacheprovider"])
    assert status == 0, "do not publish a corpus from failing oracle tests"
    assert not git(root, "status", "--porcelain"), "capture dirtied oracle"
    document = {"revision": PIN, "tree": TREE, "python": sys.version,
                "pytest": pytest.__version__, "cases": sorted(cases.values(), key=lambda c: c["input"])}
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(document, indent=2, ensure_ascii=False) + "\n")
    print(f"Captured {len(cases)} unique command inputs to {output}")


if __name__ == "__main__":
    main()
