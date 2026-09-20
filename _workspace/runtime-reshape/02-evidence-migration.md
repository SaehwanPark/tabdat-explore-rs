# Reshape syntax migration evidence

## Status and boundary

Status: `complete`.

This is the accepted parser-only `reshape` slice. It ports the direct command
boundary and typed ownership contract; it does not claim that a parsed reshape
executes. Runtime execution remains an explicit unsupported-command result
until relation/session reshape semantics are separately recovered and tested.

## Authority and recovered behavior

The Python oracle is the isolated checkout at
`C:\Users\saehwan\repos\tabdat-python-oracle`, pinned to revision
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`). Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
The relevant sources are `src/tabdat/models.py` (`ReshapeCommand`),
`src/tabdat/parser.py` (`_parse_reshape` and option helpers), and
`docs/commands/reshape.md`; the focused parser command was:

```text
uv run --no-sync pytest -q -p no:cacheprovider tests/test_parser.py -k reshape
```

It passed with `16 passed, 473 deselected`. The oracle reshape executor tests
were inspected but intentionally deferred: they require the relation/session
model and publication semantics that are not implemented in this Rust
workspace.

## Rust implementation

PR [#61](https://github.com/SaehwanPark/tabdat-explore-rs/pull/61) added
`Command::Reshape`, `ReshapeCommand`, and `ReshapeDirection` in
[`crates/tabdat-language/src/lib.rs`](../../crates/tabdat-language/src/lib.rs).
The parser accepts:

```text
reshape long|wide <varlist>, i(<id_vars>) j(<name>)
```

It preserves ordered owned names, accepts the recovered direction and quote
behavior, requires unique variables and identifiers, requires exactly one
lowercase `i()` and `j()` option, enforces the pairwise distinct-name rule, and
returns deterministic bounded diagnostics for malformed forms and unsupported
options. The parser does not inspect files, initialize DuckDB, or mutate
session state.

The runtime records `reshape` in command naming but deliberately continues to
return the typed `UnsupportedCommand { name: "reshape" }` error. The focused
runtime test demonstrates that no backend execution is introduced by this
slice: [`crates/tabdat-runtime/tests/reshape_contract.rs`](../../crates/tabdat-runtime/tests/reshape_contract.rs).
Parser coverage is in
[`crates/tabdat-language/tests/parser_contract.rs`](../../crates/tabdat-language/tests/parser_contract.rs).

The contract checkpoint is
[`48323f75423773e3b0501cd1eb7d2a8bcb0a3b5a`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/48323f75423773e3b0501cd1eb7d2a8bcb0a3b5a);
the casing correction is
[`e3e9897abb8fba93e912ee853885f7a1d18e7665`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/e3e9897abb8fba93e912ee853885f7a1d18e7665);
the implementation checkpoint is
[`0d1a059ab5c03c72fc78cb855ce4c9ea122dc0b9`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/0d1a059ab5c03c72fc78cb855ce4c9ea122dc0b9);
and the squash merge is
[`e6cc4f1768c9b55b8ead702a08a36283f2a27bee`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/e6cc4f1768c9b55b8ead702a08a36283f2a27bee).

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
44 language unit tests, 60 parser-contract tests, 33 runtime unit tests, the
reshape runtime contract, and all existing runtime integration contracts. The
geiger reports found no first-party unsafe code in `tabdat-explore-rs`,
`tabdat-language`, or `tabdat-runtime`.

Hosted PR-head acceptance passed:

- [PR-head CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35505184538), including [Rust baseline job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35505184538/job/106063705703) and [dependency/unsafe policy job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35505184538/job/106063705562).
- [PR-head runtime workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35505184479), including [Linux runtime job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35505184479/job/106063674856).

The post-merge `main` workflows also passed:

- [merge-head CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35506238505), including [Rust baseline job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35506238505/job/106066428748) and [dependency/unsafe policy job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35506238505/job/106066428825).
- [merge-head runtime workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35506238487), including [Linux runtime job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35506238487/job/106066428752).

## Deviations and deferrals

This evidence covers only the parser boundary. Long/wide relation execution,
identifier-group and missingness semantics, wide-column naming and collision
rules, row ordering, row counts, type coercion, relation publication, labels,
lazy/materialized behavior, persistence, formatting, CLI, JSON, MCP, and broad
Python `reshape` parity remain deferred. Phase 6.4 `reshape` therefore remains
unchecked; this accepted syntax item is recorded under the direct language
slice work.
