"""Capture pinned script API calls without changing the oracle checkout.

Run using the oracle's existing Python environment, with bytecode disabled:
  PYTHONDONTWRITEBYTECODE=1 ../tabdat-explore/.venv/bin/python \
    tools/capture_script_oracle.py ../tabdat-explore \
    crates/tabdat-language/tests/fixtures/python-script.json
"""
import dataclasses
import functools
import hashlib
import inspect
import json
from pathlib import Path
import sys

from capture_parser_oracle import PIN, TREE, LOCK, git


def encode(value):
    if isinstance(value, Path):
        # Source paths are synthetic test locations; preserve the filename, not
        # pytest's temporary parent. Line/message/text are never normalized.
        return value.name
    if dataclasses.is_dataclass(value):
        fields = {field.name: encode(getattr(value, field.name))
                  for field in dataclasses.fields(value)}
        if type(value).__name__ in {"SeedDirective", "ScriptContext"}:
            key = "value" if type(value).__name__ == "SeedDirective" else "seed"
            if fields[key] is not None:
                fields[key] = str(fields[key])
        return {"type": type(value).__name__, "fields": fields}
    if isinstance(value, (tuple, list)):
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
    import tabdat.script as script

    cases = {}

    def wrap(name):
        original = getattr(script, name)

        @functools.wraps(original)
        def capture(*args, **kwargs):
            arguments = inspect.signature(original).bind(*args, **kwargs)
            arguments.apply_defaults()
            inputs = encode(dict(arguments.arguments))
            case = {"operation": name, "inputs": inputs}
            if name == "read_script":
                path = arguments.arguments["path"]
                case["file_text"] = path.read_text() if path.is_file() else None
            key = json.dumps(case, sort_keys=True, ensure_ascii=False)
            try:
                result = original(*args, **kwargs)
            except script.ScriptError as error:
                case["error"] = {"path": encode(error.path), "line": error.line,
                                 "message": error.message}
                cases[key] = case
                raise
            case["result"] = encode(result)
            cases[key] = case
            return result

        setattr(script, name, capture)

    for name in ["parse_script", "expand_script_macros", "parse_script_directive",
                 "parse_control_flow_directive", "evaluate_script_condition", "read_script"]:
        wrap(name)
    status = pytest.main([str(root / "tests/test_script.py"), "-q", "-p", "no:cacheprovider"])
    assert status == 0, "do not publish a corpus from failing oracle tests"

    # Source-established boundary probes supplement the original ASCII/LF suite.
    for separator in ["\n", "\r\n", "\r", "\v", "\f", "\x1c", "\x1d", "\x1e",
                      "\x85", "\u2028", "\u2029"]:
        script.parse_script(f"# comment{separator}status{separator}count{separator}")
        script.parse_script(f'sql """{separator}select 1{separator}"""{separator}count')
    path = Path("boundary.td")
    context = script.ScriptContext(macros={"x": "value", "_a": "alpha"})
    for text in ["$x $$x $1 $_a $x界", "literal $", "'$x'", "$x$x", "$missing"]:
        try:
            script.expand_script_macros(text, context, path=path, line=3)
        except script.ScriptError:
            pass
    for text in ["seed +1", "seed -1", "seed 1_000", "seed ٠١٢", "seed " + "9" * 80,
                 "seed 9223372036854775808", "seed -9223372036854775809",
                 "seed 1__0", "seed 1.0", "let x = new", "let y = $x", "let y = a=b",
                 "let = x", "let y =", "let 1y = x", "seed\x1c42"]:
        try:
            script.parse_script_directive(text, context, path=path, line=4)
        except script.ScriptError:
            pass
    for text in ["if\ttrue", "IF true", "else extra", "end extra", "if", "if false",
                 "if a == a", "if a != A", "if\x1ctrue"]:
        try:
            script.parse_control_flow_directive(text, path=path, line=5)
        except script.ScriptError:
            pass
    for text in ["a == a == a", "a != a == a", " == a", "a != ", "\x1ctrue\x1c"]:
        try:
            script.evaluate_script_condition(text, path=path, line=6)
        except script.ScriptError:
            pass
    assert not git(root, "status", "--porcelain"), "capture dirtied oracle"
    output.parent.mkdir(parents=True, exist_ok=True)
    document = {"revision": PIN, "tree": TREE, "python": sys.version,
                "pytest": pytest.__version__, "path_normalization": "synthetic filename only",
                "seed_representation": "canonical decimal string (no integer range loss)",
                "cases": [cases[key] for key in sorted(cases)]}
    output.write_text(json.dumps(document, indent=2, ensure_ascii=False) + "\n")
    print(f"Captured {len(cases)} script calls to {output}")


if __name__ == "__main__":
    main()
