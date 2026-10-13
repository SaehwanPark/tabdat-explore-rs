# Script compatibility evidence

PR [#195](https://github.com/SaehwanPark/tabdat-explore-rs/pull/195) completes a
bounded script compatibility slice, not whole-language parity or a Phase 3 gate.

## Pinned evidence

Oracle: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, lock digest in
[the baseline manifest](python-baseline.toml). Relevant authority is
`src/tabdat/script.py`, `tests/test_script.py`, script cases in `tests/test_cli.py`
and `docs/user-guide/scripting-and-json.md`.

`tools/capture_script_oracle.py` verified the pin/tree/lock/clean checkout and
ran the original **27 script tests** using Python 3.13.3 and pytest 9.0.3 in the
existing environment. This is recorded recovery evidence, not a fresh locked
Python environment claim. The capture contains **91 API calls** in
`crates/tabdat-language/tests/fixtures/python-script.json`. Synthetic paths
normalize only to filenames; command text, lines and diagnostic messages remain
exact. Seeds use decimal strings to avoid JSON integer range loss.

The initial replay found 25 differences. Python-compatible line boundaries,
trimming and splitting resolved 20. Five seed differences are explicitly
accepted as [MIG-0005](decisions.md#mig-0005-details); the original Python fixture
is unchanged. Replay asserts the exact Rust rejection for those five named
inputs and exact Python results/errors for all other calls. New differences
fail. Full Python integer syntax/range support is deliberately not claimed.

## Supported seed contract

`seed` accepts optional `+`/`-` followed by ASCII decimal digits in signed 64-bit
range, including zero, leading zeros and both endpoints. Fractions, underscores,
Unicode digits, base prefixes and overflow fail with a path/line-qualified
`seed expects an integer` error. The value is recorded in the current script
context, shared with nested `run` files, and discarded at the end of that run.
It does **not** seed an execution RNG today; deterministic stochastic behavior
must be specified and validated when an RNG-backed command is implemented.

## Acceptance and verification

- `python_script_oracle`: all 91 calls checked (86 exact agreements, five explicit
  rejections under MIG-0005), including Unicode/control line separators and
  whitespace, macro boundaries, conditions and directives.
- `script_contract`: comments/ordering, multiline SQL grouping and unterminated
  SQL errors, macro expansion, control flow, file errors, seed syntax/range and
  unchanged context after invalid parsing.
- `cli_json_use_count_contract`: canonical `use` → nested `run` → `count` stream;
  parent/child macro sharing, seed endpoints, skipped invalid inactive commands,
  clean exit, invalid seed/control/recursive inclusion errors and preservation
  of earlier emitted results. Existing cases cover child source locations,
  runtime failure, unsupported-output rejection before file mutation and
  execution order. Script execution is sequential, not whole-script atomic.

Workspace checks and their results are recorded in the PR thread. No statistical
estimator, RNG, native adapter or new runtime dependency is changed. Multiline
SQL grouping is language-tested; this slice does not claim a cross-language SQL
execution comparison, full parser AST parity, or unrestricted script parity.
