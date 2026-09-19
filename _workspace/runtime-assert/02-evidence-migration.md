# Bounded runtime `assert` migration evidence

Status: accepted bounded slice; PR #41 merged, branch cleanup complete, post-merge matrix pending

Producer: task owner, with pinned oracle evidence and independent runtime review

Consumers: reviewers and the next runtime maintainer

Boundary: pinned Python row-assertion contract → Rust-owned eager DuckDB
aggregate over the active relation

## Authority and contract inputs

The pinned authority is the clean sibling checkout `../tabdat-explore` at
commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, package `0.25.0`, Python
`3.13.3`, and `uv.lock` SHA-256
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`. The checkout
was clean and no dependency synchronization or source edits were performed.
The recovered contract is in [`01-contract.md`](01-contract.md).

Authoritative Python paths are `models.py:208-212,1244-1247`,
`parser.py:277-278,837-867`, `executor.py:964-969`,
`backend.py:1081-1110,2303-2627`, `tests/test_assert.py`,
`docs/commands/assert.md`, and `docs/language-semantics.md:57-60`.

## Oracle recovery evidence

The focused oracle module passed at the pinned revision:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_assert.py
10 passed in 0.86s
```

The direct parser case passed as `1 passed, 9 deselected in 0.28s`. The
runtime-focused selection passed as `6 passed, 4 deselected in 0.36s` across
the eager and lazy oracle cases. The Rust slice intentionally claims only the
eager local-Parquet path; lazy engines, CLI/JSON/MCP, and the documented
function-call forms remain deferred.

## Implementation evidence

The implementation is split across these commits:

- `7a2040a` — recovered the pinned contract;
- `7dc6b90` — added the owned expression AST, parser, runtime dispatch,
  null-aware SQL aggregate, typed result/errors, and six focused runtime tests;
- `5fc2c0c` — fixed the warnings-as-errors lint findings;
- `7778f5f` — recorded the migration evidence and aligned null/type
  diagnostics;
- `9a6f955` — mirrored safe numeric arithmetic semantics, unsigned guards, and
  first-unknown diagnostics, with regression tests.

The bounded parser accepts identifiers (including backticks), numeric/string/
`null` literals, unary minus, parentheses, arithmetic, and comparisons. The
runtime validates unknown names and expression domains before querying, quotes
identifiers, applies checked integer arithmetic and finite-to-NULL numeric
normalization, rejects unsafe unsigned subtraction/negation, compiles explicit
null comparisons to `IS NULL`/`IS NOT NULL`, and counts false or SQL-NULL
predicate values as failures. Empty relations return
`checked=0, failed=0`; failures use the exact
`assertion failed: {failed} of {checked} rows failed` diagnostic and preserve
active metadata.

Focused Rust evidence:

```text
cargo test --locked -p tabdat-language --test parser_contract assert_
2 passed
cargo test --locked -p tabdat-runtime --test assert_contract
7 passed
cargo test --locked -p tabdat-runtime assert_does_not_initialize_backend_for_a_new_session
1 passed
```

Final local workspace gates after the geiger target rebuild all passed:

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
git diff --check
```

The workspace test totals were root 1, language unit 42, parser contract 34,
runtime unit 19, assert contract 7, datasignature contract 6, and use contract
63, with no failures. `cargo deny check` and `cargo audit -D warnings` passed.
The metadata-driven all-package `cargo geiger` policy scan reported clean
first-party unsafe usage for the root, `tabdat-language`, and `tabdat-runtime`
packages (transitive inventory remains a warning signal only).

## Hosted acceptance and cleanup

Draft PR [#41](https://github.com/SaehwanPark/tabdat-explore-rs/pull/41) was
opened before implementation and squash-merged as
[`019ceb1`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/019ceb1c58aabf72b263bbe40c817d3153600096).
Its final evidence head was `2435381`; PR-head hosted acceptance was green:
[CI run 35417321384](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35417321384)
(Rust baseline and dependency/unsafe-code policy) and [runtime boundary run
35417321403](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35417321403)
(Linux runtime targets). The temporary `feat/runtime-assert` branch was removed
locally and remotely. Post-merge workflow links will be appended after the
documentation closeout push.

## Deferred scope

The Rust session has no `last_operation` field, so the Python success-side effect
is an explicit deferral. Lazy/materialized execution, function calls and
`e(sample)`, row-level failure diagnostics, formatting, CLI/REPL, JSON/MCP,
`by:` wrappers, broader expression grammar, and general relation APIs remain
out of scope. This artifact records a bounded eager implementation and does not
claim full Python `assert` parity.
