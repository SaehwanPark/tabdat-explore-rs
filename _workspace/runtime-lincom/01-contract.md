# Contract: bounded linear-regression `lincom` runtime

## Status and scope

- **Producer / consumer:** task owner / implementer, reviewer, and next maintainer.
- **State:** in progress; target is read-only library-runtime `lincom` after successful Rust OLS/WLS/GLS `regress` states, including supported robust/cluster covariance results. Other Python estimator families and interface rendering remain deferred.
- **Rust base:** `main` at `8d0edb37a0c860ad0d06edbefa618550157241c7`; working branch `feat/runtime-lincom`.
- **Python oracle:** TabDat 0.25.0, commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree `601b236788872323af9277d2276a236154a0f129`; clean checkout and lock digest reverified.
- **Trusted reference:** statsmodels `0.14.6` OLS covariance/parameters plus SciPy `1.17.1` Student-t tail probability and 97.5th percentile, supplemented by the pinned TabDat implementation.
- **Roadmap:** Phase 7 §9.2, `Port lincom` (the full cross-family item remains unchecked unless all broader behavior is covered).
- **Selected skills:** `tabdat-migration`, `tabdat-statistical-validation`, `simple-code-writer`, `spec-driven-developer`, `code-reviewer`.
- **Risk:** medium — expression-to-parameter mapping and covariance-based inference must preserve coefficient order and residual degrees of freedom.

## Python contract recovered

Authoritative implementation/tests:

- `src/tabdat/executor.py::_execute_lincom` obtains active estimation coefficients, covariance, and residual df; evaluates the affine expression; derives its coefficient vector by evaluation at zero/unit vectors; and computes estimate, SE, test statistic, p-value, and 95% CI.
- `src/tabdat/executor.py::_get_active_estimation_results` supplies `intercept` followed by ordered predictors for linear regression.
- `src/tabdat/executor.py::_evaluate_linear_expression` defines supported arithmetic and nonlinearity failures; `_format_expression` defines result labels.
- `tests/test_statistical_testing.py::test_lincom_estimation`, `test_testing_unsupported_state_fails`, and `test_nonlinear_constraints_rejected`; parser characterization is in `tests/test_parser.py`.
- Pinned selection: **3 passed, 5 deselected**. Oracle runtime is Python 3.13.15, statsmodels 0.14.6, SciPy 1.17.1.

Recovered behavior:

1. `lincom <expression>` requires a current estimation result but not a dataset query. For the selected Rust boundary, only a successful prior linear `regress` state is available. No state reports `ExecutionError("no active estimation results found")`.
2. Identifiers must name fitted parameters, including `intercept` when present. Unknown names report `variable '<name>' not found in active model coefficients`.
3. Supported expressions are affine in model coefficients: numeric constants, identifiers, unary minus, addition, subtraction, multiplication where at most one operand depends on coefficients, and division by a coefficient-independent nonzero expression. Multiplying two coefficient-dependent terms and dividing by a coefficient-dependent term produce the recovered nonlinear errors; division by zero and function/unsupported-expression errors are deterministic.
4. For the expression `a + c'β`, estimate is `a + c'β`, variance is `c' V c`, and SE is `sqrt(max(variance, 0))`. Residual df comes from the fitted model. With positive df and nonzero SE, statistic is estimate/SE, p-value is two-sided Student-t, and CI is the 95% Student-t interval. With zero SE, statistic and p-value are NaN and both CI limits equal the estimate.
5. `LincomResult` contains `label`, `estimate`, `standard_error`, `statistic`, `p_value`, `ci_lower`, `ci_upper`, `ci_level=95.0`, and optional `df_residual`. Labels use parenthesized binary operations and Python-compatible numeric rendering.
6. The operation is read-only; failed expressions do not alter active data or fitted model state.

A pinned TabDat execution on six observations returned for `lincom x1 - x2`:

- estimate `1.9119718309859137`
- standard error `0.7643765262708596`
- t statistic `2.501348177597489`
- p-value `0.08760247783654895`
- 95% CI `[-0.5206154208903784, 4.344559082862206]`, df `3`

A direct independent statsmodels/SciPy calculation returned the same estimate, SE, statistic, CI, and p-value `0.08760247783654902` (roundoff only). Other probes covered scalar/constant expressions, intercept, unknown coefficients, nonlinear multiplication/division, and division by zero.

## Rust contract to implement

- Replace syntax-only `lincom` runtime deferral with `ExecutionResult::Lincom(LincomResult)` after a supported successful linear regression.
- Map AST identifiers to the ordered `LeastSquaresResult.parameter_names`; derive a complete affine coefficient vector and constant offset. Reject unsupported/nonlinear forms using Python-compatible messages.
- Use `PostEstimationModel` and its covariance matrix, preserve existing model state, and calculate Student-t p-values / 95% intervals using residual df. Do not initialize DuckDB or mutate the active relation.
- Add a pure Student-t quantile helper only if needed for CI computation and validate it against SciPy known values before use.
- Normalize missing estimation state and statistical failures into typed runtime errors with stable display text.
- Keep non-linear, IV, binary, panel, causal, and other model-family `lincom` dispatch deferred because those result families are not represented by current Rust runtime state.

## Test contract and acceptance

1. Test first: after a successful OLS fit, `lincom x1 - x2` returns a typed result; without a model the error is `no active estimation results found`.
2. Compare all result fields and label against pinned TabDat and direct statsmodels/SciPy values on a nominal, well-conditioned fixture. Declare a hybrid comparator `max(1e-10, 1e-9 * abs(expected))` before checking; exact-compare label and df; explicitly test NaN for zero-SE statistic/p-value.
3. Cover an intercept term, scalar multiplication/addition/division, constant-only expression, and no-intercept model.
4. Cover OLS and at least WLS/GLS or robust covariance to show the stored covariance is used rather than recomputed.
5. Cover unknown coefficients, nonlinear coefficient multiplication/division, division by zero, unsupported function/string/null/comparison forms, and no-model failure.
6. Verify VIF/post-estimation state and active dataset are unchanged on success and failure.
7. Run focused Rust and pinned Python selections, root locked checks, policy checks, independent review, and hosted PR-head checks before merge.

CLI/JSON/MCP formatting and other Python model-family routing are out of scope. The Phase 7 roadmap item remains unchecked unless broader family support is later delivered.

## Test-first evidence

The exact no-model runtime assertions and a successful-after-regression integration test were added before implementation. The focused Rust runs confirmed the baseline: both no-model tests return `runtime does not execute command: lincom` rather than the oracle error, and the post-regression call returns `UnsupportedCommand { name: "lincom" }`. The expected red test run therefore fails specifically on the deferred runtime behavior this slice is intended to replace.
