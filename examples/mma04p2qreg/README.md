# Conditional household expenditure quantiles

This native workflow covers the .1/.5/.9 fits from Cameron–Trivedi (2005)
§4.6.4 pp. 88–90, author [`mma04p2qreg.do`](https://cameron.econ.ucdavis.edu/mmabook/mma04p2qreg.do)
block 1: log household medical expenditure on log total expenditure with an
intercept. The selected initial illustration is the conditional median.
Separate fitted quantiles do not validate the simultaneous bootstrap in
Figure 4.1. The native Figure 4.2 workflow is below.

Download [`qreg0902.asc`](https://cameron.econ.ucdavis.edu/mmabook/qreg0902.asc)
to a working directory. Its SHA256 is
`eb1b422fa29a096e608ff5f8912ee1b9fcecc749d227fbfd37b19d3b18db8b0b`.
The original 5,999 logical records have nine fields and span physical lines;
quoted farm/urban labels must not be split as whitespace. The preparation
script validates this source and refuses replacement of an existing CSV:

```sh
python3 /path/to/repo/examples/mma04p2qreg/prepare_qreg.py qreg0902.asc qreg.csv
/path/to/repo/target/debug/tabdat-explore-rs --text -f /path/to/repo/examples/mma04p2qreg/quantiles.td
```

Run from the directory containing `qreg.csv`. The script checks source identity,
selects jointly finite log expenditure variables in source order and fits all
three quantiles. The 993 missing medical-log observations are excluded, leaving
5,006. Valid **log medical expenditure of zero is retained**; it is not a
missing/raw-zero expenditure observation. Current-row fitted quantiles and
median residuals are exported with original identifiers and design values to
`quantile_predictions.csv`; an existing destination is never replaced.
Neither the runtime nor the native workflow requires Python or R after CSV
preparation. Runtime DTA ingestion remains unavailable.

Inference assumes independent observations and a common location-error density
independent of X. Hall–Sheather residual-spacing sparsity covariance follows
R quantreg `summary.rq(se="iid", hs=TRUE)`; native coefficient tests/95% intervals
are **asymptotic normal**, not that function's finite-t tails. No robust,
clustered, weighted, survey or cross-quantile inference is claimed. Loss is the
check loss; at the median this is half the sum of absolute deviations. It is
not a Gaussian likelihood. The displayed pseudo R² compares this loss to an
intercept-only empirical quantile.

The original ASCII median gives intercept 0.592154070821636 and slope
0.621092674329863 (IID slope SE 0.0386057102526042). The author's historical DTA
output differs in the last point-estimate digits because of input precision;
its Stata 8 uncertainty convention also differs from the explicit native IID
spacing convention. These are not exact printed-output replications, and
unexplained differences are not accepted by widening tolerances.

The bounded sparse LP uses Clarabel 0.11.1 with owned CSC inputs, QR scaling and
polishing, a strict solved status and numerical gap/score checks. Ambiguous or
unresolved coefficients, rank deficiency and insufficient/invalid residual
spacing fail explicitly. See `help qreg` for numerical bounds. Clarabel is
Apache-2.0; its Rust QDLDL internals use unsafe indexing, isolated behind the
safe owned solver API. Python/native/BLAS/SDP optional solver features are off;
`serde` is enabled because this release otherwise fails to compile upstream.

## Independent evidence

Development reference: R 4.6.1, quantreg 6.1, `rq(method="br")`, IID/HS covariance,
normal tails/intervals computed explicitly. From a fresh audit directory:

```sh
Rscript /path/to/repo/examples/mma04p2qreg/quantiles_reference.R qreg0902.asc reference
/path/to/repo/target/debug/tabdat-explore-rs --json -f /path/to/repo/examples/mma04p2qreg/quantiles.td > quantiles.jsonl
python3 /path/to/repo/examples/mma04p2qreg/check_quantiles_reference.py . reference qreg0902.asc
```

Remove a prior generated export deliberately before another run. The audit pins
all raw fields and all 5,999 source identifiers, compares 66 R coefficient,
full-covariance, SE, normal-test/interval, loss, sparsity and pseudo-R² quantities,
and 35,042 identity/design/prediction cells across 5,006 retained rows. Absolute
tolerance is 1e-8; tiny normal p-values use relative 1e-7 plus 1e-300. Text and
JSON runs produce byte-identical prediction exports. No whole-book claim follows.

Small committed fixtures cover the same script, missing/nonfinite exclusions,
source-key failure and rank failure, export refusal and text/JSON agreement.
Core references additionally cover two predictors, noconstant, shifted outcomes
and rescaled predictors (185 quantities), failure/ambiguity and scale invariance.
Runtime tests verify retained sample membership, fit state, current-row prediction,
failed-fit clearing and nonmutating prediction errors.

## Figure 4.2: observed outcomes and conditional quantiles

From a fresh directory containing `qreg.csv`, run:

```sh
/path/to/repo/target/debug/tabdat-explore-rs --text -f /path/to/repo/examples/mma04p2qreg/figure.td
```

This composes `quantiles.td` and the existing SVG renderer, publishing the CSV
export and `quantile_lines.svg`. Both destinations must be unused. On a later
plot failure the earlier completed CSV remains; the script is not a multi-file
transaction. Existing SVG destinations are never replaced. No automatic viewer
opens. The figure contains all 5,006 finite observed pairs and the independently
validated q90/q50/q10 fitted lines, with solid/dashed/dotted distinctions and
explicit legends. Axes include all observations and line vertices; no outcome
clipping or jitter is applied. Lines sort stably by x and original row index.

The horizontal axis is log total expenditure and the vertical axis is log
medical expenditure. These titles match the actual plotted variables; the
author's historical do-file reverses its textual axis titles. This is a native
SVG counterpart to Figure4.2, not a Stata WMF/pixel/font reproduction. Rendering
does not add uncertainty bands or simultaneous inference. The flattened lower
quantile and steeper upper quantile describe distributional differences, not
causal effects or an independent test of slope equality.

With the R reference directory above:

```sh
python3 /path/to/repo/examples/mma04p2qreg/check_figure.py reference/predictions.csv quantile_lines.svg
```

The audit compares every observed coordinate and all 15,018 independent R line
vertices, stable row ordering, axes/legends, padded bounds and actual SVG path/
screen geometry (1e-8 absolute tolerance). The full original-source SVG was also
rasterized offline and visually inspected for axes, legend, line distinction and
clipping. Text/JSON runs produce byte-identical SVGs and CSVs. A small native test
checks reference vertices, artifact metadata, labels and no-clobber behavior;
existing scatter tests cover model/data preservation and publication failures.
The artifact embeds raw coordinates and candidate-row indices, as documented in
`help scatter`; it is not anonymized.
