# Append syntax migration evidence

## Status and boundary

Status: `complete`.

This is the accepted parser-only `append` slice. It ports the direct command
boundary and typed ownership contract; it does not claim that a parsed append
executes. Runtime execution remains an explicit unsupported-command result
until named-table state and SQL/table creation are available.

## Authority and recovered behavior

The Python oracle is the isolated checkout at
`C:\Users\saehwan\repos\tabdat-python-oracle`, pinned to revision
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`). Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
The relevant sources are `src/tabdat/models.py`, `src/tabdat/parser.py`, and
`docs/commands/append.md`; the focused parser command was:

```text
uv run --no-sync pytest -q -p no:cacheprovider tests/test_parser.py -k append
```

It passed with `7 passed, 482 deselected`. The oracle executor tests were
inspected but intentionally deferred: they depend on named-table state and SQL
creation/activation that are not implemented in this Rust workspace.

## Rust implementation

PR [#60](https://github.com/SaehwanPark/tabdat-explore-rs/pull/60) added
`Command::Append { table_name }` in
[`crates/tabdat-language/src/lib.rs`](../../crates/tabdat-language/src/lib.rs).
The parser accepts exactly `append <table>`, preserves owned table text after
the established quote/backtick decoding, validates the non-reserved table-name
boundary, and rejects extra arguments, options, conditions, and assignment
syntax with the bounded append diagnostic.

The runtime records `append` in command naming but deliberately continues to
return `RuntimeError::UnsupportedCommand { name: "append" }`. The focused
runtime test demonstrates that no backend execution is introduced by this
slice: [`crates/tabdat-runtime/tests/append_contract.rs`](../../crates/tabdat-runtime/tests/append_contract.rs).
Parser coverage is in
[`crates/tabdat-language/tests/parser_contract.rs`](../../crates/tabdat-language/tests/parser_contract.rs).

The contract checkpoint is
[`8874983441332d9916f8ae581d9b8f4a4d71d1cb`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/8874983441332d9916f8ae581d9b8f4a4d71d1cb);
the implementation checkpoint is
[`04e397caf4d61ea4aaeba42d407c198bbb80d2a1`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/04e397caf4d61ea4aaeba42d407c198bbb80d2a1);
and the squash merge is
[`8ab016f1e014445288d2411cf7d058e2bd523b49`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/8ab016f1e014445288d2411cf7d058e2bd523b49).

## Verification evidence

The following local checks passed on the implementation revision:

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo deny check
cargo audit -D warnings
metadata-driven cargo geiger checks for the workspace packages
git diff --check
```

The first full local test invocation encountered a transient Cargo artifact
format error while parallel runtime targets were compiling; the immediate
rerun passed all workspace targets. The successful run covered the scaffold,
44 language unit tests, 58 parser-contract tests, 33 runtime unit tests, and
all runtime integration contracts, including the append contract. The geiger
reports found no first-party unsafe code in `tabdat-explore-rs`,
`tabdat-language`, or `tabdat-runtime`.

Hosted PR-head acceptance passed:

- [PR-head CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35502429571), including [Rust baseline job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35502429571/job/106056506691) and [dependency/unsafe policy job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35502429571/job/106056506592).
- [PR-head runtime workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35502429569), including [Linux runtime job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35502429569/job/106056473019).

The post-merge `main` workflows are recorded after completion:

- [merge-head CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35503495604), including [Rust baseline job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35503495604/job/106059279758) and [dependency/unsafe policy job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35503495604/job/106059279641).
- [merge-head runtime workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35503495623), including [Linux runtime job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35503495623/job/106059279711).

## Deviations and deferrals

This evidence covers only the parser boundary. SQL creation, named-table
registry/activation, DuckDB append execution, schema compatibility, column
union/type and missingness semantics, row ordering/publication, labels,
lazy/materialized behavior, persistence, formatting, CLI, JSON, MCP, and broad
Python append parity remain deferred. Phase 6.4 `append` therefore remains
unchecked; this accepted syntax item is recorded under the direct language
slice work.
