# Chapter 27 supplied-imputation summary pooling — WIP contract

## Outcome and source

Deliver bounded **marginal** Rubin pooling for the Chapter27.8.1 SAS
`mma27p2milinear.sas` / `mma27p2milinear.lst` case1 linear demonstration.
The author's published log contains ten per-imputation regression estimates
and marginal standard errors, but not the ten completed data sets. Accept a
long, rectangular table of supplied per-imputation estimates/SEs, and reproduce
the author's pooled marginal means, within/between/total variance, conventional
old-Rubin t/DF, 95% intervals and missing-information summaries. The source log
is SHA256 `0d0633bf3758205423cd909fb20a6b05c6c19ed729eca35e05ad7ce379ef7074`;
the SAS program is `56f85ecf888cf2c878d9df4f2015fcdb98d7df19d4e2799b087027ceeb637f8b`.
Its printed estimates and SEs are rounded to five decimals, unlike the unrounded
values used by `PROC MIANALYZE`; require rigorous independent R validation for
exact supplied numbers and report historical comparisons separately. No raw
completed imputations or author data are redistributed.

This is pooling **already computed** estimates, not creating imputations or
fitting each regression. It cannot recover cross-parameter covariances from
marginal SEs and will not offer joint tests, prediction, original fit state,
finite-complete-data DF adjustment, general MI commands, or claim valid MI
without proper imputations, congenial analysis models and missing-at-random
assumptions. The Chapter27 simulation is generated data; original book seed and
literal historical draws differ. The preceding [single-fill comparison](README.md)
is not multiple imputation and remains separate.

## Inference and boundaries

For m>=2 estimates Q_j and positive within-imputation SE_j, let Qbar=mean(Q),
W=mean(SE²), B=sample variance(Q), T=W+(1+1/m)B. If B>0, `r=(1+1/m)B/W`,
`df=(m-1)*(1+1/r)^2`; use Student-t(df), pooled SE=sqrt(T), t=Qbar/SE, two-sided
p and t95 limits. Missing information is `(r+2/(df+3))/(1+r)` and relative
efficiency `1/(1+FMI/m)`. These are the legacy large-complete-sample rules, not
Barnard–Rubin finite-complete-DF corrections. If B is exactly zero, use the
normal infinite-DF limit, `r=FMI=0` and efficiency1; report NULL DF, not a fake
finite value. Reject, do not approximate, positive B whose implied finite DF
exceeds1,000,000. Reject missing/nonfinite values, nonpositive SE, unequal sets,
duplicate keys or inadequate m before publication. Bounds and row ordering will
be documented precisely with implementation.

A small native SQL probability boundary may expose the existing validated
`student_t_pvalue` and `student_t_quantile` for finite valid arguments; invalid
or NULL arguments must be defined/tested explicitly. This is needed for real
fractional Rubin DF: do not round DF, silently replace Student t by normal or
use a different native fit's residual DF. No production R/Python dependency.

## Acceptance and failure cases

1. Independently parse the ten rounded author-log rows per parameter into a
   no-clobber input; compare all pooled quantities to base R formulas and
   compare the unrounded author report only at honestly documented precision.
   Preserve source parameter/imputation ordering and bind source-log identity.
2. Validate positive-between, exact-zero-between and small-DF synthetic inputs
   against independent R, including two-sided tails and 95% bounds with fixed
   absolute1e-9/nonzero p relative1e-8/exact underflow zero. Validate native SQL
   Student-t boundary values/null/error semantics and relevant downstream crates.
3. Actual CLI scripts report scope, covariance/DF conventions, typed JSON and
   readable text, and identical no-clobber output CSV; retained state is explicit.
4. Fail before exports on missing/duplicate/unbalanced IDs/parameters, nonfinite
   estimates, nonpositive SE, overflow or unresolved DF; fail without replacing
   inputs/owners on converter/export collisions. Exercise source, report and
   inferential forgery checks.

This is a preimplementation contract, not evidence that pooling or a general
multiple-imputation workflow is installed.
