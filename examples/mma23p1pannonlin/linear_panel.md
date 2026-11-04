# Chapter 23 log-patents linear panel

This bounded native composition runs the pooled-cluster OLS, within-FE and
Swamy–Arora RE-GLS sequence in the March 2005
[author program](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.do)
and [log](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.txt) for
the §23.3 patents/R&D illustration in Cameron and Trivedi (2005). It loads and
checks the panel, constructs the outcome, fits and diagnoses each model,
reports native text or JSON results, and exports ordered predictions. The
separate author Figure 23.1 and remaining Table 23.1 analyses are not covered
by this workflow.

## Sample and specification

Pinned `patr7079.asc` SHA-256:
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`.
Firm `id` is original source-record position, not CUSIP. Use all **346 firms
and 1,730 1975–79 observations**, including 337 zero-count observations, 22
firms with zero counts in all five years, and missing nonmodel ARDSSIC. The
only regressor is LOGR plus an intercept or absorbed firm intercepts; no extra
covariates, year effects, weights or offsets are included. The author's ad hoc
outcome is `LPAT=ln(PAT)` when PAT is positive and `LPAT=ln(0.5)` when PAT is
zero. The constant substitution is neither a count likelihood nor an estimate
of a log for zero; its choice affects the outcome and interpretation.

The script runs in order:

1. Pooled OLS `LPAT ~ 1 + LOGR` with firm-cluster covariance. It reports native
   slope `lincom` and zero-slope F test, then pooled mean/xb and residuals.
2. One-way within FE `LPAT ~ LOGR` with absorbed firm intercepts and classical
   covariance. It reports slope `lincom` and F test, the conventional classical
   all-equal-firm-intercepts F test, and normalized and conditional predictions.
3. Balanced one-way Swamy–Arora RE-GLS `LPAT ~ 1 + LOGR` with classical
   transformed-RSS covariance; export population xb and residuals. Summaries
   of each prediction/residual are descriptive, not serial-correlation tests.

OLS cluster covariance uses `G/(G−1)*(N−1)/(N−K)` with G=346 and K=2, and
cluster coefficient/linear-test inference uses t/F with **G−1=345** degrees of
freedom. Classical FE uses `RSS/(N−G−p)` with one slope and **N−G−p=1383**
degrees of freedom; the unit-intercept equality F has (345,1383) degrees of
freedom. FE's common intercept is the normalization
`mean(LPAT)−mean(LOGR)*slope`, a **point only**, not an inferred coefficient:
native FE does not provide common or individual intercept SEs, so it does not
reproduce that part of the author's FE column. RE estimates within and between
variance components with the balanced Swamy–Arora convention, and uses its
quasi-demeaned residual variance `RSS*/(N−2)` for the full two-parameter
covariance, normal coefficient inference and a one-slope chi-square Wald test.

| Original float64 | LOGR slope | Slope SE | Intercept / normalization |
| --- | ---: | ---: | ---: |
| Pooled, firm-cluster | 0.8340744986 | 0.0228654911 | 0.7954785111 |
| Within FE, classical | 0.1067504940 | 0.0560363644 | 1.7091160062 (point only) |
| RE-GLS, classical | 0.7202376566 | 0.0237986367 | 0.9384761387 |

FE within R² is `0.0026172160`; the classical all-equal-intercepts statistic
is `F(345,1383)=16.9646511`. RE's `sigma_e²=0.2613158761`,
`sigma_u²=0.8110361205`, `theta=0.7539534939` and `rho=0.7563152054`.
The native FE **conditional** R² includes fitted firm effects and is not the
historical overall/between R². Firm clustering assumes independent firms and
is not a few-cluster guarantee; classical FE and RE inference instead assumes
their respective error and random-effect conditions. FE removes additive
firm-constant effects, not all endogeneity; RE additionally assumes suitable
random-effect exogeneity. These analyses are not causal identification and
neither their differences nor a working Gaussian likelihood establishes model
adequacy.

## Predictions, state and files

The sorted `linear_panel_predictions.csv` retains all source columns, `lpat`,
pooled xb/residual, FE normalized xb, conditional `xbu`, normalized firm effect
`u` and conditional residual, and RE population xb/residual. FE `xbu` adds the
retained firm-specific total intercept; `xb` uses the common normalization and
is not a conditional fitted level. RE `xb` is population prediction, not a
firm-specific random-effect posterior prediction. These are in-sample point
predictions, not prediction intervals.

The script rejects duplicate/missing keys, noninteger IDs outside 1–346,
noninteger years outside 1975–79, missing/nonfinite model values, noninteger or
negative PAT counts, counts over 1,000, `|LOGR|>20`, fewer than two firms, and
anything other than one row per firm/year in all five years. Existing `lpat`
or prediction-column names are rejected **case-insensitively** before any
transformation or fit, rather than silently using or renaming user columns.
Input may be reversed or interleaved; the export is sorted by
`(id,year)`. These are bounded workflow rules, not general native panel limits.

Script execution is **not transactional**. After successful execution, the
active data include all prediction columns and the retained native model is
RE-GLS, not pooled OLS or FE. Earlier fits/predictions are not restorable model
snapshots. In an actual time-invariant-X fixture, pooled OLS and predictions
succeed but FE fails; in an identical-panel fixture pooled/FE succeed but RE's
between design is singular. In either fit failure the failed native fit clears
previous estimation state, while earlier data columns remain and the failed
fit does not publish a model. Input files are unchanged; export refuses to
overwrite. Use a fresh directory and check process exit status, since model
reports can precede an export failure.

## Reproduce and independent evidence

```sh
repo=/absolute/path/to/tabdat-explore-rs
work=/absolute/path/to/empty/work-directory
curl -fsSL https://cameron.econ.ucdavis.edu/mmabook/patr7079.asc -o "$work/patr7079.asc"
shasum -a 256 "$work/patr7079.asc"  # compare with the pinned hash above
(cd "$repo" && cargo build --locked --bin tabdat-explore-rs)
sh "$repo/examples/mma23p1pannonlin/prepare.sh" "$work/patr7079.asc" "$work/patents.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma23p1pannonlin/linear_panel.td" > result.jsonl)
Rscript "$repo/examples/mma23p1pannonlin/linear_panel_reference.R" "$work/patr7079.asc" "$work/r-reference"
python3 "$repo/examples/mma23p1pannonlin/check_linear_panel.py" "$work" "$work/r-reference" "$work/result.jsonl" "$work/patr7079.asc"
```

Development-only R 4.6.1/`plm` 2.6.7 independently constructs the raw source
and all three sample/designs, computes clustered OLS from firm score sums,
within FE from `plm`, and RE from `plm` plus independently checked
quasi-demeaning and variance components. The audit checks all **1,730 exact
source/design and R prediction rows** and **91** fit, full covariance,
`lincom` and test quantities. Floating comparisons use
`1e-9+1e-10*abs(reference)`; nonzero p-values use relative tolerance `1e-9`.
Maximum original prediction difference is `4.0e-14`. Nineteen selected
historical logged numbers agree at their printed precision; this is not a
claim that all Stata storage, graphical output or unreported FE-intercept
inference has been reproduced. Original text/JSON exports are byte-identical.

An inspectable 40-firm fixture, including eight all-zero firms and missing
nonmodel ARDSSIC, checks all 200 rows and 91 quantities, text/JSON and byte-
identical exports, reversed ordering and no-clobber. Tests cover invalid,
missing, duplicate, nonfinite, fractional and out-of-range model/key data,
case-insensitive outcome/prediction-name collisions, actual singular within
and between designs, and
model/data/file failure invariants. R/Python and author data are development-only.

Figure 23.1, the full set of Table 23.1 outputs, broader panel methods and the
remaining book inventory remain required. This slice does not complete the
chapter, graduate methods core or full textbook target.
