# Review: bounded `lincom` runtime

## Scope reviewed

Reviewed the complete branch diff, surrounding `Session`/`LinearRegressionState` dispatch and ownership boundaries, `PostEstimationModel` covariance calculation, the pinned Python `_execute_lincom` and expression formatter, the direct statsmodels/SciPy reference calculations, test output, and updated `SPEC.md`, `ARCHITECTURE.md`, and roadmap entries.

## Findings and dispositions

1. **Confidence interval level boundary — fixed before final validation.** The initial public statistics method accepted confidence level `0.0` even though its documented contract was the open interval `(0, 100)`. Validation now rejects both endpoints and the statistics contract tests cover them.
2. **Python numeric label formatting — fixed and covered.** Rust's float display selects fixed notation at magnitudes where Python `str(float)` uses scientific notation. The label formatter now applies Python's `[-4, 16)` decimal-exponent threshold; contract tests cover `0.00001` and `10000000000000000.0`.
3. **Extreme-tail p-value difference — documented, not treated as a parity defect.** Python computes `2 * (1 - cdf)`, which can lose a representable tail probability to cancellation. Rust uses the statistics crate's regularized-incomplete-beta two-tailed probability, checked against SciPy survival-function references; ordinary fixtures agree within the predeclared comparator. This intentional stable-tail difference is described in `02-evidence-migration.md`.
4. **No-positive-df inference — explicit scope boundary.** The stats inference method returns `InsufficientObservations` at df 0 rather than implementing Python's normal fallback. Current Rust least-squares fits require positive residual df, so the bounded runtime cannot reach that case. The public method documents this requirement, tests it, and the broader fallback remains deferred.

## Review outcome

No actionable correctness, state-mutation, or dependency-safety issues remain in the reviewed bounded scope. The expression analyzer is pure over owned AST/model metadata; `Session::execute_lincom` borrows stored state, performs no DuckDB operation, and its success/failure immutability is tested. Estimate, covariance-derived SE, statistic, p-value, CI, residual df, and labels are exercised for OLS, WLS, GLS, HC1, clustered covariance, no-intercept, scalar/constant expressions, and zero SE.

## Validation caveat

On this Windows host, the all-target Cargo test command reports existing `export_contract` and `save_contract` path-parser failures (`unsupported token in command: \\`) in path cases. All other test binaries, including the new `lincom` tests, pass under `--no-fail-fast`. Hosted Linux PR checks are required to establish the repository acceptance baseline and are not inferred from local checks.
