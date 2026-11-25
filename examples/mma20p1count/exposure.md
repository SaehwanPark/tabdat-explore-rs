# Poisson and NB2 exposure syntax

## Objective and reference

Add `exposure(var)` to the installed `poisson` and `nbreg` commands as a
convenient way to supply a positive observation-time or population exposure.
The official [Poisson manual](https://www.stata.com/manuals/rpoisson.pdf) and
[NB2 manual](https://www.stata.com/manuals/rnbreg.pdf) define it as adding
`ln(var)` to the linear predictor with its coefficient fixed at one. TabDat's
existing `offset(var)` continues to add the supplied log-mean offset directly.

This is a syntax and model-state extension to the existing Poisson/NB2
estimators. It does not change the estimators, covariance choices, or the
separate conditional, gamma, and normal random-effects Poisson commands.

## Evidence state

The bounded `poisson` and `nbreg` options are implemented. The runnable
[`exposure.td`](../poisson-offset/exposure.td) workflow checks JSON and terminal
fit/effect/prediction/named-report output. Independent Base R 4.6.2 and MASS
7.3.66 fixtures validate exposure fits on the same varying positive synthetic
design used for the fixed-offset method references. The stats tests compare
1,900 Poisson and 2,188 NB2 quantities across three designs and model-based,
independent-row robust, and one-way clustered covariance; NB2 also checks joint
score and observed-curvature derivatives at and near the optimum. Runtime
contracts cover sample positions, current-row mean/index/PMF/CDF behavior,
invalid current exposure, retained effects, the Poisson overdispersion score,
named-report provenance, option conflicts, and failed-fit state. These are
synthetic method references, not a new textbook illustration or an exposure
analysis of the RAND data.

## Bounded syntax

```text
poisson y [xvars], exposure(var) [robust | cluster(group)] [noconstant]
nbreg y [xvars], exposure(var) [robust | cluster(group)] [noconstant]
```

For each command, `exposure(var)` and `offset(var)` are mutually exclusive and
may each be supplied at most once. The named exposure must be a numeric column.
It may also appear in `xvars`; in that case it is both an ordinary predictor
and a fixed log-exposure component.

## Estimation and sample

- For each retained row, TabDat adds `ln(exposure)` to `xb` with coefficient one.
  Exposure is not an estimated design column and does not increase the
  coefficient count, residual degrees of freedom, Wald restrictions, or NB2
  dispersion degrees of freedom.
- The outcome, predictors, exposure, and optional cluster key follow the current
  joint complete-case rule. NULL or nonfinite exposure values exclude a row,
  like NULL or nonfinite fixed offsets. Cluster-key exclusions happen before
  finite count-domain rejection.
- A finite exposure `<= 0` on an otherwise complete model/group row fails the
  whole fit before publication. TabDat does not silently drop it or clamp it.
  Invalid finite outcomes on retained rows keep the existing family-specific
  count-domain error.
- Positive exposure is converted to its natural logarithm in retained row
  order. The sample's original row indices and row order are preserved.
- Poisson retains its existing model-based inverse information, independent-row
  `N/(N-1)` score sandwich, or one-way cluster `G/(G-1)` score sandwich.
  NB2 retains its full joint beta/lnalpha observed-information covariance and
  the matching row or cluster joint-score correction. The NB2 restricted null
  uses the same log exposure and reestimates dispersion (and an intercept when
  present). No covariance, likelihood, or reference-distribution convention
  changes.

## Retained results, post-estimation, and failure behavior

- `PoissonResult`, `Nb2Result`, overdispersion and marginal-effects results,
  JSON/text output, and immutable named reports identify `exposure_variable`
  separately from `offset_variable`. The fixed coefficient remains one.
- `estat overdispersion` uses the fitted means containing log exposure.
  `estat countprob` uses the retained fitted means. The existing grouped NB2
  goodness-of-fit command remains unsupported after either an offset or an
  exposure fit.
- `estat margins` continues to report derivatives with respect to retained
  design columns while holding the fixed exposure component constant. If a
  variable serves as both exposure and predictor, the reported predictor
  derivative does not apply an additional chain rule through its log exposure.
- `predict ..., xb`, `predict ..., n`, and count PMF/CDF predictions read the
  current exposure column and add its current log value. Missing, nonfinite, or
  nonpositive current exposure produces a NULL prediction for that row. A
  missing/wrong-type source column fails before publishing any target column;
  the retained fit remains usable.
- A failed new fit clears stale estimation under the existing count-model
  contract while preserving the active dataset, labels, and row order. A failed
  prediction publishes no partial result.

## Independent validation

Use a small deterministic design with varying positive exposures. Compare
Poisson `glm(..., offset=log(exposure))` and an independently polished joint
NB2 reference using `MASS::glm.nb(..., offset(log(exposure)))`, then calculate
row and cluster score sandwiches independently. Compare exact retained row
indices, coefficients, complete covariance matrices, standard errors,
likelihoods, fitted means and indices, and applicable post-estimation results.
Use absolute tolerances of `1e-8` for coefficients, covariance, and fitted
means/indices, and `1e-7` for accumulated likelihoods; diagnose any failure
without widening tolerances.

Compare both Poisson and NB2 against an equivalent precomputed
`offset(ln_exposure)` design. Cover missing/nonfinite exclusion, zero/negative
exposure failures, count-domain ordering, changed-current-exposure predictions,
invalid current exposure rows, named-result metadata, option conflicts,
failure-state preservation, and unchanged no-exposure/offset behavior.

The reference code is development-only; production fitting adds no R or Python
dependency. This work does not add exposure semantics to weights, `xtpoisson`,
or other model families.
