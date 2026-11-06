# WIP: Chapter 24 conditional-FE Poisson whole-commune bootstrap

**Planning contract, not yet an executable or validated workflow.** The required
bounded outcome is a read-only pairs bootstrap of the existing native
conditional Poisson FE, followed by original-fit prediction and export.

## Exact sample and design

Use the pinned `vietnam_ex2` individual data and ordered ten predictors described
in [individual_conditional_fe.md](individual_conditional_fe.md). Joint finite
selection excludes the wholly missing record; the conditional model then excludes
zero-total commune 64. The author log's conditional-FE bootstrap reports 27,671
observations and **193** clusters, so the sampling frame is the original positive-
total FE estimation sample, not all 194 complete-case communes. Each draw copies
all rows in a sampled commune; repeated copies have distinct FE identities. Refit
all ten slopes, not a fixed-effects level, offset, or fixed coefficient. The
original author log uses 50 replications; this planned seeded 500-draw workflow
will not claim historical RNG or bootstrap-uncertainty parity.

## Acceptance and exclusions

- Fit, retain and diagnose the original FE model; sample its positive-total
  groups in documented deterministic order; expose ordered draws, copied sample
  sizes, all refit slopes, full B−1 covariance, bias/SE and marginal type7 95%
  percentile intervals in text and CLI-v1 JSON.
- Independently check both publisher ASCII and precise DTA against a separately
  implemented R conditional-score/information reference, with independent
  Python RNG indices, original author point facts and all prediction/exclusion
  rows. On a small fixture, cross-check selected copied fits against an R
  dummy-Poisson GLM. Fix tolerances before examining results.
- Test unbalanced groups, positive singleton and excluded zero-total group,
  exact keys, missingness, full-width seed, source order, read-only model/data,
  and source/export ownership. An identified original whose resample is
  singular or separated must fail without redraw or partial report and preserve
  the retained original state.
- Use explicit resource and rejection bounds after measuring feasibility; no
  offsets, weights, general panel resampling, RE/bootstrap-GEE, survey design,
  bias-corrected/studentized intervals or historical RNG parity is implied.

The precise numerical method, tested limits, reproducible command and independent
reference results remain to be implemented and audited before this is claimed.
