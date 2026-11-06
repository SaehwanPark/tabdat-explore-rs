# Household FE whole-commune bootstrap

The [actual workflow](household_fe_bootstrap.td) covers the FE block of
`mma24p1olscluster.do`, §24.7, after the separate [pooled-OLS bootstrap](household_ols_bootstrap.md).
The author says these B500 results are not reported in Table24.4. The RE and
individual Poisson bootstrap blocks, survey design and remaining illustrations
remain required.

## Sample and estimator

Publisher ASCII SHA256 `74cb37e8c14609e382d334e368a54f63bccd4e952d02bcbf535baf19115d4061`;
DTA SHA256 `0ea6639f93a5bc3f53bcbd26b6865ad6908a04f41ab04d90e7c9aa4b7e9787bf`.
The source-row-ordered model has outcome `lnexp12m`, six predictors `lnhhexp age
sex hhsize farm educ` and commune grouping. Of5,999 source rows, 993 have NULL
outcome; 5,006 complete cases in194 communes remain, sizes1–39 including one
singleton. `source_row` is a unique native-required ordering key, **not time**.

Each replication draws194 whole source communes with replacement, treating
repeated selections as **distinct copies**. Their selected observations remain
together. The six within slopes are refit and the common level intercept is
re-normalized on each draw: `a = mean_draw(y) - mean_draw(X)' beta`. Therefore
unbalanced group sizes and singletons affect the intercept and resampled N,
even when a singleton supplies no within-slope information. Source groups are
indexed by **sorted numeric commune key**. Seed `10001+r-1` expands through
xoshiro256++/SplitMix64 and fixed-u64 rejection, with at most G+1024 words per
replication. This is the existing Chapter21 composition stream, **not** the
generic OLS bootstrap's continuing stream or the historical Stata RNG.

The summary reports original a/slopes, bootstrap means and bias, full B−1
covariance and SE, and type7 percentile95 intervals. It does not report t/z,
finite-cluster corrections, BCa or studentized intervals. B=500 has Monte Carlo
error. Independent communes, not arbitrary cross-commune dependence, underpin
pairs resampling. This is unweighted FE inference, not survey or causal inference.

## Bounds, numerical safety and state

The script requires1..5,999 source rows, unique integer source keys1..5,999,
integer commune keys1..194, 7..194 retained communes, sizes at most39, positive
original residual DF and finite model values of magnitude at most1e6. Only NULL
outcomes are dropped. Source fields and the original native FE fit remain owned.
Each replicate must have positive residual DF; no failed draw is discarded or
retried. Prediction uses the native original FE state, not a resampled model.

For bounded work, SQL accumulates per-commune within cross-products and raw
sums in draw order, scales the six-by-six normal equations to unit diagonal,
and solves by elimination. Every scaled Schur pivot must be finite and **greater
than0.001**; invalid, unidentified or more poorly conditioned draws abort. This
conservative pivot gate is not a general condition-number certificate and may
reject an identified fit. Original composed fitted values must also agree with
the separately fitted native QR model within1e-8; all refit/summary fields must
be finite. A deliberately more collinear but identified control was rejected:
with a looser1e-10 pivot gate its covariance differed from independent R by
2.20e-6. The guard was tightened rather than relaxing the1e-9 comparison. A
separate accepted correlated control has minimum observed pivot0.00595 and
matches all500 R refits/full covariance within3.36e-13. The original-data minimum
observed pivot was0.629. These examples are evidence for these samples, not a
universal floating-point error bound.

The original-data script ran in3.94s and used726MB maximum RSS on the local
machine (the synthetic case used200MB); performance and memory are not platform
guarantees. The bounds and pivot failure are intentional, not silent fallback.
This is a specific composition, not a general multivariate FE-bootstrap API.

Four ordered no-clobber exports contain the49-row full covariance/summary,
500 refit coefficients/sample sizes, all ordered copy/step/source-commune draws,
and original source/FE xb/xbu/u/residual predictions. Scripts are not a
whole-script transaction: a later collision preserves its owner and earlier
successful exports, stops later commands and does not undo prior session effects.

## Independent evidence

Base R4.6.2 independently expands every raw copied commune, assigns distinct
copy IDs, demeans within each copy, fits six slopes by `lm.fit` QR and recomputes
the observation-weighted level intercept. The Python auditor independently
replays **all97,000 ordered native draws**. Original ASCII and directly read DTA
validate every500 seven-parameter refit and resampled N, full49 covariance cells,
all coefficient summaries/intervals, all5,999 source and5,006 prediction rows
at fixed absolute1e-9. Maximum absolute discrepancies (refit / summary /
prediction) are6.48e-14 / 3.25e-14 / 1.95e-14 for ASCII and3.91e-14 /
1.24e-14 / 2.04e-14 for DTA. Four original text/JSON exports are byte-identical.
On DTA, both R and native original FE coefficients match all seven printed author
points within half the final digit; no historical bootstrap SE or interval parity
is claimed.

The 48-commune synthetic fixture includes singletons, missing outcome, unequal
sizes and changing per-draw intercept normalization. Its24,000 draws, 500 QR
refits, summaries and all169 predictions are audited. A second correlated
fixture supplies the accepted numerical stress control. Actual-script tests cover
both controls, sorted draw order, variable N, original and singular-resample
rank failure, the rejected collinear case, input/key/size limits, retained model,
source preservation and all four export-collision stages. Fifteen independent
full-auditor forgeries and one printed-author-point forgery were rejected.
No independent source-code review or hosted CI is claimed by these local checks.

The optional DTA converter's **default** deliberately preserves earlier
15-significant-digit CSV output for reproducibility; older household DTA audits
compare against that rounded CSV representation, not the DTA binary64 values
bit-for-bit. The new `--precise` option emits17 significant digits. For this
workflow it reproduces every directly read DTA input value in the native CSV;
9,647 cells differ at binary64 precision from the old15-digit conversion. The
ASCII path is unaffected. No native DTA reader is claimed.

## Reproduce

In a fresh directory with `repo` pointing at the checkout and source paths
adjusted:

```sh
python3 "$repo/examples/mma24p1olscluster/prepare.py" vietnam_ex1.asc vietnam.csv
"$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma24p1olscluster/household_fe_bootstrap.td" > result.jsonl
Rscript "$repo/examples/mma24p1olscluster/household_fe_bootstrap_reference.R" vietnam_ex1.asc household_fe_bootstrap_draws.csv reference
python3 "$repo/examples/mma24p1olscluster/check_household_fe_bootstrap.py" . reference vietnam_ex1.asc
```

Use `--text` in another fresh directory for the same CSVs and readable tables.
For the separate DTA audit, prepare with
`Rscript "$repo/examples/mma24p1olscluster/dta_reference.R" vietnam_ex1.dta vietnam.csv --precise`,
run the native script, pass `--dta` to the R reference and Python auditor, and
run `check_household_fe_bootstrap_author.py vietnam_ex1.dta mma24p1olscluster.txt reference .`.
Reference and output paths must not already exist. R/Python are optional
preparation/validation tools, not production dependencies.
