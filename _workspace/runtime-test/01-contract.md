# Contract: bounded linear-regression `test` runtime

## Status and scope

- **State:** contract recovered; runtime implementation and validation are in progress on `feat/runtime-test`.
- **Rust base:** `main` at `0b4ae1fd37a20c8bbc0f730b816f743166cd8b54`.
- **Python oracle:** TabDat 0.25.0, commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree `601b236788872323af9277d2276a236154a0f129`, clean checkout, `uv.lock` SHA-256 `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
- **Trusted reference:** statsmodels 0.14.6 F tests and SciPy 1.17.1 F survival probabilities, on the same six-row fixture.
- **Roadmap:** Phase 7 §9.2, `Port test`; Phase 7 §9.3 statistical validation.
- **Selected skills:** `tabdat-migration`, `tabdat-statistical-validation`, `simple-code-writer`, and `code-reviewer`.
- **Risk:** medium — affine constraint mapping, covariance selection, and F-tail probabilities must preserve the fitted model contract.

This slice executes parsed linear restrictions against the current Rust OLS/WLS/GLS regression result. It includes classical, HC1 robust, and clustered covariance already represented by those results. It does not add estimators, parser forms, CLI/JSON/MCP rendering, or model-family dispatch for IV, binary, panel, causal, count, or other models. Rust least-squares fitting requires positive residual degrees of freedom, so Python's zero-df chi-square fallback is unreachable here and remains deferred.

## Python contract

Authoritative pinned paths:

- `src/tabdat/executor.py::_execute_test`, `_get_active_estimation_results`, `_evaluate_linear_expression`, and `_format_constraint_eq`.
- `src/tabdat/models.py::TestCommand` and `TestResult`.
- `tests/test_statistical_testing.py::test_wald_and_f_tests`, `test_testing_unsupported_state_fails`, and `test_nonlinear_constraints_rejected`.

Recovered behavior:

1. `test` requires the current estimation result, not a live data query. With no fit it reports `no active estimation results found`.
2. Each parsed constraint is an affine expression over fitted parameter names. The supported forms are numeric constants, identifiers, unary minus, addition/subtraction, multiplication with at most one coefficient-dependent operand, and division by a coefficient-independent nonzero expression. Nonlinear multiplication/division, functions, unsupported expression kinds, unknown names, and division by zero preserve the existing Python diagnostics.
3. For each constraint \(g_j(\beta)=0\), evaluate \(c_0=g_j(0)\), set row \(R_j\) to \(g_j(e_i)-c_0\) for each fitted parameter basis vector, and set \(r_j=-c_0\). Constraint labels display a top-level subtraction as `left = right`; other expressions display as `expression = 0`.
4. Compute \(W=(R\hat\beta-r)'(RVR')^{-1}(R\hat\beta-r)\). A singular restriction covariance reports `constraints are collinear or singular`.
5. With positive residual df, return \(F=W/q\), its upper-tail F probability with \((q, df_{resid})\), numerator df \(q\), residual df, and `is_chi2=false`. The zero/non-positive-df branch returns \(W\) and an upper-tail chi-square probability; it cannot be reached through the current Rust regression fitter.
6. The result is read-only and contains the formatted constraints, statistic, p-value, degrees of freedom, optional residual degrees of freedom, and distribution flag.

## Rust contract

- Replace `Command::Test`'s unsupported-command result with an owned `ExecutionResult::Test(TestResult)` for a stored linear regression result.
- Build \(R\) and \(r\) in the exact `LeastSquaresResult.parameter_names` order, including the `intercept` name when fitted. Reuse the existing affine-expression rules used by `lincom`, while returning test-specific typed errors.
- Use `PostEstimationModel::test_linear_hypothesis` with the model's stored covariance, then calculate an upper-tail F probability without initializing DuckDB or mutating session/data/model state.
- Keep F-test metadata exact: constraint labels/order, \(q\), positive residual df, and `is_chi2=false`. Use a numerically stable survival calculation and compare it to SciPy `f.sf`; Python's `1 - f.cdf` can lose extreme-tail precision.
- Preserve existing Python diagnostics for no fit, unknown parameters, unsupported/nonlinear expressions, division by zero, and singular/collinear restrictions.
- Keep all other estimator families and the zero-df chi-square fallback deferred.

## Test contract

1. Add failing runtime tests before implementation for the no-fit error and successful parsed tests after OLS.
2. Cover a joint zero restriction (`test x1 x2`), a single equality (`test x1 = x2`), and multiple restrictions (`test (x1 = x2) (x2 = 2)`), including exact labels, q, residual df, and F-vs-chi-square flag.
3. Compare OLS, WLS, current GLS (weights \(1/\sigma\)), HC1, clustered, and no-intercept results against both pinned Python and direct statsmodels/SciPy outputs. Also cover constant/intercept constraints, unknown coefficients, nonlinear expressions, zero division, duplicate/singular restrictions, and model/data immutability after success and failure.
4. Use the six-row fixture \((x1,x2,y,w,\sigma,grp)\): \((1,10,5,1,1,a)\), \((2,12,6,2,2,a)\), \((3,15,8,3,1.5,b)\), \((4,18,9,1,1,c)\), \((5,20,11,4,3,c)\), \((6,25,12,2,2,d)\). The four cluster groups give a positive cluster reference df matching the Rust residual df.
5. Declare a per-number comparator before judging results: \(max(10^{-12},10^{-9}|expected|)\). Compare labels, distribution, and df exactly; handle non-finite values explicitly.
6. Run focused runtime/statistics tests, pinned Python tests/probes, locked workspace checks, policy checks, and hosted PR-head workflows before merge. Re-review fixes and verify merge-head `main`.

Reference anchors from the pinned contract probe and direct statsmodels/SciPy calculations:

| Fit / restriction | F | p-value | df numerator / residual |
| --- | ---: | ---: | ---: |
| OLS / joint \(x1=0,x2=0\) | 182.12068965516917 | 0.000738336718915947 | 2 / 3 |
| OLS / equality \(x1=x2\) | 6.256742705570278 | 0.08760247783654902 | 1 / 3 |
| OLS / \(x1=x2, x2=2\) | 3537.89080459769 | 0.000008724571601717684 | 2 / 3 |
| WLS / equality \(x1=x2\) | 11.770830014080122 | 0.041511830199647035 | 1 / 3 |
| GLS / equality \(x1=x2\) | 3.6233766473472566 | 0.1531000799995986 | 1 / 3 |
| HC1 / equality \(x1=x2\) | 11.073028358067353 | 0.0447987530230212 | 1 / 3 |
| Cluster / equality \(x1=x2\) | 8.789886334525322 | 0.05931620014242662 | 1 / 3 |
| No-intercept OLS / equality \(x1=x2\) | 0.8721101941505085 | 0.40323939879340204 | 1 / 4 |

## Implementation mapping

- **Rust:** `tabdat-runtime` owns command dispatch, affine restriction construction, labels, typed result/error, and the read-only session boundary. `tabdat-stats` owns the F upper-tail probability alongside the existing Wald matrix calculation.
- **DuckDB:** not used for post-estimation calculation; fixture loading is only for integration tests.
- **Native/optional backend:** none.
- **Deferred:** zero-df chi-square fallback, unsupported model families, and all non-library presentation surfaces.
