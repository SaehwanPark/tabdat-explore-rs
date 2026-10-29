# Chapter 23 exchangeable Poisson GEE whole-firm bootstrap

This bounded SQL composition adds a 500-replicate whole-firm pairs bootstrap
to the independently validated [exchangeable GEE fit](exchangeable_gee.md)
for the §23.3/Table 23.1 patents illustration in the
[author program](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.do)
and [log](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.txt).
It is not a general `xtgee` or bootstrap command. Historical bootstrap parity
remains unresolved, as explained below.

## Sample, model and resampling

Pinned `patr7079.asc` SHA-256:
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`.
Use all 346 source-order firms/1,730 complete 1975–79 records including 22
zero-patent firms and missing nonmodel ARDSSIC metadata. Fit
`mu_it=exp(a+b*LOGR_it)` for PAT with no extra regressors, time effects,
weights or offsets, using the legacy exchangeable Pearson moment and fixed
Poisson scale 1 specified in the linked GEE workflow.

For each of 500 replicates, draw 346 whole firms with replacement and treat
each copied five-row firm as a **distinct cluster** in all GEE moment and score
calculations. Frequency weights are equivalent here only because each copy
contributes its original five-row score and moments separately. Each fit starts
at the original native pooled-Poisson coefficients, recomputes rho at every
full Fisher-scoring step, and rejects invalid correlation, bread, convergence,
score or numerical bounds. The original GEE fit is also recomputed. The output
reports its coefficients, rho and mean/residual summaries, 500 replicate means
and biases, full B−1 covariance and SEs, and type7 percentile 95% intervals.
The sorted export records every replicate, position, RNG step, source firm,
coefficients, rho and convergence metadata. Separate GEE prediction rows and
working/firm-sandwich inference are available in the original fit workflow.

The per-replicate xoshiro256++/SplitMix64 streams use seed `10001+r−1` and
unbiased u64 rejection in source-ID order. This is neither Stata's RNG nor
native TabDat's single stream. The bootstrap assumes independent firms; it does
not establish causality, mean adequacy or few-cluster reliability. No p-values,
BCa intervals or restorable bootstrap model are claimed.

| Original source, distinct copies | Intercept | LOGR slope |
| --- | ---: | ---: |
| Original GEE coefficient | 2.0675152480 | 0.5595300704 |
| Bootstrap mean | 2.0463165396 | 0.5651613749 |
| Bootstrap SE | 0.1390624901 | 0.0418896050 |
| Type7 percentile 95% interval | [1.7597982558, 2.3044170544] | [0.4870895209, 0.6533765603] |

The original rho is `0.8983666759`; the bootstrap covariance is
`[[0.0193383761, -0.0051976328], [-0.0051976328, 0.0017547390]]`.

## Historical ambiguity — not silently resolved

The author program bootstraps `xtpoisson ..., pa i(id)` with `cluster(id)` but
does not visibly specify `idcluster()`. Current [Stata guidance](https://www.stata.com/support/faqs/statistics/bootstrap-with-panel-data/)
says repeated panel draws should be assigned distinct IDs. The precise handling
of repeated IDs by that historical Stata version/program is not established;
treating duplicate copies as a single longer exchangeable panel can change the
GEE equations. This workflow explicitly uses distinct five-row copies as the
well-defined cluster-pairs bootstrap. Different RNG streams and unresolved
historical ID behavior preclude exact historical bootstrap parity.

An independent R `gee` 4.13-30 counterfactual on the **same** first prescribed
draw gives slope `0.5712201728` with distinct five-row copies versus
`0.5422699889` when copies sharing source ID are merged (maximum cluster size
35); see `exchangeable_gee_duplicate_probe.R`. This demonstrates material
sensitivity, not that historical Stata used either exact convention. The
author's printed bootstrap slope SE `.10729042` also differs from the 500-draw
distinct-copy reference `.04188961`. RNG and/or historical panel identity and
estimator behavior remain unresolved. The historical output and the remaining
Chapter 23 illustration requirements are not counted as completely reproduced.

## Numerical, state and file boundaries

The script requires unique integer `(id,year)`, IDs 1–346, one row per year
1975–79, at least two firms, integer counts 0–1,000, and finite `|LOGR|≤20`.
It bounds `|a|,|b|≤20`, linear predictors to `[−40,40]`, and scoring to 100
full steps. Pooled starting coefficients must reconstruct every native start
prediction within `1e-12*(1+max|pooled_xb|)`. Each final scaled step must be at
most `1e-12` and recomputed score at most `1e-10` on its absolute-term scale,
including absolute row-level residual contributions before summation. Pearson variation, both exchangeable-correlation boundary gaps and the bread
Schur complement use the scale-aware guards documented for the original GEE
fit. In addition, `128*epsilon*abs(B^-1)*score_scale` must be below
`1e-9+1e-10*abs(coefficient)` componentwise, rejecting cancellation-sensitive
coefficients. These numerical limits are specific to this composition, not a
general GEE feasibility guarantee.

No rho clamping, regularization, failed-draw omission or redraw is performed:
**any failed replicate aborts the SQL statement and prevents export**. Some
SQL errors currently report only `sql failed` and the script location. The
native pooled Poisson fit and `pooled_xb` prediction precede the bootstrap; if
the later SQL fails, those earlier session changes remain. A failed bootstrap
SQL statement does not replace the active data or prior fit. On success, the
active data become the draw relation and the retained native model is still
pooled Poisson, not GEE or bootstrap state. Input files remain unchanged and
export refuses to overwrite. Use a fresh directory and check process exit
status: a report may precede an export failure. Original-source runs took
roughly 1.5–2 minutes and wrote a 39 MB CSV on the development machine;
performance varies with hardware.

## Reproduce and independent evidence

```sh
repo=/absolute/path/to/tabdat-explore-rs
work=/absolute/path/to/empty/work-directory
curl -fsSL https://cameron.econ.ucdavis.edu/mmabook/patr7079.asc -o "$work/patr7079.asc"
shasum -a 256 "$work/patr7079.asc"  # compare with the pinned hash above
(cd "$repo" && cargo build --locked --bin tabdat-explore-rs)
sh "$repo/examples/mma23p1pannonlin/prepare.sh" "$work/patr7079.asc" "$work/patents.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma23p1pannonlin/exchangeable_gee_bootstrap.td" > result.jsonl)
python3 "$repo/examples/mma23p1pannonlin/conditional_bootstrap_draws.py" 10001 346 500 346 "$work/oracle.csv"
# Development-only R gee 4.13-30 must be installed separately.
Rscript "$repo/examples/mma23p1pannonlin/exchangeable_gee_bootstrap_reference.R" "$work/patr7079.asc" "$work/oracle.csv" "$work/r-reference" > "$work/r-gee.log"
python3 "$repo/examples/mma23p1pannonlin/check_exchangeable_gee_bootstrap.py" "$work" "$work/oracle.csv" "$work/r-reference" "$work/result.jsonl" "$work/patr7079.asc"
Rscript "$repo/examples/mma23p1pannonlin/exchangeable_gee_duplicate_probe.R" "$work/patr7079.asc" "$work/oracle.csv" 1 "$work/copy-id-probe.csv"
```

R 4.6.1/`gee` 4.13-30 independently reshapes the source and fits 500 replicates
with unique copied IDs, `v4.4compat=TRUE`, fixed scale 1 and the original pooled
start. Separately coded base-R moment/score iterations check every package
fit. The Python oracle checks all **173,000 ordered draw IDs and RNG steps**;
the audit checks the 500 intercepts, slopes and rho values, 23 report fields,
all source/design rows and reconstructed original mean/residual rows. The
separate original GEE workflow exports and independently checks all 1,730
prediction rows and 346 firm scores. Floating comparisons use
`1e-9 + 1e-10*abs(reference)`; observed original maximum intercept/slope/rho
refit differences were `1.18e-12`, `4.05e-13` and `7.70e-14`, respectively.

The 12-firm fixture checks all 6,000 draws and R refits, positive and negative
replicate rho values, text/JSON reporting, reversed source order and no-clobber.
The separate negative-rho original fixture hits a singular lower correlation
boundary in replicate 6 according to the independent base-R oracle; the SQL
workflow fails without publishing a draw or replacing retained state. Further
tests exercise missing/duplicate/nonfinite or out-of-range input, exact and near
correlation boundaries, zero Pearson variation, cancellation-dominated bread,
invalid start/predictor, forced iteration and RNG-work limits, and file/state
invariants. Fractional outcomes in isolated numerical probes do not bypass the
public integer-count preflight. R, Python and author data are development-only.

[Gamma-RE Poisson](gamma_re_poisson.md) is separately validated. Normal-RE Poisson, the RE bootstrap, general GEE, Figure 23.1 and other
remaining Table 23.1 outputs are separate required work. Neither this bounded
composition nor the other validated portions completes the chapter, graduate
methods core or full book inventory.
