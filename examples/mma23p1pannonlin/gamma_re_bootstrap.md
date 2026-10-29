# Chapter 23 gamma random-effects Poisson whole-firm bootstrap

This bounded SQL composition adds a 500-replicate whole-firm pairs bootstrap
to the independently validated [shared-gamma Poisson fit](gamma_re_poisson.md)
for §23.3/Table 23.1 of Cameron and Trivedi (2005), using the
[author program](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.do)
and [log](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.txt).
It is not a general native `xtpoisson` or bootstrap command. Historical
bootstrap parity remains unresolved.

## Sample, model and resampling

The pinned `patr7079.asc` SHA-256 is
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`.
Use all 346 source-record-order firms and 1,730 records in 1975–79, including
22 zero-total firms and missing nonmodel ARDSSIC. The only regressor is LOGR
plus an intercept, with no other covariates, year effects, weights or offsets.
The fitted likelihood has a single gamma effect shared by each firm's five
counts, marginal mean `exp(a+b*LOGR)` and gamma variance `alpha`; its likelihood,
observed-information covariance, boundary LR and marginal predictions are
described and independently validated in the linked original-fit workflow.

For each of 500 replicates, draw 346 whole firms with replacement. Each copy
receives a **separate shared gamma effect**. Multiplying a source firm's
five-row likelihood contribution by its draw frequency is equivalent; merging
repeated source IDs into one longer shared-effect panel is a different
likelihood. Refit `(a,b,ln(alpha))` jointly starting from the original native
pooled-Poisson coefficients and `ln(alpha)=0`, and recompute the original fit.
The report includes all three original coefficients, marginal mean/residual
summaries, replicate means and biases, the full B−1 covariance and SEs, and
type7 percentile 95% intervals. The sorted export contains every replicate,
position, RNG step, original firm ID, all three coefficients and convergence
metadata. Separate full original inference and prediction rows are available in
[gamma_re_poisson.td](gamma_re_poisson.td).

Independent xoshiro256++/SplitMix64 streams use seed `10001+r−1` and unbiased
u64 rejection in source-ID order. This is neither Stata's RNG nor native
TabDat's single stream. The cluster-pairs bootstrap assumes independent firms;
it does not establish a correct conditional distribution, exogeneity,
causality, or few-cluster reliability. No bootstrap p-values, BCa intervals or
restorable bootstrap model are claimed.

| Original source, distinct copies | Intercept | LOGR slope | ln(alpha) |
| --- | ---: | ---: | ---: |
| Original coefficient | 2.3127049950 | 0.3487831736 | 0.5454693087 |
| Bootstrap mean | 2.3481578645 | 0.3447451779 | 0.5378039097 |
| Bootstrap bias | 0.0354528695 | −0.0040379957 | −0.0076653991 |
| Bootstrap SE | 0.4084147478 | 0.1185381712 | 0.2168489657 |
| Type7 percentile 95% interval | [1.7048732777, 3.2786805003] | [0.0884274907, 0.5515419449] | [0.1432575984, 0.9672668970] |

The full covariance is `[[0.1668026062, −0.0475899159, 0.0834480632],
[−0.0475899159, 0.0140512980, −0.0247188123],
[0.0834480632, −0.0247188123, 0.0470234739]]`.

## Historical ambiguity — not silently resolved

The author program uses `cluster(id)` without visibly specifying `idcluster()`.
Current [Stata guidance](https://www.stata.com/support/faqs/statistics/bootstrap-with-panel-data/)
says repeated panel draws should receive distinct IDs, but the historical
version/program's treatment is not established. On the **same prescribed 500
draws**, an independent R counterfactual merging repeated source IDs into one
longer gamma panel yields slope bias `−0.1526529037` and SE `0.1200321049`,
versus the distinct-copy bias `−0.0040379957` and SE `0.1185381712`. The
author log prints slope bias `−0.1581585` and SE `0.1194127`. Similarity of
the merged-ID result is suggestive, not evidence of historical Stata internals
or its RNG. The distinct-copy and historical results are not claimed equal.
The historical seven-decimal ln(alpha) precision difference in the original
fit is also documented in the linked fit workflow.

## Numerical, state and file boundaries

The script requires unique integer `(id,year)`, IDs 1–346, exactly one row in
each year 1975–79, at least two firms, nonnegative integer counts at most 1,000,
and finite `|LOGR|≤20`. The native pooled start must reconstruct every native
linear prediction within `1e-12*(1+max|pooled_xb|)`. Full observed-information
Newton steps start at `ln(alpha)=0`, with at most 100 steps, `|a|,|b|≤20`,
`|ln(alpha)|<8`, and linear predictors in `[−40,40]`. Each iteration checks
that the copied-firm and row counts equal the full prescribed draw and every
source panel has five rows. Positive-definiteness uses scale-aware Schur
complements, and likelihood must not materially decrease. The final scaled step
is at most `1e-12`; all three scores obey the `1e-10` absolute-term-scale
criterion, and inverse-information and propagated-score-sensitivity guards
match the linked original fit. Bounded integer-count recurrences avoid
cancellation-prone subtraction of gamma derivatives.

Full Newton is not a global optimizer. **Any** boundary, singular,
cancellation-dominated, nonascending, incomplete, out-of-range or nonconverged
original or replicate fit aborts the SQL statement; no draws are omitted,
regularized or redrawn. In the independent 12-firm failure fixture, the
original full-Newton fit converges, but prescribed replicate 13 has minimum
information eigenvalue `−0.1530777766` at its second evaluated iterate. This
is a limitation of the declared full-Newton path, not proof that no alternative
optimizer could fit that replicate. Some SQL guards surface only `sql failed`
and a script location.

The pooled Poisson fit and `pooled_xb` prediction occur first. If the later
SQL fails, those prior changes remain; the failed statement does not replace
active data or retained fit. On success the active data are the draw relation,
which does not contain LOGR, while the retained model is still native pooled
Poisson, **not** gamma RE or bootstrap state. Native prediction therefore
cannot run directly on that draw relation. The input file is unchanged and
export refuses to overwrite. Use a fresh output directory and check exit status:
the report can precede an export failure. An original-source run took about
40 seconds on the development host and produced a large CSV; performance
varies by machine.

## Reproduce and independent evidence

```sh
repo=/absolute/path/to/tabdat-explore-rs
work=/absolute/path/to/empty/work-directory
curl -fsSL https://cameron.econ.ucdavis.edu/mmabook/patr7079.asc -o "$work/patr7079.asc"
shasum -a 256 "$work/patr7079.asc"  # compare with the pinned hash above
(cd "$repo" && cargo build --locked --bin tabdat-explore-rs)
sh "$repo/examples/mma23p1pannonlin/prepare.sh" "$work/patr7079.asc" "$work/patents.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma23p1pannonlin/gamma_re_bootstrap.td" > result.jsonl)
python3 "$repo/examples/mma23p1pannonlin/conditional_bootstrap_draws.py" 10001 346 500 346 "$work/oracle.csv"
Rscript "$repo/examples/mma23p1pannonlin/gamma_re_bootstrap_reference.R" "$work/patr7079.asc" "$work/oracle.csv" "$work/r-reference"
python3 "$repo/examples/mma23p1pannonlin/check_gamma_re_bootstrap.py" "$work" "$work/oracle.csv" "$work/r-reference" "$work/result.jsonl" "$work/patr7079.asc"
# Optional sensitivity probe; this is not a claim about the historical implementation.
Rscript "$repo/examples/mma23p1pannonlin/gamma_re_bootstrap_reference.R" "$work/patr7079.asc" "$work/oracle.csv" "$work/r-merged-ids" merged_source_ids
```

Development-only R 4.6.1 independently maximizes the integrated likelihood
with BFGS and refines each fit with its analytic score and full observed
information. The Python oracle checks all **173,000 ordered draw IDs and RNG
steps**, and the audit checks all 500 three-parameter refits, 31 report fields,
and all 1,730 exact source/design and reconstructed marginal mean/residual
rows. Floating comparisons use `1e-9+1e-10*abs(reference)`; maximum observed
intercept/slope/ln(alpha) refit differences were `3.34e-12`, `4.50e-13` and
`1.48e-12`. The independently checked likelihood decomposition and full
original covariance are in the linked original-fit workflow.

The reproducible 40-firm fixture includes eight zero-total firms and checks all
20,000 draws and 500 R refits, full summary, text/JSON, reversed input and
no-clobber. Recreate it and the separate 12-firm Newton failure probe with:

```sh
python3 "$repo/examples/mma23p1pannonlin/gamma_re_bootstrap_synthetic.py" "$work/synthetic40.asc"
python3 "$repo/examples/mma23p1pannonlin/conditional_bootstrap_draws.py" 10001 40 500 40 "$work/oracle40.csv"
Rscript "$repo/examples/mma23p1pannonlin/gamma_re_bootstrap_reference.R" "$work/synthetic40.asc" "$work/oracle40.csv" "$work/r-reference40"
python3 "$repo/examples/mma23p1pannonlin/conditional_bootstrap_draws.py" 10001 12 500 12 "$work/oracle12.csv"
Rscript "$repo/examples/mma23p1pannonlin/gamma_re_bootstrap_failure_probe.R" "$repo/tests/fixtures/mma23_conditional_fe/source.asc" "$work/oracle12.csv" "$work/failure12.csv"
```

The failure probe identifies the failed draw above; the SQL workflow aborts
without export and preserves the prior native fit and data. Tests also cover
malformed, missing, duplicate,
nonfinite and out-of-range data, identical-panel dispersion boundary, singular
and cancellation-sensitive information, reconstructed-start and predictor
limits, forced iteration/RNG bounds, incomplete iteration membership and state
and file invariants. Python/R and author data are development-only.

The separate [normal-RE Poisson workflow](normal_re_poisson.md) uses a different
random-effect distribution. Normal-RE bootstrap, Figure 23.1, general nonlinear
panels and other required book analyses remain open. This slice does not complete
Chapter 23, the graduate methods core or the full book inventory.
