# Individual pooled Poisson whole-commune bootstrap: WIP contract

**WIP: not yet an executable validated workflow.** This targets the first
individual bootstrap block in `mma24p2poiscluster.do`, §24.7. The publisher
notes that the B500 output at the website is not reported in Table24.6; the
program itself sets `breps` to50. Conditional-FE and shared-gamma-RE individual
bootstraps, survey design and other illustrations remain required.

## Exact sample and target

Publisher ASCII SHA256 `e876f8ee17791881c6378de2761436e1e72e1eda84347089a9dc365ed290c85d`;
DTA SHA256 `284a5dd732926fe6ee02157012b17d8751523b89f1dd2f652a6f4dd021f3dac3`.
Outcome `pharvis`; regressors in order `lnhhexp insurance sex age married
illdays actdays injury illness educ` and an intercept. Source-row order is an
identity key, not time. Of27,766 source rows the one wholly missing row is
excluded, leaving27,765 people in194 communes; retain all zero-count people and
the all-zero commune. The source mapping and numeric codes are documented in
the [pooled workflow](README.md).

Fit the original pooled log-link Poisson, then draw194 whole communes with
replacement 500 times, retaining every individual in each copied group and
refitting all eleven coefficients on each draw. A copy's point contribution is
its original group's row likelihood; no FE/RE grouping semantics are implied.
Report original coefficients, B−1 covariance, SE, bias and type7 percentile95
intervals; all ordered draws, replicate sizes and coefficients must be
independently auditable. Use a documented deterministic native RNG, not a claim
of historical Stata draw/SE/interval parity. Independent communes are assumed;
this is neither a valid survey design nor a causal or few-cluster guarantee.

## Acceptance, failure and exclusions

1. Actual CLI text and structured JSON perform load/prepare → native original
   Poisson → whole-commune B500 pooled refits → summaries and original-model
   mean/index predictions. The bootstrap report does not replace original
   retained Poisson state. Input remains owned; exports refuse replacement and
   scripts remain nontransactional.
2. Independently parse both original ASCII and precise DTA; use base R Poisson
   QR/GLM or another independently justified likelihood solver on the *exact*
   copied sample for every replicate. A separate Python RNG replays every native
   ordered draw. Compare all500 refits, covariance/intervals, sample sizes and
   all source/prediction rows; investigate discrepancies before setting fixed,
   justified tolerances. Compare printed DTA original point facts only, without
   claiming historical bootstrap uncertainty parity.
3. Synthetic unbalanced groups cover zero-total units, variable N, original and
   resampled singularity/nonexistence, integer/count and key failures, source and
   retained-state preservation, deterministic text/JSON and export collisions.
   No failed replicate is redrawn or discarded. Bounds and actual B500 runtime
   and memory must be justified and measured; no production R/Python dependency.
4. Start with the smallest safe reuse of native Poisson and existing bootstrap
   RNG/reporting; a new public estimator option is justified only if a usable
   bounded end-to-end workflow and resource profile require it. Do not silently
   change existing OLS/quantile bootstrap contracts. Explicitly run new tests,
   affected crate and reverse-consumer regressions, numerical references and
   scoped format/Clippy/diff checks. Update SPEC, roadmap and ledger only to
   independently evidenced behavior. No hosted CI or independent source review
   is implied by local tests.

**Excluded from this slice:** offsets, frequency/survey weights, general
nonlinear bootstrap, simultaneous bands or studentized/BCa intervals, clustered
analytic covariance in the resampled fits, FE/RE component refits, survey design
and claims of whole-table/chapter completion. These exclusions do not waive
required future roadmap work.
