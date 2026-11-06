# WIP: Chapter 24 individual shared-gamma RE whole-commune bootstrap

**Planning contract only; not yet an executable or validated workflow.** The
required bounded outcome is a read-only pairs bootstrap of native shared-gamma
Poisson RE, followed by original marginal prediction and export.

## Exact sample, design and independent reference

Use pinned `vietnam_ex2` and the ten ordered numeric predictors described in
[individual_gamma_re.md](individual_gamma_re.md), plus an intercept and
ln(alpha). Joint-finite selection excludes the wholly missing row and retains
**27,765 people in 194 communes**, including all 94 all-zero individuals in
commune64. The author log's RE bootstrap reports the same N and group count.
For each draw sample 194 whole communes with replacement, assigning every copy
its own shared gamma effect, and refit all 12 parameters including dispersion.
Repeated source-ID copies must not be merged into one longer RE group: that
changes the integrated likelihood. The author program uses the older
`bootstrap ..., cluster(COMMUNE)` form without visible `idcluster()`; as in the
[Chapter23 gamma bootstrap](../mma23p1pannonlin/gamma_re_bootstrap.md), its
historical repeated-ID semantics are not established. This distinct-copy
workflow will make no RNG or historical bootstrap-uncertainty parity claim.

## Acceptance, failure cases and exclusions

- Fit and retain original native gamma RE, then report original/refit parameter
  vectors, all ordered group draws, variable sample sizes, full B−1 covariance,
  SE/bias and marginal type7 percentile95 intervals via text and CLI-v1 JSON.
- On both pinned publisher ASCII and precise DTA, independently verify source,
  all ordered seeded draws, all B500 refits and full covariance using a base-R
  integrated-likelihood/score reference; check all original-model marginal
  prediction rows and printed author original-point facts. Cross-check distinct
  raw-copy and multiplicity formulations; do not compare B500 uncertainty with
  the pinned B50 author log as if RNGs matched. Fix tolerances before auditing.
- Exercise unbalanced and singleton groups, zero-total groups, structural-key
  and exact-count handling, deterministic group order, large seed serialization,
  state/label/named-report preservation and input/export ownership.
- Boundary/failed original or resampled fits must abort with the replicate index
  without redraw, pooled fallback, partial result or retained-state mutation.
  Resource, dispersion and convergence bounds must be explicit and tested.
- No weights, offsets, normal RE, posterior effects, GEE, survey-design,
  bias-corrected/studentized intervals or historical RNG parity is implied.

Prototype feasibility, implementation, independent evidence and executable
commands are still required before this is claimed complete.
