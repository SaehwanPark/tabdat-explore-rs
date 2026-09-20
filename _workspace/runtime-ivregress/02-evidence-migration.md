# `ivregress` syntax migration evidence

## Status and boundary

Status: `complete`.

This is the accepted parser-only `ivregress` slice. It ports the direct
command boundary and typed ownership contract; it does not claim IV
estimation, covariance, model state, post-estimation, or active-session
execution. Runtime execution remains an explicit unsupported-command result
until the statistical runtime and typed model-state contracts are separately
recovered and tested.

## Authority and recovered behavior

The Python oracle is the isolated checkout at
`C:\Users\saehwan\repos\tabdat-python-oracle`, pinned to revision
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`). Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
The relevant sources are `src/tabdat/models.py` (`IvRegressCommand`),
`src/tabdat/parser.py` (`_parse_ivregress`), and
`docs/commands/ivregress.md`.
The focused parser command was:

    uv run --no-sync pytest -q -p no:cacheprovider tests/test_parser.py -k ivregress

It passed with `11 passed, 478 deselected`. The recovered parser accepts
unquoted `2sls` or `gmm`, an outcome with ordered exogenous variables, exactly
one `endog(<var>)`, one or more `iv(<vars>)`, and the bounded robust,
cluster, and noconstant options. It rejects unsupported/duplicate structured
options, valued flags, missing or malformed required options, robust+cluster,
and an endogenous variable repeated among exogenous variables.

## Rust implementation

PR [#64](https://github.com/SaehwanPark/tabdat-explore-rs/pull/64) added
`Command::IvRegress`, `IvRegressCommand`, and `IvEstimator` in
[crates/tabdat-language/src/lib.rs](../../crates/tabdat-language/src/lib.rs).
Parser coverage is in
[crates/tabdat-language/tests/parser_contract.rs](../../crates/tabdat-language/tests/parser_contract.rs),
including 2SLS/GMM, ordered and quoted variables, robust/cluster/noconstant,
required-option validation, and exact diagnostics.

The runtime records `ivregress` in command naming but deliberately continues
to return `RuntimeError::UnsupportedCommand { name: "ivregress" }`. The
focused runtime test demonstrates that no model fitting or backend execution
is introduced by this slice:
[crates/tabdat-runtime/tests/ivregress_contract.rs](../../crates/tabdat-runtime/tests/ivregress_contract.rs).

The contract checkpoint is
[a3c5d86](https://github.com/SaehwanPark/tabdat-explore-rs/commit/a3c5d86fa4df49f3a915214a28e8fb7591cfb2b9);
the implementation checkpoint is
[8a67747](https://github.com/SaehwanPark/tabdat-explore-rs/commit/8a6774703c72470f174731ad1c488734dcfad7df); and the squash merge is
[d35555111800b4655d30d1bc1448da8ba41d2f21](https://github.com/SaehwanPark/tabdat-explore-rs/commit/d35555111800b4655d30d1bc1448da8ba41d2f21).

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

The successful test run covered the scaffold, 44 language unit tests, 66
parser-contract tests, 33 runtime unit tests, the ivregress runtime contract,
and all existing runtime integration contracts. The geiger reports found no
first-party unsafe code in `tabdat-explore-rs`, `tabdat-language`, or
`tabdat-runtime`.

Hosted PR-head acceptance passed:

- [CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35517936562),
  including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35517936562/job/106096961268)
  and [dependency/unsafe policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35517936562/job/106096961340).
- [Runtime boundary workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35517936558),
  including [Linux runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35517936558/job/106096922223).

The post-merge `main` workflows passed:

- [merge-head CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35519118291),
  including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35519118291/job/106100009300)
  and [dependency/unsafe policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35519118291/job/106100009377).
- [merge-head runtime workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35519118356),
  including [Linux runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35519118356/job/106100009485).

## Deviations and deferrals

This evidence covers only the parser boundary. IV identification checks,
first-stage and second-stage fitting, GMM weighting, robust/cluster covariance,
degrees of freedom, missingness, prediction, post-estimation state, labels,
formatting, CLI, JSON, MCP, and broad Python `ivregress` parity remain
deferred. Phase 11.1 `ivregress 2sls` and `ivregress gmm` runtime work therefore
remains unchecked; the accepted syntax item is recorded under the direct Phase
5.1 language slice.
