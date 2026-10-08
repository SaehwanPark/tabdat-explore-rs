# Evidence and migration map: bounded `lincom` runtime

## Oracle identity and baseline

The external Python oracle was verified before execution:

- Commit: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`
- Tree: `601b236788872323af9277d2276a236154a0f129`
- Worktree: clean
- `uv.lock` SHA-256: `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`
- Runtime: Python 3.13.15, statsmodels 0.14.6, SciPy 1.17.1

Pinned baseline command:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_statistical_testing.py \
  -k 'lincom_estimation or nonlinear_constraints_rejected or testing_unsupported_state_fails'
```

Result: **3 passed, 5 deselected**. The full `tests/test_statistical_testing.py` module also passed (**8 passed**). Direct executor probes on the pinned checkout confirmed all eight runtime error messages covered below. Numeric-label probes confirmed Python renders `0.00001` as `1e-05` and `10000000000000000.0` as `1e+16`.

## Three-way numerical comparison

The runtime integration fixture has six observations:

| x1 | x2 | y | w | sigma | group |
|---:|---:|---:|---:|---:|:---|
| 1 | 10 | 5 | 1 | 1 | a |
| 2 | 12 | 6 | 2 | 2 | a |
| 3 | 15 | 8 | 3 | 1.5 | b |
| 4 | 18 | 9 | 1 | 1 | c |
| 5 | 20 | 11 | 4 | 3 | c |
| 6 | 25 | 12 | 2 | 2 | d |

The verified OLS combination is `lincom x1 - x2`. For the independent reference, `statsmodels.api.OLS` fits the design with an intercept; SciPy computes two-sided Student-t tails and the 97.5th percentile. The formula is `estimate = c' beta`, `SE = sqrt(c' V c)`, `t = estimate / SE`, `p = 2 * t.sf(abs(t), df_resid)`, and `CI = estimate ± t.ppf(.975, df_resid) * SE`. Direct `statsmodels.api.WLS` with `weights=w`, WLS with `weights=1/sigma` for current GLS semantics, HC1 and cluster covariance, and no-intercept OLS also provide independent reference results.

| Fit | Estimate | SE | t | p | 95% CI | df |
|:---|---:|---:|---:|---:|:---|---:|
| OLS | 1.9119718309859137 | 0.7643765262708596 | 2.501348177597489 | 0.08760247783654902 | [-0.5206154208903784, 4.344559082862206] | 3 |
| WLS (`w`) | 2.2141020932794717 | 0.6453481932756083 | 3.430864324638927 | 0.041511830199647035 | [0.16031611996799588, 4.267888066590947] | 3 |
| GLS (`sigma`) | 1.5992544268406328 | 0.8401577174716867 | 1.9035169154350209 | 0.1531000799995986 | [-1.0745023969827296, 4.273011250663995] | 3 |
| OLS HC1 | 1.9119718309859137 | 0.5745770609021545 | 3.327616017221241 | 0.0447987530230212 | [0.08341118641708012, 3.7405324755547475] | 3 |
| OLS cluster (`grp`) | 1.9119718309859137 | 0.6448962625217005 | 2.9647742468062086 | 0.05931620014242662 | [-0.14037589696754393, 3.9643195589393714] | 3 |
| OLS no intercept | -0.41664869527711823 | 0.4461535414752152 | -0.9338684030153865 | 0.403239398793402 | [-1.6553695116726401, 0.8220721211184037] | 4 |

The pinned TabDat executor and the direct reference agree within floating-point roundoff. Rust integration tests compare all six modes with the predeclared hybrid threshold `max(1e-10, 1e-9 * abs(expected))`; labels, 95% level, and df are exact-compared. OLS is also checked for `x1 + 2*x2`, `intercept + x1`, `2 + x1`, `x1 / 2`, unary `-x1`, and constant-only `2.0`.

## Distribution reference

`student_t_quantile` was compared against SciPy values:

- `t.ppf(0.975, 3) = 3.1824463052837055`
- `t.ppf(0.975, 10) = 2.2281388519649385`
- the df=3 lower-tail symmetry and `student_t_pvalue(t.ppf(.975, 3), 3) = 0.05` also pass.

The pure post-estimation contract uses the regularized-incomplete-beta two-tailed Student-t probability and bisection over its CDF for quantiles. The pinned Python code forms its p-value as `2 * (1 - scipy.stats.t.cdf(...))`; for `t=1e10, df=3`, that expression rounds to `0.0`, while SciPy's survival function and Rust both return approximately `2.205315581687166e-30`. Rust intentionally follows the stable two-tailed reference (`student_t_pvalue`, also used by the existing statistics kernel) rather than reproducing avoidable CDF-subtraction cancellation. The extreme-tail value has a dedicated Rust reference assertion; ordinary and robust fixture values agree within the comparator above. This is an intentional numerical difference, not a tolerance widening.

## Error and state evidence

Pinned Python and Rust agree on these execution diagnostics:

| Command after a fitted model | Error text |
|:---|:---|
| `lincom x3` | `variable 'x3' not found in active model coefficients` |
| `lincom x1 * x2` | `nonlinear coefficient multiplication is not supported in linear testing` |
| `lincom x1 / x2` | `nonlinear coefficient division is not supported in linear testing` |
| `lincom x1 / 0` | `division by zero in expression` |
| `lincom abs(x1)` | `functions are not supported in linear testing` |
| `lincom null` | `unsupported expression type in linear testing` |
| `lincom "text"` | `unsupported expression type in linear testing` |
| `lincom x1 == 0` | `unsupported binary operator: ==` |

Without a fitted result, the typed runtime error displays `no active estimation results found`. Tests also execute a valid combination after projecting the active relation down to `x1`, and verify both the active metadata and stored fit remain identical across successful and failed `lincom` calls.

## Implementation mapping

- `crates/tabdat-runtime/src/lib.rs`: typed `LincomResult`, `ExecutionResult::Lincom`, read-only dispatch from `Session::last_regression`, affine expression analysis, Python-compatible labels and diagnostics.
- `crates/tabdat-stats/src/post_estimation.rs`: offset-aware covariance combination and typed Student-t inference with zero-SE point interval.
- `crates/tabdat-stats/src/matrix.rs`: pure Student-t quantile backed by the existing two-sided probability helper.
- `crates/tabdat-runtime/tests/lincom_contract.rs` and `lincom_runtime_contract.rs`: no-model, AST, estimator modes, inference values, state invariants, and expression errors.
- `crates/tabdat-stats/tests/linear_combination_inference_contract.rs`: known quantiles, offset/covariance arithmetic, zero SE, and invalid-df/confidence boundaries.

No DuckDB relation mutation, estimator/backend initialization, CLI, JSON, or MCP result adapter is added. Other Python estimator families and the normal fallback for missing/non-positive residual df remain deferred; current Rust least-squares fits require positive residual df.
