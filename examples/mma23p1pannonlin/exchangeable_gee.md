# Chapter 23 patents: exchangeable Poisson GEE composition

This bounded SQL workflow reproduces the §23.3/Table 23.1 exchangeable
Poisson population-averaged fit and its working and firm-robust inference. It
is **not** a general native `xtgee` command. The [author program](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.do)
and [log](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.txt)
provide historical comparisons. CLI text and JSON report the same results;
`exchangeable_gee_predictions.csv` is sorted by `(id,year)` and includes the
original fields, GEE mean/residual and labeled repeated fit summaries.

## Sample, model and inference

Pinned `patr7079.asc` SHA-256:
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`.
Firm ID is the original source-record position, not CUSIP. The sample contains
all **346 firms/1,730 firm-years** in 1975–79, including 22 firms with zero
total patents. Missing ARDSSIC metadata do not remove rows. Model PAT with
`mu_it=exp(a+b*LOGR_it)`; there are no additional covariates, time effects,
weights or offset. Means and residuals are in-sample predictions, not fitted
count probabilities or prediction intervals.

For each balanced T=5 firm, Pearson residuals are
`e_it=(PAT_it−mu_it)/sqrt(mu_it)`. The historical/legacy moment convention is
`rho = sum_i[(sum_t e_it)^2−sum_t e_it^2] / (4*sum_it e_it^2)`.
With fixed Poisson scale 1, `R=(1−rho)I+rho*11'`,
`V_i=A_i^(1/2) R A_i^(1/2)` and `D_i=diag(mu_i) X_i`, the script solves
`U=sum_i D_i' V_i^-1 (y_i−mu_i)=0` using full Fisher-scoring steps
`B^-1 U` and recomputed rho, where `B=sum_i D_i' V_i^-1 D_i`. The native
pooled Poisson fit provides only starting values. The output separately labels:

- **Working covariance:** `B^-1`, with scale fixed at 1.
- **Firm sandwich:** `G/(G−1) * B^-1 (sum_i U_i U_i') B^-1`, without an
  additional `(N−1)/(N−2)` factor or a few-cluster correction.

Both full 2×2 matrices, coefficient SEs, z statistics and normal 95% intervals
are reported; p-values are not computed. Working inference depends on the
working variance/correlation as well as the conditional mean. Firm inference
allows within-firm score dependence but requires independent firms and a
correctly specified mean. Neither covariance establishes mean adequacy, a
causal R&D effect, a Poisson count distribution or few-cluster reliability.

| Original source, float64 | Intercept | LOGR slope |
| --- | ---: | ---: |
| GEE coefficient | 2.0675152480 | 0.5595300704 |
| Working SE | 0.01851663 | 0.004380259 |
| Firm-sandwich SE | 0.11132558 | 0.03264358 |

The converged rho is `0.8983666759`. R `gee` 4.13-30 with `v4.4compat=TRUE`,
a supplied pooled-Poisson start and fixed scale 1 uses this legacy moment
convention. Its default **modern** moment corrections are different and yield
slope about `0.5603399`; that result is not a parity target.

The historical log prints intercept `2.067515`, slope `.5595302`, working SEs
`.0185166`/`.0043803` and firm SEs `.1113256`/`.0326436`. The converged
float64 slope differs from the seven-decimal printed slope by `1.296e-7`; we do
not claim exact printed-slope parity or widen the R comparison tolerance. The
log reports nine GEE iterations and source LOGR stored as Stata `float`.
An independently coded nine-full-step probe gives slope `.5595301498` on
float64 input and `.5595301640` after float32 conversion; the latter rounds to
the historical print. This supports stopping and input precision as material
contributors, but does not establish identical Stata optimizer internals.

## Numerical, state and file boundaries

The script requires unique integer `(id,year)`, IDs 1–346, one row per year
1975–79, at least two firms, integer counts 0–1,000, and finite `|LOGR|≤20`.
It bounds `|a|,|b|≤20`, every linear predictor to `[−40,40]`, and scoring to
100 full steps. Starting coefficients are recovered from native `pooled_xb`
and must reconstruct every start prediction within `1e-12*(1+max|pooled_xb|)`.
The final scaled step must be at most `1e-12`; the recomputed
score must satisfy its `1e-10` absolute-term scale criterion. Pearson
variation and firm-sandwich diagonals must exceed `(128*epsilon)^2` times their
respective propagation scales. Both correlation-boundary gaps and the bread
Schur complement must exceed `128*epsilon` on their stated scales. Boundary
gaps are evaluated as sums of squares to avoid subtracting near-equal moments.
The script rejects invalid, nonfinite, singular, cancellation-dominated,
non-positive-definite or nonconvergent cases: it does not silently drop rows,
clamp rho, regularize the bread or publish a failed iterate. These bounds are
specific to this composition, not general GEE feasibility guarantees. Some SQL
guards currently report only `sql failed` with script location.

The native pooled Poisson fit and `pooled_xb` prediction precede the SQL GEE;
if a later step fails, those earlier session changes **remain**. A failed GEE
SQL statement does not replace the active data or prior fit. On success, the
active data become the GEE prediction relation, but the retained native model
is still the pooled Poisson fit: a later native `predict` is **not** a GEE
prediction. The GEE summaries are not restorable estimation state. Input files
are unchanged and export refuses to overwrite; use a fresh directory and check
process exit status because a report can precede an export failure.

## Reproduce and independent evidence

```sh
repo=/absolute/path/to/tabdat-explore-rs
work=/absolute/path/to/empty/work-directory
curl -fsSL https://cameron.econ.ucdavis.edu/mmabook/patr7079.asc -o "$work/patr7079.asc"
shasum -a 256 "$work/patr7079.asc"  # compare with pinned hash above
(cd "$repo" && cargo build --locked --bin tabdat-explore-rs)
sh "$repo/examples/mma23p1pannonlin/prepare.sh" "$work/patr7079.asc" "$work/patents.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma23p1pannonlin/exchangeable_gee.td" > result.jsonl)
# Development-only R package gee 4.13-30 must be installed separately.
Rscript "$repo/examples/mma23p1pannonlin/exchangeable_gee_reference.R" "$work/patr7079.asc" "$work/r-reference"
python3 "$repo/examples/mma23p1pannonlin/check_exchangeable_gee.py" "$work" "$work/r-reference" "$work/patr7079.asc"
Rscript "$repo/examples/mma23p1pannonlin/exchangeable_gee_historical_audit.R" "$work/patr7079.asc" "$work/historical-probe.csv"
```

R 4.6.1/`gee` 4.13-30 independently reshapes the raw source and fits the legacy
model. Separately coded base-R matrix equations evaluate final rho, score,
working bread and all firm scores; their full covariance matrices agree with
the package within `1e-9`. The Python checker verifies the pinned hash, exact
source/design fields, 27 report quantities, every one of the 1,730 predicted
means/residuals and all 346 firm scores. Report/prediction comparisons use
`1e-9 + 1e-10*abs(reference)`; scores use `1e-8 + 1e-10*abs(reference)` to
account for score cancellation and 15-digit R CSV serialization. Observed
maximum report difference was `6.34e-12`, prediction difference `6.51e-12`,
and firm-score difference `6.12e-11`. Original-source text and JSON exports
were byte-identical. Selected historical values are checked
at their printed precision except the separately disclosed slope discrepancy.

Two 12-firm R fixtures cover positive and negative rho, all predictions, both
covariances and all firm scores. Integration tests exercise text/JSON,
reversed input, no-clobber and preserved data/model state, invalid/missing/
duplicate inputs, zero Pearson variation, exact and near correlation
boundaries, cancellation-dominated bread, parameter bounds and forced bounded
nonconvergence. Fractional outcomes in isolated numerical-boundary probes do not
bypass the public integer-count preflight. R, Python and author data are
**development-only**; production does not depend on them.

The separate [GEE firm bootstrap](exchangeable_gee_bootstrap.md) and [NLS firm bootstrap](exponential_nls_bootstrap.md) are validated under explicit conventions. Historical GEE bootstrap ID handling and its reported SE remain unresolved.
Random-effects Poisson, general GEE, RE firm bootstraps, Figure 23.1 and
remaining Table 23.1 outputs remain separate required work. This bounded
composition does not complete the chapter, graduate-method core or book
inventory.
