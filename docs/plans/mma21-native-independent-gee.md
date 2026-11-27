# MMA §21.5.5 native independent Gaussian GEE

## Outcome

Add `corr(independent)` to the reusable Gaussian identity-link `xtgee` command.
The target is the March 2005 MMA `MOM.dat` design: `lnhr` on an intercept and
`lnwg`, grouped by person, with all 5,320 source rows and 532 people. The native
command must also support bounded unbalanced joint-finite samples because an
independent working matrix does not require a shared time grid.

For the retained sample, use `R = I`, so the coefficient estimate is OLS on
the same rows. Define the working scale as `RSS/N` and working covariance as
`(RSS/N)(X'X)^(-1)`. Define the raw person-sandwich covariance as
`(X'X)^(-1) sum_g (X_g' e_g)(X_g' e_g)' (X'X)^(-1)` without a finite-sample
multiplier. Report normal coefficient inference and a slope Wald chi-square
test, consistent with the native Gaussian GEE contract. The separate MMA SQL
composition applies `G/(G-1)` to its unit sandwich; this command must identify
its uncorrected covariance and must not silently reuse the adjusted report.

## Bounded scope

- Gaussian family, identity link, independent working correlation, explicit
  group key, intercept, 1–32 numeric slopes and no weights or time variable.
- Use the existing joint-finite model sample. Permit unequal group sizes and
  singleton groups, while requiring `G > K`, a full-rank design and positive
  residual scale. Keep current resource limits and source-row provenance.
- Retain independent correlation identity in results and named-estimate
  metadata. Preserve the exchangeable result/command wire shape when its
  default correlation is used.
- Preserve state on missing/invalid group keys, rank/variance failure and
  inference failure. Support current-data `xb` and residual predictions plus
  explicitly selected working/sandwich named reports.

## Acceptance cases

1. Parser and serde accept `corr(independent)` without `time()`, reject an
   independent fit with a time option, and keep old exchangeable command JSON
   unchanged.
2. On pinned original MOM data, an independent Base-R QR/group-score reference
   matches sample rows, OLS coefficients, `RSS/N`, the full working covariance,
   the full raw person-sandwich covariance, normal inference and all ordered
   predictions at fixed justified tolerances. Check the raw/`G/(G-1)` relation
   explicitly; do not label the adjusted SQL-composition result as native.
3. An unbalanced synthetic design with singleton groups matches the same
   independent reference and preserves first-seen row order and group count.
4. JSON and terminal results identify the independent method; named working and
   sandwich tables preserve their covariance conventions, and current-row
   predictions use the retained coefficients.
5. A rank-deficient or effectively zero-residual-scale fit fails atomically,
   retaining prior estimates and the active dataset; the existing exchangeable
   and Poisson GEE paths continue to pass their focused contracts.

## Exclusions

This slice does not add independent Poisson GEE, weights, time-varying
correlation, bootstrap, a combined four-correlation source table or exact
Stata 8 output parity. It does not change the chosen period-standardized
unstructured estimator, whose formula and statistical validation status remain
separate in the coverage ledger.

## Result and evidence

Implemented in the native command and documented in the
[independent GEE evidence note](../../examples/mma21p1panel/native_independent_gee.md).
The existing [Chapter21 independent-working composition](../../examples/mma21p1panel/independent_gee.md)
remains separate: its P4 sandwich applies `G/(G-1)` while this command reports
the raw grouped-score covariance, and the native command retains a GEE fit
rather than pooled OLS.

The pinned original MOM sample is checked against an independent Base-R 4.6.2
QR/group-score calculation: all 8 working/raw-sandwich covariance cells, 28
coefficient and inference values, 37,240 prediction values on 5,320 ordered
rows, and exact identity/order of 42,560 original data values. The raw native
`lnwg` sandwich SE is `0.0292408738783`; multiplying its covariance by
`532/531` matches the author-log robust SE `0.0292684` at printed precision.
The runtime fixture covers unequal group sizes and a singleton; focused
statistical, parser/wire, reporting and runtime tests cover failure, state,
prediction and named-report behavior. Existing exchangeable and unstructured
contracts remain in the focused regression set.

Exact Stata 8 output parity, independent Poisson GEE, weights, time-varying
correlation, bootstrap, and a combined four-correlation report remain excluded.
The period-standardized unstructured estimator and its unresolved historical
differences remain unchanged.
