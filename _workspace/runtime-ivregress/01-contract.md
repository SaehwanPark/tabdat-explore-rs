# Bounded `ivregress` syntax contract

## Status and scope

- Status: `accepted` after PR #64 and its PR-head and merge-head workflows
  passed.
- Owner: the parent task owner; this slice is intentionally parser-only.
- Target: recover and port the direct `ivregress` command boundary without
  claiming instrumental-variables estimation, covariance, model state, or
  post-estimation execution.
- Rust paths in scope: `crates/tabdat-language/src/lib.rs`, its parser tests,
  the runtime unsupported-command contract, and this evidence directory.
- Runtime paths are otherwise out of scope. `tabdat-runtime` will return an
  explicit typed unsupported-command error for `Command::IvRegress` until the
  statistical state and active-relation contracts are separately implemented.

The accepted syntax target is:

    ivregress 2sls|gmm <y> [exog_vars], endog(<var>) iv(<vars>) [robust|cluster(<var>) noconstant]

The parser produces an owned typed command containing the estimator, outcome,
ordered exogenous variables, one endogenous variable, one or more instruments,
and the bounded robust/cluster/intercept options.

## Authority and recovered behavior

- Python oracle checkout: `C:\Users\saehwan\repos\tabdat-python-oracle`.
- Pinned revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`.
- Pinned tree: `601b236788872323af9277d2276a236154a0f129`.
- Lockfile SHA-256: `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
- Oracle sources: `src/tabdat/models.py` (`IvRegressCommand`),
  `src/tabdat/parser.py` (`_parse_ivregress`), and
  `docs/commands/ivregress.md`.
- Focused oracle selection:
  `uv run --no-sync pytest -q -p no:cacheprovider tests/test_parser.py -k ivregress`
  produced `11 passed, 478 deselected` at the pinned revision.

The recovered parser accepts unquoted `2sls` or `gmm` as the estimator,
requires an outcome and optional ordered exogenous variables, and requires
exactly one `endog(<var>)` option plus at least one variable in `iv(<vars>)`.
`robust` and `noconstant` are flag-only options; `cluster(<var>)` accepts one
variable, and robust cannot be combined with cluster. The endogenous variable
must not also appear in the exogenous list. Option names and estimator control
tokens retain the oracle's case/quoting boundary; variable names retain their
decoded quoted text.

## Rust contract

1. `parse_command` recognizes `ivregress` case-insensitively and returns
   `Command::IvRegress` with an owned `IvRegressCommand`.
2. `IvEstimator::TwoStageLeastSquares` represents `2sls`, and
   `IvEstimator::GeneralizedMethodOfMoments` represents `gmm`.
3. The command requires at least an estimator and outcome, exactly one
   `endog(<var>)` variable, and at least one `iv(<var> ...)` variable.
4. Exogenous and instrument lists preserve source order. The endogenous
   variable may not appear in exogenous variables.
5. `robust` and `noconstant` are flags; `cluster(<var>)` accepts exactly one
   variable; duplicate structured options and valued flag forms are rejected.
   Robust and cluster are mutually exclusive.
6. Conditions, assignment syntax, unsupported options, malformed option
   values, and invalid estimator boundaries preserve bounded diagnostics.
7. Parsing does not inspect files, initialize DuckDB, fit a model, or mutate
   session state. Runtime execution is explicitly deferred.

## Acceptance checks

- Oracle: focused `tests/test_parser.py -k ivregress` passes at the pinned
  revision.
- Rust: focused language parser tests cover 2SLS/GMM, ordered variables,
  quoted names, robust/cluster/noconstant, required structured options,
  duplicate options, estimator boundaries, and exact diagnostics.
- Runtime: a focused contract test proves parsed `ivregress` returns the typed
  unsupported-command error without backend work.
- Repository baseline: format, locked workspace check/test, and clippy with
  warnings denied pass before the PR is marked ready.
- Documentation and `git diff --check` pass; only this slice's files are
  staged.

## Known gaps and next dependency

This slice does not implement instrument relevance or exogeneity checks,
first-stage/second-stage fitting, GMM weighting, covariance estimators,
cluster degrees-of-freedom, missingness, prediction, post-estimation state,
labels, formatting, CLI, JSON, MCP, or broad Python `ivregress` parity. Those
behaviors remain future statistical evidence, gated by active-relation and
typed model-state contracts. The parser can be merged without implying that a
parsed `ivregress` command executes today.

The contract checkpoint is
[`a3c5d86`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/a3c5d86fa4df49f3a915214a28e8fb7591cfb2b9);
the implementation checkpoint is
[`8a67747`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/8a6774703c72470f174731ad1c488734dcfad7df); and PR
[#64](https://github.com/SaehwanPark/tabdat-explore-rs/pull/64) was squash-merged
to `main` as
[`d35555111800b4655d30d1bc1448da8ba41d2f21`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/d35555111800b4655d30d1bc1448da8ba41d2f21).
