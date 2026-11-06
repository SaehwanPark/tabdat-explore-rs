# Chapter 24 household Gaussian random-intercept ML

This bounded workflow covers the author's §24.7/Table24.4 household
`xtreg ..., mle` analysis, separately from the [RE-GLS](household_re_gls.md)
workflow and the [equal-group-between RE-GLS method](household_equal_group_re.md).
It uses native unbalanced Gaussian marginal ML, retains its fitted model and
exports population predictions. It is not a complex-survey, causal, GEE,
posterior commune-effect or whole-table analysis. Source hashes and preparation
are in the [household README](README.md).

## Sample, fit and interpretation

Outcome `lnexp12m`, six numeric predictors `lnhhexp age sex hhsize farm educ`
and intercept, with a shared Gaussian random intercept per `commune`. Numeric
EDUC/FARM codes and SEX 1/2 are not factor-expanded. From 5,999 source records,
993 missing outcomes are excluded, retaining **5,006 households in 194 communes**,
sizes1..39 with one singleton. Zero log-outcomes remain. `source_row` is an
artificial unique within-group key, **not time**: no lag or serial interpretation
follows. The script rejects missing/nonpositive/non-BIGINT raw source/commune
keys, nonfinite or missing predictors and nonfinite nonmissing outcomes before
selection. The ordered joint-complete cohort is fitted; the native fit itself
reports 5,006 candidate cases and no further exclusions. Smaller valid input
CSVs can be used subject to the native fit's rank, group and convergence bounds.

The native model assumes independent commune effects and idiosyncratic Gaussian
errors, with commune effects exogenous to covariates; it is neither robust
cluster inference nor design-based survey inference. It profiles nonnegative
unit variance, independently re-estimating the intercept-only null, and reports
normal95 coefficient inference from joint observed information, full covariance,
slope Wald, marginal log likelihood, slope LR where both unit variances are
interior, and a one-sided 50:50 chi-square(0,1) unit-variance LR test. Positive
components get delta SEs and log-Wald intervals. When the unit variance is zero,
its regular normal-Wald inference is withheld, constrained information is used
and the unit LR p-value is 1. If either full or restricted unit variance is on
the boundary, ordinary slope-LR chi-square inference is withheld. The finite
profile scan `[0,24]` in eta with 0.5 grid and polished crossings is a bounded
search, not a general global-optimality proof; identification, rank, positive
information and convergence guards may reject an input.

`ml_xb` and `ml_residual` are population linear predictor and observed y−xb,
not unit-specific or posterior predictions. The export contains all selected
source fields in source-row order; the retained model remains the original ML
fit after predictions, summaries and export. Export collisions do not overwrite
existing files or the source and do not replace the retained fit. Scripts are
**not transactional**: prior table/model effects are not rolled back. Run in a
fresh working directory and explicitly remove an old export before rerunning.

## Reproduce and audit

Build the binary, set `repo` to the repository path, prepare `vietnam.csv` in a
fresh `work` directory as in the household README, then run:

```sh
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma24p1olscluster/household_re_ml.td" > result.jsonl)
Rscript "$repo/examples/mma24p1olscluster/household_re_ml_reference.R" /path/to/vietnam_ex1.asc "$work/reference"
python3 "$repo/examples/mma24p1olscluster/check_household_re_ml.py" "$work" "$work/reference" /path/to/vietnam_ex1.asc
```

An independent R 4.6.2 profile likelihood, analytic joint observed information
checked against a finite-difference Hessian, and `nlme` 3.1.171 likelihood and
coefficients validate **210 quantities**, all **5,999 source rows** and all
**5,006 prediction rows** on both ASCII and DTA. Fixed absolute 1e-9 tolerance
is used for fit/prediction comparisons and relative 1e-8 for nonzero p-values;
source fields, rows and IDs are exact. Interior profile scores are individually
required to be <1e-8 in absolute value because native eta bisection and R's
root solver leave slightly different residual scores; this is a stationarity
check, not a loosened coefficient tolerance. Maximum non-score scalar and
prediction differences: ASCII **5.59e-10 / 3.76e-13**; precise DTA **8.81e-10 /
2.45e-13**. Maximum nonzero-p relative differences are **2.76e-10** and
**4.36e-10**, respectively. Both source modes' text and JSON coefficient/SE/z/
p/interval rows match numerically and prediction CSVs are byte-identical.

Three unbalanced synthetic controls each have 180 source/179 complete cases,
16 communes and a singleton: interior full/null, interior full with boundary
null, and boundary full/null. They validate 210, 209 and 187 independent R
quantities plus all predictions and boundary/withholding behavior; see
`tests/fixtures/mma24_household_re_ml/README.md`. Twenty-five full-auditor and
one author-point forgeries were rejected. R/Python are optional development
reference tools, not production dependencies.

For a separate DTA audit, in a fresh directory:

```sh
Rscript "$repo/examples/mma24p1olscluster/dta_reference.R" /path/to/vietnam_ex1.dta "$work/vietnam.csv" --precise
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma24p1olscluster/household_re_ml.td" > result.jsonl)
Rscript "$repo/examples/mma24p1olscluster/household_re_ml_reference.R" /path/to/vietnam_ex1.dta "$work/reference" --dta
python3 "$repo/examples/mma24p1olscluster/check_household_re_ml.py" "$work" "$work/reference" /path/to/vietnam_ex1.dta --dta
python3 "$repo/examples/mma24p1olscluster/check_household_re_ml_author.py" /path/to/vietnam_ex1.dta /path/to/mma24p1olscluster.txt "$work/reference" "$work/result.jsonl"
```

The `--precise` optional conversion emits 17 digits to round-trip DTA binary64;
the original default remains unchanged. Against the pinned author log, all
**seven coefficient points and two of seven coefficient SEs** match both R and
native to half a printed digit. The other five coefficient SEs (`intercept`,
`lnhhexp`, `sex`, `hhsize`, `farm`) and both printed component-SD SEs differ
at the displayed precision. E.g. intercept SE is **0.4192870181** versus
**0.4192856**, and commune-SD SE **0.0329750004** versus **0.0329742**.
The log likelihood, slope and unit-variance LR statistics, rho, and both
component-SD points do match their displayed precision. The causes of the
historical SE differences are unverified; fixed tolerances were not widened,
and historical optimizer or whole-table parity is not claimed.

Other household RE diagnostics, other GEE workflows, genuine survey weights/
strata and remaining textbook outputs are required; this slice does not complete
the chapter or roadmap.
