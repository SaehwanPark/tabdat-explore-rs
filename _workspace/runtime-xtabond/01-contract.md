# Bounded `xtabond` syntax contract

## Status and scope

- Status: `draft` at the contract checkpoint; acceptance follows the PR-head
  implementation and merge-head workflow evidence.
- Owner: the parent task owner; this slice is intentionally parser-only.
- Target: recover and port the direct `xtabond` command boundary without
  claiming panel metadata validation, dynamic-panel GMM estimation,
  instrument construction, covariance, model state, or post-estimation.
- Rust paths in scope: `crates/tabdat-language/src/lib.rs`, its parser tests,
  the runtime unsupported-command contract, and this evidence directory.
- Runtime paths are otherwise out of scope. `tabdat-runtime` will return an
  explicit typed unsupported-command error for `Command::XtAbond` until panel
  and statistical model-state contracts are separately implemented.

The accepted syntax target is:

    xtabond <y> [xvars] [, robust lags(#) instlag(#)]

The parser produces an owned typed command containing the outcome, ordered
predictors, robust flag, lag depth, and instrument lag start. Defaults are
`lags(1)` and `instlag(2)`; the instrument lag start must be greater than the
lag depth.

## Authority and recovered behavior

- Python oracle checkout: `C:\Users\saehwan\repos\tabdat-python-oracle`.
- Pinned revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`.
- Pinned tree: `601b236788872323af9277d2276a236154a0f129`.
- Lockfile SHA-256: `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
- Oracle sources: `src/tabdat/models.py` (`XtAbondCommand`),
  `src/tabdat/parser.py` (`_parse_xtabond`), and `docs/commands/xtabond.md`.
- Focused oracle selection:
  `uv run --no-sync pytest -q -p no:cacheprovider tests/test_parser.py -k xtabond`
  produced `8 passed, 481 deselected` at the pinned revision.

The recovered parser accepts one outcome with zero or more ordered predictors,
the flag-only `robust` option, and integer `lags()`/`instlag()` options. Option
names are case-sensitive; duplicate structured options are rejected while
repeated `robust` is set-membership behavior. `lags` is at least 1,
`instlag` is at least 2, and `instlag` must be greater than `lags`. Quoted
variables retain decoded text. Conditions, valued flags, unsupported options,
non-integer values, and invalid lag bounds are rejected with the recovered
diagnostics.

## Rust contract

1. `parse_command` recognizes `xtabond` case-insensitively and returns
   `Command::XtAbond` with an owned `XtAbondCommand`.
2. The command requires one outcome and preserves predictor source order and
   quoted decoded text.
3. `robust` is flag-only; repeated `robust` remains true. `lags(<integer>)`
   defaults to 1 and requires a minimum of 1. `instlag(<integer>)` defaults to
   2 and requires a minimum of 2.
4. The resolved instrument lag start must be strictly greater than the
   resolved lag depth. Duplicate structured options, valued flags, unsupported
   options, conditions, and malformed values preserve bounded diagnostics.
5. Parsing does not inspect files or panel metadata, construct instruments,
   fit a model, initialize DuckDB, or mutate session state. Runtime execution
   is explicitly deferred.

## Acceptance checks

- Oracle: focused `tests/test_parser.py -k xtabond` passes at the pinned
  revision.
- Rust: focused language parser tests cover defaults, robust, lag options,
  quoted variables, ordering, duplicate/valued options, unsupported options,
  numeric bounds, and the lag-order invariant with exact diagnostics.
- Runtime: a focused contract test proves parsed `xtabond` returns the typed
  unsupported-command error without backend work.
- Repository baseline: format, locked workspace check/test, and clippy with
  warnings denied pass before the PR is marked ready.
- Documentation and `git diff --check` pass; only this slice's files are
  staged.

## Known gaps and next dependency

This slice does not implement panel metadata lookup, dynamic-panel GMM,
instrument matrices, lagged-variable construction, weighting, covariance,
missingness, prediction, `estat overid`, post-estimation state, labels,
formatting, CLI, JSON, MCP, or broad Python `xtabond` parity. Those behaviors
remain future statistical evidence, gated by panel metadata and typed
model-state contracts. The parser can be merged without implying that a
parsed `xtabond` command executes today.
