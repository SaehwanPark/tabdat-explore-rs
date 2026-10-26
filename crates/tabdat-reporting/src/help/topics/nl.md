# nl

Fit bounded, unweighted nonlinear least squares with an **explicit mean formula**.
Both text and CLI-v1 JSON/script execution are supported.

```text
use "analysis.csv"
nl y = exp(-a-b*x), params(a b) start(1 0)
predict fitted, mean
predict residual, residuals
export csv "nls_predictions.csv"
```

Syntax:
`nl y = expression, params(name...) start(value...) [robust noconstant]`

## Formula and sample

- Supply every parameter and one finite start in the same explicit order. Names in
  `params()` are parameters, not data columns; other identifiers are numeric data
  predictors. Every declared parameter must occur in the formula.
- The formula alone determines the mean. No automatic intercept is inserted.
  `noconstant` is a retained syntax flag: it neither inserts nor removes a formula
  parameter. For a constant mean, use e.g. `nl y = exp(a), params(a) start(0)`.
- Supported expressions: finite numeric constants, data/parameter identifiers,
  parentheses, unary minus, `+`, `-`, `*`, `/`, and one-argument `exp()`.
  Powers, log/other functions, comparisons, strings, conditions, categorical
  encodings, weights, clusters and constraints are not installed.
- Use the joint finite sample of y and referenced X, in original input order.
  Missing/nonfinite unrelated columns do not drop a row. Arithmetic/domain failure
  on an otherwise eligible row is an error, not another missing-data exclusion.

For the exponential-mean illustration above, mean=exp(-a-b*x) and
rate=exp(a+b*x); they are not interchangeable. This is NLS, not exponential ML.

## Estimation and inference

The solver uses column-scaled QR Gauss–Newton with value-only backtracking.
Both normalized score and the **full undamped step** must converge; a tiny line
search step or tiny loss change alone is insufficient. Full Jacobian rank and
positive observed-loss curvature are required at the solution. A regular local
solution is not a global-optimum or uniqueness guarantee: try scientifically
meaningful alternative starts when nonlinear shape permits multiple minima.

Classical covariance is the homoskedastic working formula
`RSS/(N-P) * inverse(J'J)`. `robust` requests the independent-row HC1 Gauss–Newton
sandwich with multiplier `N/(N-P)`, **under a correctly specified conditional mean**.
The observed-loss Hessian checks the minimum; it is not substituted as covariance
bread. Coefficient t tests and 95% intervals use an approximate Student-t(N-P)
reference, not exact nonlinear finite-sample inference or a normal-reference test.
A genuine zero SE has unavailable t/p/intervals, not certain inference.

The report includes sample exclusions, RSS, residual variance, RMSE, centred R²
(when defined), covariance correction and convergence/work diagnostics. There is
no invented likelihood, LR or omnibus F test; prediction uncertainty, weighted or
feasible generalized NLS, marginal effects and generic linear-model post-estimation
are not installed for this owned fit.

## Prediction and state

`predict new`, `predict new, xb`, and `predict new, mean` all evaluate the nonlinear
formula **mean**, not a latent linear index. They use current X and do not require
current y; `predict new, residuals` additionally requires current y. Parameter-only
means require no X. Missing/nonfinite required current inputs give NULL. A complete
row's arithmetic failure fails the whole prediction without appending any column.
Existing targets (including case variants) are rejected. Successful publication
appends a typed DOUBLE/NULL column and preserves input order, original columns,
labels and the owned fit; the active named table's schema is updated normally.

An attempted fit clears previous estimation state even when it fails. A failed
prediction preserves the owned fit and data. Failed fits do not mutate data,
labels, named tables or files.

Limits: at most10000 candidate/current rows, 1..32 parameters, 128 expression
nodes, depth32; bounded per-linearization and cumulative work, 200 iterations and
60 line-search halvings. N must exceed P. Rank deficiency, zero/nonrepresentable
residual variance, nonrepresentable/erased nonzero arithmetic and failure to
converge are errors; no pseudoinverse, ridge, clipping or approximate-success
fallback is used. Unsupported options are rejected, not ignored.
