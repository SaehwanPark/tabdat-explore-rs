# Chapter 23 gamma random-effects Poisson — WIP contract

## Bounded target and sample

Implement a complete, independently validated workflow for the gamma
random-effects Poisson column of Cameron and Trivedi (2005), §23.3/Table 23.1,
using the [author program](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.do)
and [log](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.txt).
Use the existing preparation of pinned `patr7079.asc` (SHA-256
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`):
346 source-order firms, five complete years 1975–79, 1,730 observations,
including all 22 firms with zero total patents. Missing nonmodel ARDSSIC must
not remove firms. The only regressor is LOGR with intercept; no year effects,
weights, offsets or extra covariates.

Assume independent firms and, conditionally on a persistent multiplicative
firm effect `u_i`, independent counts
`Y_it ~ Poisson(u_i * exp(a+b*x_it))`, with `u_i ~ Gamma(shape=1/alpha,
rate=1/alpha)` so E(u)=1 and Var(u)=alpha. This is one *shared* effect per
firm, not an independently mixed NB2 count for each row. Maximize the joint
integrated panel likelihood over `(a,b,ln(alpha))` at an interior finite
solution; validate joint observed-information covariance, all cross terms,
normal inference, likelihood and marginal mean/residual predictions. The
joint likelihood can also be checked by decomposing each panel into a negative
binomial total and a conditional multinomial timing component. The supplied
[`xtpoisson` manual](https://www.stata.com/manuals/xtxtpoisson.pdf) specifies
the gamma and distinct normal random-effect conventions.

The author log reports slope `.3487832`, intercept `2.312705`, ln(alpha)
`.5454692`, SEs `.0332254`, `.124758`, `.0899144`, and log likelihood
`-5553.1787`; historical numerical/storage differences must be investigated,
not hidden by widened tolerances. The separate normal-RE model, bootstrap,
general `xtpoisson`, robust inference, Figure 23.1 and other chapter analyses
remain required and are not implied by this slice.

## Acceptance and failure cases before implementation

1. Original data: independently check all 1,730 sample/design/prediction rows,
   all three parameters, full observed covariance, likelihood and normal
   inference against R using a separately implemented likelihood/derivatives;
   additionally audit likelihood decomposition and selected author-log facts.
2. Small inspectable synthetic data: compare original fit and covariance to an
   independent reference, including zero-total firms and input ordering.
3. Reject invalid keys, missing/nonfinite values, noninteger or negative counts,
   singular design, invalid/near-boundary dispersion, nonconvergence or
   cancellation-dominated curvature; no silent row drops, regularization or
   publication of failed fits. Test input/export and prior session-state
   invariants, no-clobber, text/JSON and sorted output.
4. Declare parameter bounds, initialization, convergence, covariance,
   likelihood constants, mean interpretation and retained state before merging;
   update SPEC, roadmap and coverage only after numerical and integration
   evidence. R is development-only, not a production dependency.

Implementation approach is intentionally open pending independent numerical
and platform probes; a WIP contract is not executable estimator support.
