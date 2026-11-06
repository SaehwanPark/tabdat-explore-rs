# Household pooled OLS commune bootstrap

This validated bounded workflow covers the first of three additional cluster
bootstrap blocks in the author `mma24p1olscluster.do`; the author notes these
results are not reported in the book's Table24.4. It does not complete FE/RE
bootstrap or survey design.

## Exact outcome and sample

Use original `vietnam_ex1.asc` with the established preparation and verify the
optional separately converted DTA: 5,999 source rows, 5,006 joint-finite outcomes
and regressors, 194 communes of sizes1–39, six numeric slopes `lnhhexp age sex
hhsize farm educ` and intercept. Sort by stable `source_row`; it is not time.
OLS on complete households is the original point fit. Draw194 **whole communes**
with replacement in first-seen order per replication, preserving within-commune
row order and repeated copies, B=500, seed10001. Unequal group sizes imply
variable replicate N. The native xoshiro256++/SplitMix64/rejection stream is
explicit and **not the historical Stata stream**.

Report point estimates, replicate means/bias, full B−1 sample covariance and
standard errors, R type-7 marginal percentile95 intervals; distinguish these from
Stata's normal, bias-corrected and historical-stream bootstrap intervals. Do not
claim 500-rep Monte Carlo precision or few-commune robustness. The bootstrap is a
read-only result, not a replacement of the owned original OLS state/covariance;
default/xb and residual predictions come from that retained OLS model.

## Resource decision

Existing generic cluster bootstrap conservatively limits
`G * largest_selected_group_size * B * K^2 <= 100,000,000`, K including intercept.
The actual original design requires `194*39*500*7^2 = 185,367,000`, so the exact
author-program replication count is rejected before fitting. A real250-rep
probe (`92,683,500` worst-case work) completed in1.57s/72.4MB maximum RSS on the
current development machine; that is feasibility evidence, not a platform claim.

The **cluster bootstrap** conservative maximum is now200,000,000, retaining
checked arithmetic, max G×largest group allocation bound, B2..5000, deterministic
stream, immediate fail-on-any-replicate semantics and unchanged100,000,000 IID
work cap. The original B500 script completed in2.81s on the same machine; a
separate B500 fit probe used73.3MB maximum RSS. These observations are not a
platform guarantee. Tests cover both sides of the new cluster boundary, the
unchanged IID boundary and failure-state safety. This is a bounded numerical
work-limit change, not permission for unlimited replications.

## Evidence

The [actual script](household_ols_bootstrap.td) performs load/key checks → ordered
complete cases → original OLS → B500 commune bootstrap → original-model fitted
values/residuals → report/export. The original 5,006 selected rows have194
communes, sizes1–39. Resampled N ranges from4,765 to5,188 in this seed/stream.

R4.6.2 `lm.fit`, B−1 `cov` and type7 quantiles independently refit all500 draws
from the separate Python xoshiro oracle. Both original ASCII and directly read
DTA validate94 point/mean/bias/SE/interval/full-covariance quantities, all5,999
source rows, all5,006 prediction rows and all500 native replicate sizes at fixed
absolute1e-9. Maximum discrepancies are1.85e-13 ASCII and1.28e-13 DTA. The oracle
regenerates97,000 reference group indices; a Rust unit test directly compares the
first388 native indices. Aggregate numerical and sample-size checks do not expose
or claim a per-index native audit for the remaining draws. The DTA author check
matches seven printed **original OLS points** within half the final digit, not
historical bootstrap SEs/intervals. Original text/JSON exports are byte-identical.
The strict auditor rejected17 forged source, result, draw or prediction fields;
the printed-author checker rejected one forged point.

An uneven 48-group fixture includes singletons and two missing cases; focused
actual-script tests check all94 quantities, both covariance/intervals, all500
replicate sizes, deterministic reruns, prediction rows, retained original
model, original and resampled rank errors, invalid keys, source integrity and
successful/colliding output. Failed scripts leave earlier effects in place.
No independent source-code review or hosted CI is claimed by this local evidence.

## Reproduction

From a fresh working directory (`repo` is the repository path):

```sh
python3 "$repo/examples/mma24p1olscluster/prepare.py" vietnam_ex1.asc vietnam.csv
"$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma24p1olscluster/household_ols_bootstrap.td" > result.jsonl
Rscript "$repo/examples/mma24p1olscluster/household_ols_bootstrap_reference.R" vietnam_ex1.asc reference
python3 "$repo/examples/mma24p1olscluster/check_household_ols_bootstrap.py" . reference vietnam_ex1.asc
```

`--text` provides the same exported CSV and readable report. For a separate DTA
precision check, prepare the native CSV with `dta_reference.R`, pass `--dta` to
the R reference and Python auditor, and run
`check_household_ols_bootstrap_author.py vietnam_ex1.dta mma24p1olscluster.txt reference result.jsonl`.
R and Python are optional reference/preparation tools, not runtime dependencies.
The reference directory and output files must not already exist.

No automatic FE/RE/Poisson refit, variance-component resampling, survey weights,
strata, multiway clustering, bootstrap t/normal/BC interval, new RNG interface,
or native DTA importer is included. All remain required where the roadmap calls
for them; no scope exclusion is implied.
