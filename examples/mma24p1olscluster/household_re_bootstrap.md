# Household RE whole-commune bootstrap

The [actual workflow](household_re_bootstrap.td) covers the third household
bootstrap block in `mma24p1olscluster.do`, §24.7. The author notes that these
B500 results are not reported in Table24.4. The separate household
[pooled-OLS](household_ols_bootstrap.md) and [FE](household_fe_bootstrap.md)
bootstraps are validated. Other household RE outputs, the individual Poisson
bootstrap blocks, survey design and other illustrations remain required.

## Sample, estimator and inference

Publisher ASCII SHA256 `74cb37e8c14609e382d334e368a54f63bccd4e952d02bcbf535baf19115d4061`;
DTA SHA256 `0ea6639f93a5bc3f53bcbd26b6865ad6908a04f41ab04d90e7c9aa4b7e9787bf`.
The source-row-ordered model has outcome `lnexp12m`, predictors `lnhhexp age
sex hhsize farm educ`, and commune grouping. Of5,999 source rows, 993 have a
NULL outcome; 5,006 complete cases in194 communes remain, sizes1–39 including a
singleton. `source_row` is a unique ordering key, **not time**.

The original fit uses the separately validated opt-in
`vcmethod(equal_group_between)`, not the default trace-corrected Swamy–Arora:
fit within slopes and the equal-copy-weighted Between regression of group means;
`e=RSS_within/(N-G-6)`, `raw_u=RSS_between/(G-7)-e*mean(1/n_g)`, and
`u=max(0,raw_u)`. Each draw **refits both components**, including zero
truncation, before quasi-GLS with `theta_g=1-sqrt(e/(e+n_g*u))`. Repeated
selections are distinct copied units in both regressions and all denominators.
The named formula reproduces this sample's printed RE coefficient/SE facts; it
is not a claim of general Stata compatibility or fixed-theta resampling.

Each of500 replications samples G sorted-key source communes with replacement.
Per-rep seed `10001+r-1` expands through xoshiro256++/SplitMix64 and fixed-u64
rejection, at most G+1024 words per replication. This is the FE composition
stream, not generic OLS's continuing stream or the historical Stata RNG. The
summary reports original intercept/slopes, bootstrap means and bias, full B−1
covariance/SE and R-type7 percentile95 intervals. There are no t/z tests, BCa,
studentized intervals or finite-cluster corrections; B=500 has Monte Carlo
error. Independent communes and the RE mean/exogeneity assumption remain
necessary. This is unweighted pairs inference, not survey-design or causal
inference.

## Bounds, numerical safety and state

The script requires1..5,999 source rows, unique integer source keys1..5,999,
integer commune keys1..194, 8..194 retained communes, sizes at most39, positive
within and Between residual degrees of freedom, and finite model values of
magnitude at most1e6. Only NULL outcomes are dropped. Any failed original or
replicated fit aborts without redraw or partial summary; zero-truncated **unit**
variance is valid, but within residual variance must be positive. Predictions
use the retained original native RE fit, not a resampled model.

SQL accumulates per-commune within moments, computes equal-copy Between moments,
then solves scaled normal equations for within, Between and quasi-GLS slopes.
Every standardized Schur pivot must be finite and greater than0.001; this
conservative gate is not a general condition-number certificate and may reject
an identified fit. The within residual quadratic must exceed `1e-4` times the
within outcome sum of squares to limit cancellation. The transformed-intercept
weight ratio `e/(e+n_g*u)` must exceed `1e-24`. All components, refits and
summaries must be finite; original composed fitted values must agree with the
native original fit within1e-8. These checks are explicit failure boundaries,
not clipping or numerical accuracy guarantees for arbitrary admitted data.

A deliberately identified but more collinear control was rejected: with a
looser1e-10 pivot gate its 500-refit covariance differed from independent R by
up to3.92e-7. A different near-perfect-fit control was rejected by the
cancellation guard: with a much looser guard, its tiny within variance differed
from R by up to1.27e-5 **relative**, despite small absolute differences. The
fixed1e-9 absolute reference tolerance was not widened. An accepted correlated
control matches all500 R refits and full covariance within1.25e-12. These are
sample-specific stress observations, not universal error bounds.

On the local Apple M3 Max, the original B500 run took22.48s and about1.28 GB
maximum RSS for ASCII; the precise DTA run took22.58s and about1.23 GB. This is
resource-intensive for a small dataset; machines with less available memory may
fail, and timings are not platform guarantees. An intermediate-materialization
change reduced peak memory while leaving all four original ASCII and DTA exports
byte-identical. The bounded composition is not a general RE-bootstrap API.

Four ordered no-clobber exports contain the49-row full covariance/summary,
500 refit coefficients, component diagnostics and sample sizes, all ordered
copy/step/source-commune draws, and original source/RE xb/residual predictions.
Scripts are not whole-script transactions: a later collision preserves its owner
and earlier successful exports, stops later commands and does not undo prior
session effects. The original source and native fit remain owned.

## Independent evidence

Base R4.6.2 independently expands every raw copied commune, assigns distinct
copy IDs, performs within and equal-group Between QR fits, recalculates both
variance components and fits actual quasi-transformed rows by QR. A separate
Python RNG auditor checks **all97,000 ordered native draws**. For publisher ASCII
and directly read DTA with lossless `--precise` conversion, the auditor compares
all500 seven-parameter refits and component diagnostics, the full49 covariance
cells and summary/intervals, and all5,999 source/5,006 prediction rows at fixed
absolute1e-9. Maximum absolute discrepancies (fit diagnostic / summary /
prediction) are6.55e-11 / 4.04e-14 / 2.10e-13 for ASCII and7.64e-11 /
8.11e-14 / 2.34e-13 for DTA. Four original text/JSON exports are byte-identical.
On DTA, both R and native original coefficients match the seven printed author
points within half the final digit; no historical bootstrap SE/interval or RNG
parity is claimed.

Synthetic controls cover balanced single-theta and unbalanced/singleton designs,
500 zero-truncated refits, and a mixed sample with28 zero/472 positive
replicates. Their refits and summaries are checked against R; all24,000 draws
for each were independently audited. Tests cover source-row reordering,
determinism, original and resampled rank/variance failures, the numerical gates,
state preservation and every export-collision stage. Twenty-four full-auditor
forgeries and one printed-author-point forgery were rejected. No independent
source-code review or hosted CI is claimed by these local checks.

The optional DTA converter's default preserves older15-significant-digit CSV
output; older household DTA audits therefore did not establish binary64 identity
with DTA inputs. `--precise` emits17 significant digits and round-trips all
source values used here. No native DTA reader is claimed.

## Reproduce

In a fresh directory, with `repo` pointing at this checkout and source paths
adjusted:

```sh
python3 "$repo/examples/mma24p1olscluster/prepare.py" vietnam_ex1.asc vietnam.csv
"$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma24p1olscluster/household_re_bootstrap.td" > result.jsonl
Rscript "$repo/examples/mma24p1olscluster/household_re_bootstrap_reference.R" vietnam_ex1.asc household_re_bootstrap_draws.csv reference
python3 "$repo/examples/mma24p1olscluster/check_household_re_bootstrap.py" . reference vietnam_ex1.asc
```

Use `--text` in a second fresh directory for readable tables and the same CSVs.
For DTA, prepare with
`Rscript "$repo/examples/mma24p1olscluster/dta_reference.R" vietnam_ex1.dta vietnam.csv --precise`,
run the native script, pass `--dta` to the R reference and Python auditor, and
run `check_household_re_bootstrap_author.py vietnam_ex1.dta mma24p1olscluster.txt reference .`.
Reference and output paths must not already exist. R/Python are optional
preparation/validation tools, not production dependencies.
