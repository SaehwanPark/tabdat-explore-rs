# Bounded `xtreg` syntax contract

## Status and scope

- Status: `draft` at the contract checkpoint; acceptance follows the PR-head
  implementation and merge-head workflow evidence.
- Owner: the parent task owner; this slice is intentionally parser-only.
- Target: recover and port the direct `xtreg` command boundary without
  claiming fixed/random-effects estimation, covariance, panel validation, or
  model-state execution.
- Rust paths in scope: `crates/tabdat-language/src/lib.rs`, its parser tests,
  the runtime unsupported-command contract, and this evidence directory.
- Runtime paths are otherwise out of scope. `tabdat-runtime` will return an
  explicit typed unsupported-command error for `Command::XtReg` until panel
  metadata and statistical model-state contracts are separately implemented.

The accepted syntax target is:

    xtreg <y> <xvars>, fe|re [robust|cluster(<var>)]

The parser produces an owned typed command containing the outcome, ordered
predictors, fixed- or random-effects estimator, and bounded robust/cluster
options.

## Authority and recovered behavior

- Python oracle checkout: `C:\Users\saehwan\repos\tabdat-python-oracle`.
- Pinned revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`.
- Pinned tree: `601b236788872323af9277d2276a236154a0f129`.
- Lockfile SHA-256: `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
- Oracle sources: `src/tabdat/models.py` (`XtRegCommand`),
  `src/tabdat/parser.py` (`_parse_xtreg`), and `docs/commands/xtreg.md`.
- Focused oracle selection:
  `uv run --no-sync pytest -q -p no:cacheprovider tests/test_parser.py -k xtreg`
  produced `10 passed, 479 deselected` at the pinned revision.

The recovered parser accepts at least one predictor after the outcome,
requires exactly one unquoted `fe` or `re` flag, and accepts `robust` or one
variable in `cluster(<var>)`. Option names are case-sensitive in the oracle;
quoted variable names retain decoded text. Robust and cluster are mutually
exclusive. Conditions, valued flags, unsupported options, duplicate cluster
options, and multi-variable cluster options are rejected.

## Rust contract

1. `parse_command` recognizes `xtreg` case-insensitively and returns
   `Command::XtReg` with an owned `XtRegCommand`.
2. `XtRegEstimator::FixedEffects` represents `fe`, and
   `XtRegEstimator::RandomEffects` represents `re`.
3. The command requires one outcome, at least one predictor, and exactly one
   accepted estimator flag.
4. Predictors retain source order and quoted decoded text.
5. `robust` is a flag and `cluster(<var>)` accepts exactly one variable;
   robust and cluster cannot be combined. Duplicate structured options and
   valued flags preserve bounded diagnostics.
6. Conditions, assignment syntax, unsupported options, and malformed
   variable boundaries preserve the recovered syntax diagnostics.
7. Parsing does not inspect files, require panel metadata, fit a model,
   initialize DuckDB, or mutate session state. Runtime execution is explicitly
   deferred.

## Acceptance checks

- Oracle: focused `tests/test_parser.py -k xtreg` passes at the pinned
  revision.
- Rust: focused language parser tests cover FE/RE, ordered and quoted
  variables, robust/cluster, estimator exclusivity, option arity, valued flags,
  and exact diagnostics.
- Runtime: a focused contract test proves parsed `xtreg` returns the typed
  unsupported-command error without backend work.
- Repository baseline: format, locked workspace check/test, and clippy with
  warnings denied pass before the PR is marked ready.
- Documentation and `git diff --check` pass; only this slice's files are
  staged.

## Known gaps and next dependency

This slice does not implement panel metadata lookup, within transformation,
fixed/random-effects formulas, Hausman comparisons, covariance estimators,
cluster degrees of freedom, missingness, prediction, post-estimation state,
labels, formatting, CLI, JSON, MCP, or broad Python `xtreg` parity. Those
behaviors remain future statistical evidence, gated by panel metadata and
typed model-state contracts. The parser can be merged without implying that a
parsed `xtreg` command executes today.
