# Household pooled-residual Breusch–Pagan random-effect LM

## Bounded contract

Deliver H-BP in the [Chapter24 companion inventory](inventory.md), source
`mma24p1olscluster.do` lines118–121: the original household seven-parameter
numeric design on 5,006 complete cases/194 communes, with pooled OLS residuals.
This is an unweighted cross-sectional commune diagnostic, not a time-series,
causal, cluster-robust or complex-survey test. It is not the ML variance LR.

Let N be the selected row count, n_g the group sizes, e the pooled OLS
residuals, R=sum(e²), Q=sum_g(sum_g e)² and M=sum_g n_g². Report signed
score z=N*(Q/R−1)/sqrt(2*(M−N)), LM=z² and the **legacy two-sided squared-score
chi-square(1)** upper-tail probability 2*Phi(−abs(z)). This is the source log's
chi2(1) convention, independently implemented by R/plm 2.6.7 `plmtest` with
`effect="individual", type="bp"`; it is not a one-sided boundary mixture.
A negative z is retained and squared, not clipped or relabelled evidence for
positive unit variance. Null calibration assumes the classical homoskedastic
independent Gaussian idiosyncratic-error model; asymptotic inference, not an
exact finite-sample guarantee. No estimated RE variance is needed for this test.

Acceptance cases:

1. Original ASCII and precise-DTA data: source cohort/order, all predictions,
   residual aggregates, signed score/LM/p against pooled R QR and plm; DTA LM
   matches the author's 432.75 to half a printed digit without widening tolerance.
2. Balanced and unbalanced/singleton controls agree with independently generated
   R QR/plm; balanced formula reduces to N/(2*(T−1)) times (Q/R−1)²,
   where N is the total observation count (not the number of groups).
3. Negative signed-score control preserves sign and uses the documented squared
   reference, not a silent positive-part boundary test.
4. JSON/text exports agree; predictions and reporting retain pooled OLS.
5. Invalid raw keys/missing predictors, no within-group pairs, singular or nearly
   perfect fit fail; export collisions preserve existing files/model ownership.

Scripts are ordered and nontransactional. Reject source rows beyond40,000,
selected groups beyond10,000, fewer than two selected groups/no pair, nonfinite
aggregates and residual RSS <=1e-20 times centered outcome sum of squares.
These are explicit bounded-work/numerical guards, not general estimator limits.
No Hausman test, native generic `estat` API, one-sided LM or whole-table parity
is part of this slice.

## Sample, state and output

The [household README](README.md) pins publisher hashes, numeric code mappings
and conversion. The script requires positive unique BIGINT `source_row`, positive
BIGINT commune IDs, finite nonmissing six predictors and finite nonmissing
outcomes before selection; only NULL outcomes are dropped. FARM/EDUC are numeric
codes, not factors. Zero log-outcomes remain. The complete-case cohort is ordered
by source_row and fitted with an intercept plus six slopes. A missing predictor
or invalid group key on an excluded-outcome row is still an input error.

`household_bp_diagnostic.csv` contains13 numerical fields: N/G, minimum/maximum
sizes, singleton count, M, Q, RSS, centered outcome sum of squares, signed score,
LM, reference DF1 and p. Additional fields name the method, legacy reference and
assumptions. `household_bp_predictions.csv` contains all selected source fields,
population `bp_xb` and `bp_residual=y−xb`, in source order. The diagnostic is also
emitted as a labelled table in JSON/text. Population predictions are pooled OLS,
not posterior/commune-effect predictions. The retained model remains the pooled
OLS fit after aggregate reporting, reactivation and exports; no RE model is fitted.

Both exports refuse existing destinations. A first export can succeed before a
second fails; prior model/table/output effects are not rolled back. Run in a
fresh work directory. Source files are not overwritten; failed exports preserve
the existing owner and retained pooled fit. The scripts add columns to the named
selected cohort, not to the source file.

## Reproduce and audit

Build `target/debug/tabdat-explore-rs`, set `repo` to this repository and prepare
`vietnam.csv` in a new `work` directory as in the household README:

```sh
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma24p1olscluster/household_bp_lm.td" > result.jsonl)
Rscript "$repo/examples/mma24p1olscluster/household_bp_lm_reference.R" /path/to/vietnam_ex1.asc "$work/reference"
python3 "$repo/examples/mma24p1olscluster/check_household_bp_lm.py" "$work" "$work/reference" /path/to/vietnam_ex1.asc
```

Use `--text` instead of `--json` in a separate fresh directory for readable
output. Both modes produce byte-identical diagnostic/prediction CSVs.

For precise DTA, in another new work directory:

```sh
Rscript "$repo/examples/mma24p1olscluster/dta_reference.R" /path/to/vietnam_ex1.dta "$work/vietnam.csv" --precise
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma24p1olscluster/household_bp_lm.td" > result.jsonl)
Rscript "$repo/examples/mma24p1olscluster/household_bp_lm_reference.R" /path/to/vietnam_ex1.dta "$work/reference" --dta
python3 "$repo/examples/mma24p1olscluster/check_household_bp_lm.py" "$work" "$work/reference" /path/to/vietnam_ex1.dta --dta /path/to/mma24p1olscluster.txt
```

R4.6.2/plm2.6.7 independently checks69 quantities: seven pooled coefficients,
49 covariance entries and13 residual/sample/LM fields, plus every source and
selected prediction row. Base-R QR residual moments and the separately fitted
[`plmtest`](https://rdrr.io/cran/plm/man/plmtest.html) BP statistic and Honda
signed score agree. Optional Python `math.erfc` independently checks the native
normal-tail calculation. Fixed absolute1e-9 numerical/prediction tolerance and
relative1e-8 for nonzero p-values are used; reference zero must equal zero.
All original source fields/keys and row membership are compared exactly.
DTA source parsing uses `foreign::read.dta(convert.factors=FALSE)` directly;
allowing factor conversion changes the design and yields a different LM.

Original ASCII LM **432.7481623551**, p **4.0997287399e-96**; precise-DTA LM
**432.7480997699**, p **4.0998573282e-96**. Both complete audits pass. The DTA
statistic matches the pinned author log's **432.75** within half its printed
last digit. This author fact does not establish full Table24.4 parity or resolve
the separate ML SE discrepancies.

Three [frozen synthetic controls](../../tests/fixtures/mma24_household_bp_lm/README.md)
validate all69 quantities and every prediction row for balanced N128, unbalanced
N150 with one singleton/missing source outcome, and balanced negative-score N128.
Five actual-script Rust tests cover those references/JSON-text exports, model
ownership and both export collisions, plus invalid keys/missing predictors,
no pairs/one group, collinearity and constant/nearly-perfect outcomes. Optional
`python3 examples/mma24p1olscluster/test_check_household_bp_lm.py` rejects14
forged artifacts, including shared native/reference score-sign corruption.
Python/R are development validation/conversion tools, not production dependencies.

Remaining classical Hausman, descriptive/graphical and unified comparison
requirements stay open; see the inventory. In particular, a small legacy BP
p-value with a negative signed score is not a positive-variance alternative test.
Neither robust error assumptions nor survey-design validity follow from this
classical diagnostic.
