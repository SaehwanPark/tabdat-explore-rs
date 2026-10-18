# regress

How to invoke:
`regress y x1 x2 [, robust cluster(var) noconstant wls(var) gls(var)]`
or `regress y x1 x2 if <predicate> [, ...]`

What it does:
Fit an ordinary, weighted, or generalized least-squares regression. With `if`,
fit the same model on the TRUE predicate cohort without filtering active data.

What problem it answers:
How do I model a continuous outcome with linear predictors, optionally for a
selected cohort?

Examples:
- `regress cost age bmi`
- `regress cost age bmi if eligible == 1, robust`

Conditional notes:
- The bounded predicate selects TRUE rows only (FALSE/UNKNOWN excluded) before finite outcome/predictor, cluster and weight validation. The estimation sample retains original active-row indices, total rows and exclusion counts with aligned weights/clusters.
- Active data, labels, named relations and source stay unchanged. A successful fit replaces the cached model; a failed attempted regression clears it. `lincom`/`test`/`estat vif` use the retained selected design; `predict` predicts current data.
- Predicates match bounded `keep if` rules: comparisons/arithmetic, quoted names/string literals, Boolean columns and SQL-NULL equality. Numeric truthiness, function calls and logical breadth are uninstalled.

Reporting notes:
- Unweighted OLS reports a Gaussian working-model log likelihood using ML variance RSS/N, not residual variance RSS/(N-K). This is descriptive, not a Bernoulli likelihood, robust LR inference or cross-family LR comparison. Weighted/GLS/IV and zero/nonfinite residual variance have no reported likelihood convention.
- CLI-v1 includes selected coefficient covariance with explicit row/column parameter names; this does not expose training rows or change inference.
- With binary outcomes, OLS is a linear probability model: `predict ..., xb` remains unbounded, not clipped or relabeled probabilities. Binary logit/probit use different likelihood/covariance conventions; compare their assumptions explicitly.
- `examples/mma14p1binary/comparison.td` fits six classical/robust logit/probit/OLS variants on the same fishing cohort. Reusable stored-fit comparison table commands are not supplied.

Links:
- `docs/microecometrics_topics.md`