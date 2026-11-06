# Household RE whole-commune bootstrap: WIP contract

**Status: WIP, not yet an executable validated workflow.** This targets the
third household bootstrap block in `mma24p1olscluster.do`, §24.7; the author
notes that these B500 bootstrap results are not reported in Table24.4. The
separate household pooled/FE bootstraps are validated. The individual Poisson
bootstrap blocks, survey design and other illustrations remain required.

## Exact sample and estimator

Publisher ASCII SHA256 `74cb37e8c14609e382d334e368a54f63bccd4e952d02bcbf535baf19115d4061`;
DTA SHA256 `0ea6639f93a5bc3f53bcbd26b6865ad6908a04f41ab04d90e7c9aa4b7e9787bf`.
Ordered source-row, outcome `lnexp12m`, predictors `lnhhexp age sex hhsize farm
educ`, commune grouping: 5,999 source rows, 993 NULL outcomes, 5,006 complete
cases/194 communes, sizes1–39 including a singleton. `source_row` is a unique
ordering key, **not time**. Preserve source fields and native original RE state.

The fitted method is the separately validated opt-in
`vcmethod(equal_group_between)`, not the default trace-corrected Swamy–Arora:
fit within slopes and equal-copy-weighted Between regression of group means;
`e=RSS_within/(N-G-6)`, `raw_u=RSS_between/(G-7)-e*mean(1/n_g)`,
`u=max(0,raw_u)`. Re-estimate both components on **each** draw, including zero
truncation, then fit quasi-GLS using `theta_g=1-sqrt(e/(e+n_g*u))`. This explicit
formula is not a claim of general Stata compatibility.

Each of500 replications samples G sorted-key source communes with replacement,
with each selected block a distinct copy. Per-rep seed `10001+r-1`,
xoshiro256++/SplitMix64 and fixed-u64 rejection match the previous FE
composition stream, not generic OLS's continuing stream or the Stata RNG.
Report original intercept/slopes, means, bias, full B−1 covariance/SE and
R-type7 percentile95 intervals, with auditable per-refit component/size fields
and all ordered copies. No t/z, BCa, studentized or finite-cluster correction;
B=500 has Monte Carlo error. Independent communes and the RE mean/exogeneity
assumption remain necessary; not survey-design or causal inference.

Use a bounded native-SQL composition if feasible, without a generic new API or
production R/Python. Admission follows the FE bootstrap's original-domain keys,
finite <=1e6 fields, at most5,999 rows, <=194 groups and group size<=39;
require at least8 groups and positive within and Between DF. Require finite,
identified within/Between/GLS fits and positive within residual variance, with
conservative explicit numerical-conditioning gates and original native-model
agreement. Any invalid replicate aborts without redraw or partial summary;
zero-truncated unit variance is a valid outcome. Scripts are nontransactional
and export only no-clobber files. Benchmark B500 time and memory before deciding
whether this implementation is practical.

## Acceptance and evidence plan

Risk: high (unbalanced component re-estimation, numerical stability and state).

1. Actual terminal and CLI-v1 JSON script performs validated load → ordered
   complete cases → native original equal-group RE fit → B500 distinct-copy
   **component-refitting** RE-GLS → original-model population xb/residual →
   full summary and ordered exports. Source and retained fit survive both success
   and failures, and each export collision preserves its existing owner.
2. Independent base R expands raw source blocks with fresh copied IDs, uses QR
   within/Between/actual quasi-transformed regressions and explicitly recomputes
   both variance components per draw. On publisher ASCII and original DTA with
   lossless `--precise` conversion, compare all500 seven-coefficient refits,
   N/components/truncation, full covariance/bias/SE/intervals, all source and
   prediction rows, and all97,000 native draws against a separate Python RNG.
   Fixed absolute1e-9; investigate discrepancies instead of widening tolerance.
   Check printed DTA original points only, not historical RNG/SE parity.
3. Synthetic unbalanced/singleton controls cover positive and zero-truncated
   unit variance, per-draw re-estimation, original and resampled rank/variance
   failures and conditioning; check deterministic outputs, state/source/no-clobber
   behavior and auditor-forgery rejection. Compare text/JSON exports bytewise.
4. Explicitly run new focused tests, affected-crate/reverse-consumer integration
   regressions, scoped Clippy/format/diff and final review; disclose actual
   resource observations, hosted CI and independent-review status. Update SPEC,
   roadmap and coverage only to the independently demonstrated slice.
