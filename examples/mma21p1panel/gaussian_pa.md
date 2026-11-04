# Chapter21 Gaussian population-averaged GEE

## Preimplementation contract — in progress

Target C21-P1-PA: first author program mma21p1panfeandre.do lines307–308,
original MOM.dat SHA256
`2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77`,
N5,320/G532/T10, years1979–1988, marginal lnhr on intercept and lnwg.

Compose native SQL Gaussian identity-link exchangeable GEE with the legacy
Pearson correlation moment and RSS/N scale. Independent R gee4.13.30
`v4.4compat=TRUE`, explicit OLS start, free scale, tolerance1e-12 reproduces the
author's printed working SEs and scale; modern-default corrections differ and
must not be substituted silently. Permit admissible negative correlation,
not just positive random-intercept variances. Retain prior native pooled OLS
state explicitly; there is no new owned GEE estimator or generic predict API.

Acceptance:
1. Original source binding, independently validated GEE coefficients, working
   and separately labelled unadjusted unit-score sandwich covariance/normal95,
   phi/rho/score certificates, every source prediction and unit score.
2. Readable/CLI-v1 coefficient tables, reproducible CSVs, JSON/text byte parity,
   source-row order and explicit retained pooled-OLS model semantics.
3. Small positive/negative correlation controls with full covariance and failure
   cases for missing/duplicate/incomplete panels, singular correlation/design,
   failed convergence and no-clobber ordered exports.

Bounds: 3–532 complete ten-year people, finite model values<=20 in absolute
magnitude, exact bounded person/year/source keys, finite identified within and
between wage designs, strict positive-definite working correlation and bounded
100-step convergence. Fail instead of clipping correlation or dropping cases.
No production R/Python dependency. No p-values (SQL lacks a trusted normal-tail
primitive), likelihood/LR, weights/unbalanced panels, causal/few-cluster guarantee,
general GEE API or literal historical iteration/output replay. Coefficient z,
normal95 intervals and labelled variance conventions provide bounded inference;
remaining historical outputs stay explicit. No capability is claimed before
implementation and independent checks complete.
