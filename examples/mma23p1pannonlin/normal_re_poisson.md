# Chapter 23 normal random-intercept Poisson — WIP contract

## Exact target

Complete a bounded original-source normal random-intercept Poisson workflow for
the §23.3/Table 23.1 patents illustration and the March 2005
[author program](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.do)/
[log](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.txt). Pinned
`patr7079.asc` SHA-256:
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`.
Retain all 346 source-record-order firms and 1,730 1975–79 firm-years,
including 22 zero-total firms and missing nonmodel ARDSSIC. Fit PAT on LOGR
plus intercept, with no additional covariates, weights, year effects or offsets.

Assume independent firms and conditional Poisson counts with log mean
`a+b*LOGR+u_i`, `u_i ~ N(0,sigma²)` independently of observed covariates, shared
across each firm's five outcomes. Estimate `(a,b,ln(sigma²))` by the integrated
joint likelihood, report full joint observed-information covariance and normal
inference, marginal population means `exp(a+b*LOGR+sigma²/2)` and residuals,
model/sample/convergence diagnostics, text/JSON and ordered predictions.
Conventional boundary LR inference, if included, must explicitly state its
reference and regularity. This is not the gamma-RE model, a general native
`xtpoisson`, or robust inference. Define deterministic quadrature and optimizer
bounds; prove accuracy by independent adaptive numerical integration and
quadrature refinement, with an identifiable true boundary and genuine failure
case. No runtime Python/R dependency or restorable fitted state is implied.

The historical normal-RE log prints slope `.815977`, intercept `1.156293`,
`ln(sigma²)=-1.310299`, log likelihood `−6261.9825`, and **missing SEs**; it
terminates after two displayed full-model iterations. Whether those values are
a converged integrated-likelihood solution or reflect the historical quadrature
and stopping convention must be investigated, not assumed. No exact historical
parity or inference recovery may be claimed without evidence.

## Acceptance and failure cases

1. Independently reshape and audit every original source/design row; compare
   all fitted parameters, full covariance, likelihood and marginal predictions
   with independently evaluated one-dimensional R integration, numerical
   derivatives and sensitivity to tighter integration/quadrature. Fixed
   justified tolerances only; investigate discrepancies.
2. Small inspectable synthetic fixture: validate fit/covariance and all
   predictions against independent R, text/JSON, input order and no-clobber.
   Include zero-total firms.
3. Test invalid/missing/duplicate/nonfinite/out-of-domain inputs, singular or
   cancellation-dominated information, a real dispersion boundary,
   nonconvergence and quadrature/mode/predictor limits; verify data, retained
   native fit, failed statement and export-file invariants.
4. Update SPEC, roadmap, coverage ledger and related notes only after full
   evidence. Normal-RE bootstrap, general panel commands, Figure 23.1 and the
   remaining book inventory are separate required work. WIP is not a capability.
