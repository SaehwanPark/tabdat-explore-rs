# Household FE whole-commune bootstrap: WIP contract

**Status: WIP; not an executable validated workflow yet.** Target: the FE block
of `mma24p1olscluster.do`, §24.7, after the separately delivered pooled-OLS
bootstrap. The author says these B500 results are not reported in Table24.4.
The unrelated RE/Poisson bootstrap blocks, survey design and remaining chapter
illustrations remain required, not waived.

## Exact sample, estimand and limits

Publisher ASCII SHA256 `74cb37e8c14609e382d334e368a54f63bccd4e952d02bcbf535baf19115d4061`;
separate DTA SHA256 `0ea6639f93a5bc3f53bcbd26b6865ad6908a04f41ab04d90e7c9aa4b7e9787bf`.
Ordered source-row, `lnexp12m` outcome, six predictors `lnhhexp age sex hhsize
farm educ`, commune grouping. Original 5,999 rows, 993 NULL outcomes, 5,006
complete cases/194 groups of sizes1–39 including one singleton. `source_row`
is a unique key and native-required ordering field, **not time**.

Each of500 deterministic replications samples194 entire original communes
with replacement; each selected block is a distinct copy, and all observations
in it remain together. Refit the six within slopes and re-normalize the common
level intercept using the observation-weighted copied-sample means:
`a = mean_draw(y) - mean_draw(X)' beta`. Thus unbalanced group sizes and the
singleton affect the intercept and resampled N even when they contribute no
within slope information. Report original a/beta, bootstrap means, bias, B−1
full7x7 covariance/SE, type7 percentile95; no finite-cluster, t/z, BCa or
studentized inference. Resampled copies preserve original within-group means;
normalization is **observation weighted**, not equal-group weighted. Model is
unweighted and assumes independent communes; not survey or causal inference.

Use existing native FE model and current-data predictions for the original fit,
without replacing its retained state. A bounded SQL composition may refit from
within group moments (with rank/finite/conditioning checks) rather than adding
an estimator API. A per-replication seed `10001+r-1`, xoshiro256++/SplitMix64
and fixed-u64 rejection matches the existing Chapter21 FE-composition stream,
not the generic OLS bootstrap's continuing stream or historical Stata RNG.
Any original/replicate rank, numerical, nonfinite or RNG failure aborts without
redraw or partial bootstrap summaries. Strict finite source/key guards precede
fitting. Scripts are ordered, nontransactional and export only no-clobber files.

## Acceptance and evidence plan

Risk: high (unbalanced FE normalization, numerical refit, sample/file state).

1. Actual terminal and CLI-v1 JSON script performs guarded load → ordered
   complete cases → native original FE fit → B500 distinct-copy refits → original
   FE fitted/effect/residual predictions → labeled summary and ordered exports.
   Text/JSON exports match; original model and source are retained; existing
   output owners survive failures.
2. Base R independently expands the raw copied observations, assigns new copy
   identities, centers within copies and solves six slopes by `lm.fit` QR;
   recomputes observation-weighted common intercept in every draw. Compare all
   500 seven-parameter refits, full covariance/bias/SE/type7 intervals, all
   resampled sizes, all source/prediction rows, and every ordered RNG draw with
   a separate Python oracle. Fixed abs1e-9; investigate differences rather than
   widen. Reproduce on publisher ASCII and separate original DTA; compare any
   available printed original FE coefficients at their printed precision only.
3. An inspectable unbalanced/singleton synthetic panel tests changed per-draw
   normalization, variable N and full inference; include within-identification
   failures in original and resampled data, invalid keys/missingness, deterministic
   draws, state preservation, source/export collisions and negative exit status.
   Auditors must reject representative forged draws, coefficients, summary,
   prediction and source fields.
4. Bound runtime/memory against actual B500 before finalizing an implementation;
   explicitly test new/modified tests first, affected/integration regressions,
   scoped Clippy/format/diff and final review. Update SPEC/roadmap/ledger only
   to what the evidence supports; disclose hosted CI and code-review status.
