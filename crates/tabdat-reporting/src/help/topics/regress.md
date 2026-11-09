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
- `regress lnwage educ i.region i.year`
- `regress lnwage educ ib(North).region`
- `regress lnwage i.region, noconstant`

Factor terms:
- `i.var` declares one explicitly categorical `var`. The first sorted retained
  level is the omitted reference; the remaining levels enter as indicator
  contrasts named `<var>_<level>` (for example `region_North`).
- `ib(level).var` omits the named level instead. The level may be written with
  quotes (`ib('North East').region`) and matched by exact label, or by an
  equivalent numeric spelling such as `ib(1.20)` for a DECIMAL `1.2` level.
- Text, boolean and finite numeric codes are accepted. Numeric codes sort
  numerically and text levels sort lexically; level identity uses the exact
  source label, so adjacent wide `DECIMAL`/`UBIGINT` codes that round to one
  `f64` stay separate categories.
- Levels are learned from the retained estimation sample only, so rows excluded
  by missingness or an `if` predicate cannot create empty columns.
- `noconstant` keeps every level (no contrast is estimated) and rejects
  `ib(level).var`, which would have nothing to contrast against.
- A factor with one retained level in an intercept model, and a reference level
  absent from the estimation sample, fail with explicit messages.
- Only `regress` accepts factor terms. Other estimators and commands such as
  `summarize`, `logit` or `bootstrap` still reject `i.var`.

Prediction after a factor fit:
- `predict newvar` reconstructs the retained contrast columns from the current
  data, so it never looks for columns named like expanded coefficients.
- A missing factor cell, or a level first seen after the fit, yields NULL for
  that row. No new level is silently folded into the reference.
- `predict ..., residuals` subtracts the same reconstructed index from the
  current outcome.

Stored reports:
- `estimates store` retains the ordered design terms for factor fits, and
  `estimates table` prints an explicit reference-level row. Continuous-only fits
  keep the established report shape without design metadata.

Conditional notes:
- The bounded predicate selects TRUE rows only (FALSE/UNKNOWN excluded) before finite outcome/predictor, cluster and weight validation. The estimation sample retains original active-row indices, total rows and exclusion counts with aligned weights/clusters.
- Active data, labels, named relations and source stay unchanged. A successful fit replaces the cached model; a failed attempted regression clears it. `lincom`/`test`/`estat vif` use the retained selected design; `predict` predicts current data.
- Predicates match bounded `keep if` rules: comparisons/arithmetic, quoted names/string literals, Boolean columns and SQL-NULL equality. Numeric truthiness, function calls and logical breadth are uninstalled.

Reporting notes:
- Unweighted OLS reports a Gaussian working-model log likelihood using ML variance RSS/N, not residual variance RSS/(N-K). This is descriptive, not a Bernoulli likelihood, robust LR inference or cross-family LR comparison. Weighted/GLS/IV and zero/nonfinite residual variance have no reported likelihood convention.
- CLI-v1 includes selected coefficient covariance with explicit row/column parameter names; this does not expose training rows or change inference.
- Factor fits also report the retained levels and reference level in text, and additive `design_terms` in JSON. Continuous-only fits omit `design_terms` so existing envelopes keep their shape.
- With binary outcomes, OLS is a linear probability model: `predict ..., xb` remains unbounded, not clipped or relabeled probabilities. Binary logit/probit use different likelihood/covariance conventions; compare their assumptions explicitly.
- `examples/mma14p1binary/comparison.td` fits six classical/robust logit/probit/OLS variants on the same fishing cohort. Reusable stored-fit comparison table commands are not supplied.

Links:
- `docs/microecometrics_topics.md`
