"""Seed the external fuzz corpus with pinned parser cases and hostile syntax."""
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parents[1]
corpus = root / "fuzz/corpus/syntax"
corpus.mkdir(parents=True, exist_ok=True)
cases = json.loads((root / "crates/tabdat-language/tests/fixtures/python-parser.json").read_text())["cases"]
inputs = [case["input"] for case in cases]
inputs.extend([
    "generate x = " + "(" * 20_000 + "1" + ")" * 20_000,
    "assert " + "-" * 20_000 + "1",
    "generate x = " + "f(" * 10_000 + "1" + ")" * 10_000,
    "generate x = " + "+".join(["1"] * 1_000),
    "by x: " * 100 + "count",
    "bayes: " * 100 + "regress y x",
    "generate x = " + "-" * 255 + "1",
    "assert " + "(" * 127 + "1" + ")" * 127,
    'sql """\nselect 1\n""" into example',
    "let threshold = 3\nif $threshold > 2\ncount\nelse\nstatus\nend",
])
for value in inputs:
    data = value.encode()
    (corpus / hashlib.sha256(data).hexdigest()).write_bytes(data)
print(f"Seeded {len(inputs)} inputs into {corpus}")
