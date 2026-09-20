# `xtreg` syntax migration evidence

## Status and boundary

Status: `complete`.

This is the accepted parser-only `xtreg` slice. It ports the direct command
boundary and typed ownership contract; it does not claim panel validation,
fixed/random-effects estimation, covariance, model state, post-estimation, or
active-session execution. Runtime execution remains an explicit
unsupported-command result until panel metadata and statistical model-state
contracts are separately recovered and tested.

## Authority and recovered behavior

The Python oracle is the isolated checkout at
`C:\Users\saehwan\repos\tabdat-python-oracle`, pinned to revision
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`). Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
The relevant sources are `src/tabdat/models.py` (`XtRegCommand`),
`src/tabdat/parser.py` (`_parse_xtreg`), and `docs/commands/xtreg.md`.
The focused parser command was:

    uv run --no-sync pytest -q -p no:cacheprovider tests/test_parser.py -k xtreg

It passed with `10 passed, 479 deselected`. The recovered parser accepts an
outcome and ordered predictors, exactly one unquoted `fe` or `re` estimator
flag, and the bounded `robust` or one-variable `cluster(<var>)` options.
Option names are case-sensitive in the oracle and quoted variable names retain
decoded text. Robust and cluster are mutually exclusive. Conditions, valued
flags, unsupported options, duplicate structured options, and multi-variable
cluster options are rejected with the recovered diagnostics.

## Rust implementation

PR [#65](https://github.com/SaehwanPark/tabdat-explore-rs/pull/65) added
`Command::XtReg`, `XtRegCommand`, and `XtRegEstimator` in
[crates/tabdat-language/src/lib.rs](../../crates/tabdat-language/src/lib.rs).
Parser coverage is in
[crates/tabdat-language/tests/parser_contract.rs](../../crates/tabdat-language/tests/parser_contract.rs),
including fixed/random-effects forms, ordered and quoted variables,
robust/cluster, estimator exclusivity, option arity, valued flags, and exact
diagnostics.

The runtime records `xtreg` in command naming but deliberately continues to
return `RuntimeError::UnsupportedCommand { name: "xtreg" }`. The focused
runtime test demonstrates that no panel metadata lookup, model fitting, or
backend execution is introduced by this slice:
[crates/tabdat-runtime/tests/xtreg_contract.rs](../../crates/tabdat-runtime/tests/xtreg_contract.rs).

The contract checkpoint is
[4005c2d](https://github.com/SaehwanPark/tabdat-explore-rs/commit/4005c2d);
the implementation checkpoint is
[0791d74](https://github.com/SaehwanPark/tabdat-explore-rs/commit/0791d74); and
the squash merge is
[c88d0028395633d9353fbac7afc4b485231042e3](https://github.com/SaehwanPark/tabdat-explore-rs/commit/c88d0028395633d9353fbac7afc4b485231042e3).

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

The successful test run covered the scaffold, 44 language unit tests, 68
parser-contract tests, 33 runtime unit tests, the xtreg runtime contract, and
all existing runtime integration contracts. The geiger reports found no
first-party unsafe code in `tabdat-explore-rs`, `tabdat-language`, or
`tabdat-runtime`.

Hosted PR-head acceptance passed:

- [CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35522920538),
  including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35522920538/job/106110012332)
  and [dependency/unsafe policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35522920538/job/106110012172).
- [Runtime boundary workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35522920548),
  including [Linux runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35522920548/job/106110012362).

The post-merge `main` workflows passed:

- [merge-head CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35524113771),
  including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35524113771/job/106113162854)
  and [dependency/unsafe policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35524113771/job/106113162977).
- [merge-head runtime workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35524113865),
  including [Linux runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35524113865/job/106113163280).

## Deviations and deferrals

This evidence covers only the parser boundary. Panel identifier/time
validation, within transformation, fixed/random-effects fitting, Hausman
comparisons, covariance and clustered degrees-of-freedom semantics,
missingness, prediction, post-estimation state, labels, formatting, CLI, JSON,
MCP, and broad Python `xtreg` parity remain deferred. Phase 11.1 `xtreg, fe`
and `xtreg, re` runtime work therefore remains unchecked; the accepted syntax
item is recorded under the direct Phase 5.1 language slice.
