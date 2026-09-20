# `xtabond` syntax migration evidence

## Status and boundary

Status: `complete`.

This is the accepted parser-only `xtabond` slice. It ports the direct command
boundary and typed ownership contract; it does not claim panel metadata
validation, dynamic-panel GMM estimation, instrument construction, covariance,
model state, post-estimation, or active-session execution. Runtime execution
remains an explicit unsupported-command result until panel metadata and
statistical model-state contracts are separately recovered and tested.

## Authority and recovered behavior

The Python oracle is the isolated checkout at
`C:\Users\saehwan\repos\tabdat-python-oracle`, pinned to revision
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`). Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
The relevant sources are `src/tabdat/models.py` (`XtAbondCommand`),
`src/tabdat/parser.py` (`_parse_xtabond`), and `docs/commands/xtabond.md`.
The focused parser command was:

    uv run --no-sync pytest -q -p no:cacheprovider tests/test_parser.py -k xtabond

It passed with `8 passed, 481 deselected`. The recovered parser accepts one
outcome with zero or more ordered predictors, the flag-only `robust` option,
and integer `lags()`/`instlag()` options. Options are case-sensitive; repeated
`robust` is accepted, structured duplicates are rejected, `lags` is at least 1,
`instlag` is at least 2, and `instlag` must be greater than `lags`. Quoted
variables retain decoded text and malformed values preserve the recovered
diagnostics.

## Rust implementation

PR [#67](https://github.com/SaehwanPark/tabdat-explore-rs/pull/67) added
`Command::XtAbond` and `XtAbondCommand` in
[crates/tabdat-language/src/lib.rs](../../crates/tabdat-language/src/lib.rs).
Parser coverage is in
[crates/tabdat-language/tests/parser_contract.rs](../../crates/tabdat-language/tests/parser_contract.rs),
including defaults, robust, lag options, ordering, quoted names, duplicate and
valued options, unsupported options, numeric bounds, and exact diagnostics.

The runtime records `xtabond` in command naming but deliberately continues to
return `RuntimeError::UnsupportedCommand { name: "xtabond" }`. The focused
runtime test demonstrates that no panel metadata lookup, instrument
construction, model fitting, or backend execution is introduced by this slice:
[crates/tabdat-runtime/tests/xtabond_contract.rs](../../crates/tabdat-runtime/tests/xtabond_contract.rs).

The contract checkpoint is
[eb9f4c2](https://github.com/SaehwanPark/tabdat-explore-rs/commit/eb9f4c2);
the implementation checkpoint is
[43179df](https://github.com/SaehwanPark/tabdat-explore-rs/commit/43179df); and
the squash merge is
[820376d](https://github.com/SaehwanPark/tabdat-explore-rs/commit/820376d6c7ce3c4a4ff8abf24fe9a9b3ad84cea6).

## Verification evidence

The focused Rust parser and runtime contract tests passed. The following
repository checks also passed locally:

    cargo fmt --all -- --check
    cargo check --locked --workspace --all-targets -j 1
    cargo test --locked --workspace --all-targets -j 1
    cargo clippy --locked --workspace --all-targets -j 1 -- -D warnings
    cargo deny check
    cargo audit -D warnings
    metadata-driven cargo geiger checks for the workspace packages
    git diff --check

The successful test run covered the scaffold, 44 language unit tests, 72
parser-contract tests, 33 runtime unit tests, the xtabond runtime contract,
and all existing runtime integration contracts. The geiger reports found no
first-party unsafe code in `tabdat-explore-rs`, `tabdat-language`, or
`tabdat-runtime`.

Hosted PR-head acceptance passed:

- [CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35532708950),
  including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35532708950/job/106136105095)
  and [dependency/unsafe policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35532708950/job/106136104900).
- [Runtime boundary workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35532708947),
  including [Linux runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35532708947/job/106136033403).

The post-merge `main` workflows passed:

- [merge-head CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35534031685),
  including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35534031685/job/106139722391)
  and [dependency/unsafe policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35534031685/job/106139722190).
- [merge-head runtime boundary workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35534031690),
  including [Linux runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35534031690/job/106139722191).

## Deviations and deferrals

This evidence covers only the parser boundary. Panel identifier/time
validation, dynamic-panel GMM formulas, instrument matrices, lagged-variable
construction, weighting, covariance and degrees of freedom, missingness,
prediction, post-estimation state, labels, formatting, CLI, JSON, MCP, and
broad Python `xtabond` parity remain deferred. Phase 11.1 `xtabond` runtime
work therefore remains unchecked; the accepted syntax item is recorded under
the direct Phase 5.1 language slice.
