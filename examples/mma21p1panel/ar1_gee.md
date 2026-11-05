# Chapter21 AR(1)-working Gaussian GEE

## Preimplementation contract (WIP)

Source P4-AR1: [inventory](gls_inventory.md), `mma21p4pangls.do` lines100–107.
Original MOM sample: 532 people, all ten years1979–1988 (5,320 person-years),
`lnhr ~ 1 + lnwg`, Gaussian identity link, no weights or case deletion. Require
finite unique exact id/year/source_row keys and bounded finite model fields; do
not use row adjacency as a substitute for keyed annual pairs. Native retained
model remains pooled OLS; this is a bounded composition, not general `xtgee`.

For residuals r_it, phi=RSS/N and Pearson rho=sum_i,t=2..10 r_it*r_i,t-1 /
(9G*phi). Given rho, use the positive-definite AR(1) working matrix with entries
rho^|t-s|; update both population coefficients by GLS and recompute residuals,
phi and rho, with a finite cap and explicit convergence/score/positive-definite
checks. Starting values are native pooled OLS. Full working covariance is
`H^-1`, `H=sum_i X_i' V_i^-1 X_i`; unit-sandwich covariance is `G/(G-1)` times
`H^-1 sum_i U_i U_i' H^-1`, `U_i=X_i' V_i^-1 r_i`, not OLS CR1. Report
normal z/two-sided p/95 and slope chi-square(1), predictions and unit scores.
No temporal-gap substitution, AR1 correlation test, causal or few-cluster claim.

Acceptance cases:

1. Actual CLI load/guard → pooled start → converged GEE → working and adjusted
   unit covariance/inference → in-sample predictions/scores → no-clobber report,
   through text and typed CLI-v1 JSON. Existing pooled-OLS retained prediction
   state and all source rows/order are explicit.
2. Independent R gee4.13.30 `corstr='AR-M', Mv=1, v4.4compat=TRUE`, plus a
   separate dense-matrix iteration/score reference, checks sample, both full
   covariance matrices, phi/rho, normal tails, all5,320 predictions and all532
   unit scores. Use fixed absolute1e-9, nonzero p relative1e-8 and exact zero
   underflow. Author facts checked only at printed precision; investigate any
   differences, never widen tolerance. Preliminary R gee original probe matches
   author point/SEs, rho about0.4795901; its tol1e-13 run exhausted100 iterations,
   while tol1e-12 converged in8. Failed iterates are not a reference; independently
   certify final score and compare any converged package result.
3. Two independent small complete panels include positive and negative admissible
   rho; compare all fit/inference/score/prediction fields to R and ensure original
   source ordering, text/JSON parity and retained pooled predictions.
4. Invalid/missing/duplicate keys, incomplete years, singular/unresolved AR1
   working covariance, zero or unresolved unit-score variances, rank failure,
   forced iteration exhaustion and all export collisions fail without clobbering
   source/existing files; earlier script state/files may remain.

The source's separate AR1 B500 bootstrap omits `t(year)` in its bootstrap command
and needs independent duplicate-ID/refitting/RNG decisions; it is not included.
Unstructured correlation, full mixed reporting, historical exact iteration/precision
and remaining chapter/book requirements stay open. Production cannot depend on R
or Python. Interpret the estimate as a marginal association, not an FE/causal effect.
