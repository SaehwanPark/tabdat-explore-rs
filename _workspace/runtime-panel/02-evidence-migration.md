# Panel syntax migration evidence

## Status and boundary

Status: `complete`.

This is the accepted parser-only `panel` slice. It ports the direct command
boundary and typed ownership contract; it does not claim panel metadata or
structural-summary execution. Runtime execution remains an explicit
unsupported-command result until active-relation panel state and summary
semantics are separately recovered and tested.

## Authority and recovered behavior

The Python oracle is the isolated checkout at
`C:\Users\saehwan\repos\tabdat-python-oracle`, pinned to revision
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`). Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
The relevant sources are `src/tabdat/models.py` (PanelCommand),
`src/tabdat/parser.py` (the panel parser), and
`docs/commands/panel.md`; the focused parser command was:

    uv run --no-sync pytest -q -p no:cacheprovider tests/test_parser.py -k panel

It passed with `7 passed, 482 deselected`. The oracle panel executor tests
were inspected but intentionally deferred: they require active relation
metadata, panel grouping and ordering, structural summaries, and session
publication semantics that are not implemented in this Rust workspace.

The recovered parser accepts `panel` as a report action,
`panel <id_var> <time_var>` as a set action, and `panel clear` as a clear
action. The command keyword and clear control token are case-insensitive.
String-quoted `clear` follows the oracle's clear-action tokenization, while
backtick-quoted `clear` is retained as a variable name in the two-argument
form. Entity and time names must be distinct; conditions, options, assignment
syntax, extra arguments, and malformed arity preserve the bounded syntax
diagnostic.

## Rust implementation

PR [#62](https://github.com/SaehwanPark/tabdat-explore-rs/pull/62) added
`Command::Panel`, `PanelCommand`, and the typed `PanelAction` report,
clear, and set forms in
[crates/tabdat-language/src/lib.rs](../../crates/tabdat-language/src/lib.rs).
Parser coverage is in
[crates/tabdat-language/tests/parser_contract.rs](../../crates/tabdat-language/tests/parser_contract.rs),
including the string/backtick `clear` boundary and bounded diagnostics.

The runtime records `panel` in command naming but deliberately continues to
return `RuntimeError::UnsupportedCommand { name: "panel" }`. The focused
runtime test demonstrates that no backend execution is introduced by this
slice:
[crates/tabdat-runtime/tests/panel_contract.rs](../../crates/tabdat-runtime/tests/panel_contract.rs).

The contract checkpoint is
[331b5fb4fb0a59c91a886c3e07f47fab68592cd5](https://github.com/SaehwanPark/tabdat-explore-rs/commit/331b5fb4fb0a59c91a886c3e07f47fab68592cd5);
the implementation checkpoint is
[67de5b835489d196dc800607054b227871b8b6d4](https://github.com/SaehwanPark/tabdat-explore-rs/commit/67de5b835489d196dc800607054b227871b8b6d4);
the keyword-boundary test checkpoint is
[d1eeb2a321c80ec13e6f5daa57129cbecbe8d7c4](https://github.com/SaehwanPark/tabdat-explore-rs/commit/d1eeb2a321c80ec13e6f5daa57129cbecbe8d7c4);
and the squash merge is
[92e5d5e5be9aaededbfe3f44ded2d4319cdcac87](https://github.com/SaehwanPark/tabdat-explore-rs/commit/92e5d5e5be9aaededbfe3f44ded2d4319cdcac87).

## Verification evidence

The focused Rust parser and runtime contract tests passed. The following
repository checks also passed locally:

    cargo fmt --all -- --check
    cargo check --locked --workspace --all-targets
    cargo test --locked --workspace --all-targets -j 1
    cargo clippy --locked --workspace --all-targets -j 1 -- -D warnings
    cargo deny check
    cargo audit -D warnings
    metadata-driven cargo geiger checks for the workspace packages
    git diff --check

The first unconstrained local full-test invocation hit Windows error 1455
because the paging file was too small for concurrent rustc work; the
single-job locked rerun passed all workspace targets. The successful run
covered the scaffold, 44 language unit tests, 62 parser-contract tests, 33
runtime unit tests, the panel runtime contract, and all existing runtime
integration contracts. The geiger reports found no first-party unsafe code in
`tabdat-explore-rs`, `tabdat-language`, or `tabdat-runtime`.

Hosted PR-head acceptance passed:

- [CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35508978847),
  including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35508978847/job/106073500338)
  and [dependency/unsafe policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35508978847/job/106073500170).
- [Runtime boundary workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35508978878),
  including [Linux runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35508978878/job/106073496562).

The post-merge `main` workflows also passed:

- [merge-head CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35510023970),
  including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35510023970/job/106076191527)
  and [dependency/unsafe policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35510023970/job/106076191440).
- [merge-head runtime workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35510023960),
  including [Linux runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35510023960/job/106076191136).

## Deviations and deferrals

This evidence covers only the parser boundary. Panel metadata ownership,
active-schema lookup, numeric/time validation, duplicate entity-time checks,
structural summaries, ordering, missingness, persistence, lazy/materialized
behavior, labels, formatting, CLI, JSON, MCP, and broad Python `panel` parity
remain deferred. Phase 11.1 `panel` runtime work therefore remains unchecked;
the accepted syntax item is recorded under the direct Phase 5.1 language slice.
