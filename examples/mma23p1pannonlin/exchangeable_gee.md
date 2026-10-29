# Chapter 23 exchangeable Poisson GEE — WIP contract

## Exact target and conventions

Reproduce the author program's §23.3/Table 23.1 exchangeable Poisson
population-averaged/GEE fit and its working and firm-robust inference, not a
general native `xtgee` command. Pinned `patr7079.asc` SHA-256
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`:
346 source-order firms, all 1,730 records in 1975–79, including 22 zero-total
firms; outcome PAT, design `(1,LOGR)`, `mu=exp(a+b*LOGR)`, no extra regressors,
weights or offset. Supply native pooled Poisson starts, compute a bounded
composed GEE fit, diagnose/report and export every original row with fitted
mean/residual; text and CLI-v1 JSON agree. No restorable GEE estimation state.

Balanced T=5. At each iterate Pearson residual `e_it=(y_it−mu_it)/sqrt(mu_it)`;
exchangeable working correlation `rho = sum_i[(sum_t e_it)^2−sum_t e_it^2] /
(4*sum_it e_it^2)`. Fix Poisson scale 1 and use
`R=(1−rho)I+rho*11'`, `V_i=A_i^1/2 R A_i^1/2`, `D_i=diag(mu_i) X_i`,
`U=sum D_i' V_i^-1 (y_i−mu_i)` and working bread
`B=sum D_i' V_i^-1 D_i`. Iteratively update beta by the full Fisher-scoring
step `B^-1 U`, recomputing rho, with explicit finite, positive-definiteness,
step/score and iteration bounds. Do not clamp rho, ridge, silently switch
working correlations, return a failed iterate or drop rows/firms.

Report `B^-1` (scale1) and `G/(G−1) B^-1 sum_i U_i U_i' B^-1` as **distinct**
working and firm-sandwich covariances, normal z/95% intervals and rho, with no
p-values unless independently accurate tails are implemented. Working inference
requires the working mean/variance/correlation; firm inference assumes the
conditional mean and independent firms. Neither repairs mean misspecification,
implies causal identification, or guarantees few-cluster reliability.

Author log: slope `.5595302`, intercept `2.067515`, working slope/intercept SE
`.0043803`/`.0185166`, robust `.0326436`/`.1113256`. Preliminary R `gee` 4.13-30
with `v4.4compat=TRUE`, scale fixed 1 and Poisson start gives slope `.5595301`,
intercept `2.067515`, rho `.8983667`; its uncorrected sandwich becomes the
historical robust SE after multiplying variance by G/(G−1). The default modern
`gee` correlation uses different degrees-of-freedom moment corrections and
produces slope `.5603399`: do not conflate these conventions or widen tolerance.

## Acceptance and failure controls

1. Independently compare original source sample/design, every mean/residual,
   rho, both coefficients/full covariances/SEs/intervals and selected author
   log values against R `gee` 4.13-30 (legacy correlation, fixed scale1) and a
   separately coded base-R estimating-equation/cluster-score calculation.
   R/Python and author data remain development-only. Numerical tolerances are
   fixed and justified; discrepancies must be investigated.
2. A small inspectable synthetic panel checks all rows, both covariance matrices,
   current text/JSON and order/file invariants. Include a distinct negative-rho
   case, if feasible, and compare to the same independent R convention.
3. Test invalid/duplicate/missing/nonfinite data, zero Pearson variance, rho at
   or numerically near either exchangeable positive-definiteness boundary,
   singular or cancellation-dominated bread, invalid predictor and bounded
   nonconvergence; no export/overwriting on failure. Explicitly describe prior
   native Poisson/prediction commands that can remain after a later script error.
4. Update SPEC/roadmap/ledger only with executable and independent evidence;
   run focused and affected tests, final diff review, documented PR and safe merge.

Bounds planned: integer IDs 1–346, exactly 1975–79, count 0–1,000,
`|LOGR|≤20`, at least two firms, `|a|,|b|≤20`, every linear predictor in
[−40,40], 100 iterations and 128-epsilon scale-aware guards for working
correlation/bread boundaries. These are deliberate bounded workflow semantics,
not claims about all possible valid GEE datasets. If the composition or reference
cannot meet them, investigate and stop rather than relaxing checks for parity.
Random-effects Poisson, general GEE, other author bootstraps, figure and the
rest of the textbook remain required.
