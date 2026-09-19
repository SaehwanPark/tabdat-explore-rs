# Bounded eager-runtime `drop` migration evidence

Status: implementation complete on `feat/runtime-drop` at `9a90db8`; hosted
acceptance, merge, and branch cleanup remain pending.

Boundary: pinned Python explicit-varlist `drop` contract → Rust-owned eager
DuckDB complement projection over the active local-Parquet relation.

## Authority and oracle evidence

The clean sibling checkout `/Volumes/research/gitrepos/tabdat-explore` is pinned
to revision `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, Python `3.13.3`, with `uv.lock`
SHA-256 `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
The corrected lock hash is recorded in [`01-contract.md`](01-contract.md);
the authoritative Python paths and bounded deviations are listed there.

Focused oracle selections passed:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider tests/test_parser.py -k drop
2 passed, 487 deselected in 0.80s

PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider tests/test_executor.py -k drop
15 passed, 402 deselected in 3.03s

PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider tests/test_cli.py -k drop_predicate
2 passed, 181 deselected in 0.55s
```

## Implementation evidence

The branch commits are:

- `d2d7389` — complete the pinned drop contract and bounded scope;
- `d0501bb` — add typed parser/runtime drop projection and contract tests;
- `d5c910d` — align deferred-predicate syntax diagnostics;
- `2e6a28b` — satisfy the warnings-as-errors Clippy gate;
- `9a90db8` — add embedded-quote SQL coverage, tokenizer-error coverage, and
  mismatched-active-relation failure evidence.

The parser accepts case-insensitive explicit varlists, quoted/backtick names,
and exact empty/assignment/option/mixed-condition diagnostics. Predicate syntax
is recognized but execution remains explicitly deferred; duplicate `if` and
trailing-operator diagnostics are retained before deferral. The runtime validates
all names, computes the schema-order complement, rejects zero-column results,
quotes identifiers, stages in `__tabdat_next`, and publishes transactionally.

Focused Rust evidence:

```text
cargo test --locked -p tabdat-language --test parser_contract drop
2 passed
cargo test --locked -p tabdat-runtime --test drop_contract
9 passed
cargo test --locked -p tabdat-runtime tests::failed_drop_keeps_the_published_dataset_metadata
1 passed
```

Final local baseline at `9a90db8` passed:

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
git diff --check
```

Local policy checks passed on the implementation head before the final
evidence-only test hardening: `cargo deny check`, `cargo audit -D warnings`,
and the metadata-driven all-package `cargo geiger` scan for
`tabdat-explore-rs`, `tabdat-language`, and `tabdat-runtime`. The hosted policy
job is the acceptance gate for the final head.

## Hosted acceptance and merge record

Draft PR [#43](https://github.com/SaehwanPark/tabdat-explore-rs/pull/43) was
opened before implementation. The final PR-head workflow links, ready-for-review
transition, squash merge SHA, temporary-branch cleanup, and post-merge matrix
will be appended here before closeout.

## Deferred scope

Predicate-form `drop if <expression>` execution, boolean/null retention,
expression functions, overflow/non-finite arithmetic, lazy/materialized
execution, wildcard/range expansion, labels/panel metadata, `last_operation`,
formatting, CLI/REPL, JSON/MCP, and broad transform sequencing remain deferred.
This evidence does not claim full Python `drop` parity or broad Phase 4
completion.
