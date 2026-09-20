# Join syntax migration evidence

## Status and boundary

Status: `complete`.

This is the accepted parser-only `join` slice. It ports the direct command
boundary and typed ownership contract; it does not claim that a parsed join
executes. Runtime execution remains an explicit unsupported-command result
until named-table state and SQL/table creation are available.

## Authority and recovered behavior

The Python oracle is the isolated checkout at
`C:\Users\saehwan\repos\tabdat-python-oracle`, pinned to revision
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`). Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
The relevant sources are `src/tabdat/models.py`, `src/tabdat/parser.py`, and
`docs/commands/join.md`; the focused parser command was:

```text
uv run --no-sync pytest -q -p no:cacheprovider tests/test_parser.py -k join
```

It passed with `12 passed, 477 deselected`. The oracle executor tests were
inspected but intentionally deferred: they depend on named-table state and SQL
creation/activation that are not implemented in this Rust workspace.

## Rust implementation

PR [#59](https://github.com/SaehwanPark/tabdat-explore-rs/pull/59) added
`Command::Join` and owned `JoinCommand`/`JoinHow` types in
[`crates/tabdat-language/src/lib.rs`](../../crates/tabdat-language/src/lib.rs).
The parser accepts:

```text
join <table> on <keylist> [, how=inner|left suffix(_right)]
```

It preserves table/key text, requires an unquoted `on` separator, rejects
duplicate keys, supports `inner`/`left` and a non-empty suffix, applies the
oracle defaults `inner` and `_right`, and rejects reserved table names and
unsupported or duplicate options with bounded diagnostics. The parser does
not inspect files, initialize DuckDB, or mutate session state.

The runtime records `join` in command naming but deliberately continues to
return the typed `UnsupportedCommand { name: "join" }` error. The focused
runtime test demonstrates that no backend execution is introduced by this
slice: [`crates/tabdat-runtime/tests/join_contract.rs`](../../crates/tabdat-runtime/tests/join_contract.rs).
Parser coverage is in
[`crates/tabdat-language/tests/parser_contract.rs`](../../crates/tabdat-language/tests/parser_contract.rs).

The contract checkpoint is
[`c9cb448ff3d781a2f8ccb19f7f0adb6892bbfca6`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/c9cb448ff3d781a2f8ccb19f7f0adb6892bbfca6);
the implementation checkpoint is
[`dfa1ecc72a99f8dac42170c4bc38cf840886bb74`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/dfa1ecc72a99f8dac42170c4bc38cf840886bb74);
and the squash merge is
[`585c53fcdce135456abded9b27df75fbcbcda8cf`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/585c53fcdce135456abded9b27df75fbcbcda8cf).

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

The workspace test command passed the scaffold, language, parser-contract,
runtime, and join-contract tests. The geiger reports found no first-party
unsafe code in `tabdat-explore-rs`, `tabdat-language`, or `tabdat-runtime`.

Hosted PR-head acceptance passed:

- [PR-head CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35500099510), including [Rust baseline job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35500099510/job/106050265204) and [dependency/unsafe policy job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35500099510/job/106050265271).
- [PR-head runtime workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35500099554), including [Linux runtime job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35500099554/job/106050232679).

The post-merge `main` workflows also passed:

- [merge-head CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35501135166), including [Rust baseline job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35501135166/job/106052979896) and [dependency/unsafe policy job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35501135166/job/106052979942).
- [merge-head runtime workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35501135235), including [Linux runtime job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35501135235/job/106052979870).

## Deviations and deferrals

This evidence covers only the parser boundary. SQL creation, named-table
registry/activation, DuckDB join execution, key type/null semantics, right-side
collision naming, ordering, relation publication, labels, lazy/materialized
behavior, persistence, formatting, CLI, JSON, MCP, and broad Python join parity
remain deferred. Phase 6.4 `join` therefore remains unchecked; this accepted
syntax item is recorded under the direct language-slice work.
