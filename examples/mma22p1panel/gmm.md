# Planned Table 22.2 base-instrument panel GMM slice

**Status: preimplementation; not an installed or validated command.** Reproduce
§22.3's base-instrument column on 4,256 person-years/532 people, using the
existing complete-panel source preparation, five differenced endogenous
regressors, nine lagged-level instruments and no intercept. The author program
`mma22p1pangmm.do` specifies 2SLS pilot residuals u₁ and uncentered cluster sums
`S₁ = Σᵢ (Zᵢ′u₁ᵢ)(Zᵢ′u₁ᵢ)′`. With `A=Z′X`, `b=Z′y`, estimate
`β₂=(A′S₁⁻¹A)⁻¹A′S₁⁻¹b`. It then recomputes `S₂` from β₂ residuals and reports
`(A′S₂⁻¹A)⁻¹` and `u₂′Z S₂⁻¹ Z′u₂`. These are **not** the finite-sample
fixed-weight sandwich and objective used by standard two-step GMM. Do not call
them those quantities or silently replace the author's conventions.

Proposed bounded executable slice: explicit `ivregress gmm` with required one-way
cluster, no intercept and no included exogenous variables, complete joint-finite
sample, full-rank design and moment covariance, at most 10,000 rows, 8 regressors
and 16 instruments. Retain an owned GMM state distinct from 2SLS; current-data xb
and residual predictions, clearly identified covariance and overidentification
results, text/script/CLI-v1 JSON. Report both standard fixed-weight two-step
sandwich/objective and explicitly labelled author-updated approximations, with
normal and chi-square asymptotics as applicable and no finite-sample correction.
Reject unsupported combinations, singular or unidentified designs and too few
clusters; no pseudoinverse, weights, GMM iterations, clustered finite-sample
correction or stacked 72-instrument claim. The pilot and updated fits reject
maximum absolute residual at or below 128 machine epsilons times the larger
maximum absolute response or fitted value, rather than treating floating-point
noise from an exact fit as an invertible moment covariance. Exact result and post-estimation
contracts must be checked against current state/JSON behavior before implementation.

Acceptance: (1) source design, coefficients, both full covariance matrices,
both objective statistics and all predictions against independent R and selected
author-log facts at documented justified tolerances; (2) small synthetic
independent-reference case including just-identified behavior; (3) singular
instrument/cluster moment and unsupported-option failures with explicit state
and output invariants; (4) executable text and JSON script workflow. Historical
Stata float precision, weak instruments, asymptotic assumptions and the lack of
causal or entire-table guarantees remain explicit. Do not merge a kernel-only
milestone as this workflow.

## Preliminary independent reference

R 4.6.1 [`gmm_reference.R`](gmm_reference.R) uses the independently generated
base design from `reference.R`, scales X and Z columns, obtains the pilot by QR,
and calculates both covariances and objectives by direct solves. On the pinned
original data it yields wage β=0.5467960797, fixed-weight SE=0.3297578764,
author-updated SE=0.3276239308, fixed-weight J=5.3264914492 and updated
J=5.4503865499 (four overidentifying degrees of freedom). The author log
reports β=0.54679602, updated SE=0.32762396 and updated J=5.4503878; the
small differences are not evidence for digit-level historical parity. The
standard and author-updated quantities are materially distinct, not alternative
labels for the same finite-sample computation. This reference run is preliminary;
it does not validate a native implementation.
