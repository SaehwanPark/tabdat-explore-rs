# Patents native conditional-Poisson FE bootstrap (WIP contract)

## Bounded outcome and rationale

The existing native conditional-FE replay fits the original patents data, but
native bootstrap rejects its324 positive-total firms at the current256-group
cap. The independent SQL bootstrap remains usable and separate. This slice
will remove that concrete native barrier, not add a new estimator family or
substitute a named-report-only change for required resampling work.

Scope: raise **only conditional-Poisson FE** bootstrap's retained positive-group
limit to512. Preserve B2..500,1..16 slopes, G>K, G*largest_group_size<=40000 and
G*largest_group_size*B*K²<=2500000000, fit/convergence checks, exact owned sample,
first-seen group order, one continuous xoshiro256++/SplitMix64/u64-rejection
stream, distinct repeated-group copies, fail-abort/no redraw and read-only model
state. Pooled-Poisson and gamma-RE bootstrap keep their256-group bounds. No
statistical formula, RNG, parser/API/schema or dependency change is planned.

## Exact source/sample/inference

Cameron–Trivedi(2005) §23.3 patents companion program, `patr7079.asc`,
SHA256 `87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`.
Use the [native conditional-FE source guards](native_conditional_fe.md):1730
1975–79 firm-years/346 firms, PAT counts/sole LOGR slope, no common intercept,
weights, offsets or year effects. Original22 all-zero firms/110 rows are
excluded from conditional FE, leaving1620 rows/324 positive-total firms.
Resample those324 firms with replacement500 times, seed10001, copying all five
years per selected occurrence. Report point, bootstrap mean/bias/SE, full scalar
B−1 covariance and type7 percentile95 limits; no normal/t p-value is invented.
Retain the original actual FE model/prediction normalization, not a bootstrap
replicate or a current-subset re-normalization. Zero-total original groups keep
NULL means/residuals; complete-X slope indices stay defined.

## Acceptance and important failures (declared before implementation)

1. Kernel accepts stable257/324/512-group controls, rejects513 retained positive
   groups with a clear method-specific error, and still enforces copied-row/work
   budgets. Existing below256 RNG/results/exclusions remain unchanged. Repeated
   groups remain distinct; no per-group fixed-width-key collision.
2. Actual source-native B500 workflow completes load/guard/cohort → fit →
   bootstrap → retained predictions/diagnostics → readable/CLI-v1 report/export.
   Inspect original state, all1730 source/prediction records/110 exclusions,
   JSON/text equality, labels/registry, source permutation and no-clobber/failure.
3. Reuse unchanged development-only `conditional_bootstrap_reference.R` on the
   exported native draw sequence: independent base-R conditional score roots for
   all500 replicates, three weighted dummy-Poisson GLM cross-checks, scalar B−1
   variance/SE/mean/bias/type7 limits. Verify all162000 original draws separately
   with the independent Python RNG oracle, not just native-exported indices.
   Freeze a small existing synthetic source's500 R refits and summary fields;
   keep original publisher data external. Fixed numerical gates: absolute1e-9
   for coefficients/bootstrap summaries, exact membership/indices/counts;
   retain the existing native FE fit/reference gates unchanged. Investigate
   source/sample/normalization/stream differences, never widen tolerances.
4. Independent raw-source score equations and recomputed empirical covariance/
   quantiles reject jointly forged native/R bootstrap fields. Test wrong source,
   draw labels/order/counts, selected groups, coefficient/refit and retained
   prediction ownership; singular refits abort instead of omitting/redrawing.
5. Explicitly run new/modified focused kernel/runtime/actual-script/Python tests,
   affected stats-crate and relevant reverse-consumer/bootstrap/help checks;
   account for retained baselines honestly, then reconcile SPEC/roadmap/ledger/
   source inventory/help and prior256-limit notices. Author review, complete PR
   evidence, merge/tree identity and branch pruning precede another slice.

Excluded from this slice, still required where applicable: native gamma/pooled
cap extensions, Gaussian FE/RE automatic resampling, native normal-Poisson RE,
GEE bootstrap, general/unbalanced/factor/robust conditional-FE inference, source
named tables, historical Stata draw/duplicate-ID/bootstrap-SE/interval parity
and the complete book inventory. The SQL bootstrap remains distinct (different
native stream construction); no claim of all source/historical output parity.

Status: preimplementation contract. No completed B500 native original audit,
512-bound behavior or new passing test is claimed until evidence is recorded.
