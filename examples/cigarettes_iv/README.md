# Overidentified cigarette-demand IV example

This reproduces the **AER `ivreg` example**, not an empirical illustration from
Cameron–Trivedi (2005). It validates Sargan, independent-row robust overidentification
score, classical Wu–Hausman and HC1 joint endogeneity methods needed by the
graduate IV workflow; it does not advance the
book's empirical-illustration status.

- Specification: [AER 1.2-12 `ivreg` documentation](https://search.r-project.org/CRAN/refmans/AER/html/ivreg.html).
- Data: external [Rdatasets AER/CigarettesSW CSV](https://vincentarelbundock.github.io/Rdatasets/csv/AER/CigarettesSW.csv),
  derived from the AER dataset. No source data are redistributed here.
- Expected CSV SHA-256: `629e887a8104d483d450a9bbe487cb5613e62d116c4ecc84d6c8893b7393fcd2`.
  A changed checksum requires inspecting the source/sample, not regenerating
  expected numerical results without explanation.

## Reproduce

From the repository root (network needed only to acquire the external CSV):

```sh
cargo build --locked --bin tabdat-explore-rs
repo="$PWD"
work=$(mktemp -d)
curl -fsSL https://vincentarelbundock.github.io/Rdatasets/csv/AER/CigarettesSW.csv \
  -o "$work/CigarettesSW.csv"
shasum -a 256 "$work/CigarettesSW.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --text \
  -f "$repo/examples/cigarettes_iv/overid.td")
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/cigarettes_iv/overid.td")
Rscript examples/cigarettes_iv/reference_overid.R "$work/CigarettesSW.csv"
```

R is an optional **development reference**, not a runtime dependency. The
committed `.td` script uses native DuckDB SQL natural logs, selects `year=1995`,
and orders the 48 states by their state code. The reference checks all transformed
values are finite, N=48 and no duplicated states. No weights or additional row
exclusions are used.

Model with an intercept:
- Outcome: `ln(packs)`.
- Included exogenous control: `ln(income/population/cpi)`.
- Endogenous regressor: `ln(price/cpi)`.
- Excluded instruments: `(taxs-tax)/cpi`, `tax/cpi`.

## Numerical evidence and interpretation

Base R 4.6.1 `lm.fit` independently projects the design onto all instruments,
fits 2SLS coefficients, forms **structural** residuals against the original design,
and regresses those residuals on all instruments. References:

| Quantity | Independent R reference |
| --- | ---: |
| Intercept | 9.8949555411551628 |
| Log real-income coefficient | 0.2804048250834234 |
| Log real-price coefficient | -1.2774241334272722 |
| Sargan N*R² | 0.3326221419365246 |
| Chi-square DF | 1 |
| Upper-tail p-value | 0.5641191400175745 |

Native replay matches the coefficient/statistic/p-value references within 1e-12
absolute tolerance, including the exact sorted state identities and N. The
reference rounds to Sargan 0.333, p=0.5641. The AER documentation specifies the
model; numerical validation here executes base R, not the AER package.

This is the **homoskedastic, asymptotic Sargan N*R² convention**, using
`N*(u'P_Z u)/(u'u)`, with DF equal to excluded-instrument count minus endogenous
count. It is not an N−K-corrected OIR statistic, Basmann, robust Hansen J, or an
exact finite-sample F test. Conventional strong-identification asymptotics and
the correctly specified maintained model are required. A large p-value does
**not** prove instrument validity or resolve weak identification. Rejection may
reflect invalid instruments or model misspecification; the test does not identify
which instrument is invalid. Use substantive design knowledge and the separately
labelled relevance/weak-IV diagnostics.

`estat overid` reuses the retained 2SLS sample/design/structural residuals without
refitting or mutating state. Just-identified or cluster parent fits and undefined
residual variance/rank/DF fail explicitly. Independent-row robust parents return
the separate score test below, not Sargan. Hansen J and GMM remain required work.

## Classical joint Wu–Hausman endogeneity diagnostic

Using the same `repo` and `work` directories from above:

```sh
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --text \
  -f "$repo/examples/cigarettes_iv/endogenous.td")
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/cigarettes_iv/endogenous.td")
Rscript examples/cigarettes_iv/reference_endogenous.R "$work/CigarettesSW.csv"
```

This uses the identical pinned/sorted 48-state sample and 2SLS specification,
reports Sargan and then `estat endogenous`. Base R4.6.1 independently fits the
full-instrument first stage and compares OLS on the original regressors against
OLS augmented by the first-stage innovation:

| Quantity | Independent R reference |
| --- | ---: |
| Restricted OLS RSS | 1.578794093504594 |
| Augmented OLS RSS | 1.475890440962176 |
| Wu–Hausman F(1,44) | 3.067816272944078 |
| Upper-tail p-value | 0.08682504624130956 |

Native F and p match within 1e-12 absolute tolerance. The null tests exogeneity
of **all** declared endogenous regressors, conditional on valid instruments and
the maintained model. Here m=1 and K=3, giving N-K-m=44 residual DF. Unlike
Sargan, this test also applies to just-identified designs.

The R reference separately computes an **HC0 contrast**, F=3.823467179945732,
p=0.05690916902691472. That is a different auxiliary covariance convention, not
the implemented homoskedastic RSS test. No AER package execution, robust/cluster
endogeneity is implied by that classical comparison. The separate HC1 workflow
below uses a different correction/reference, not automatic HC0 substitution.
Ordinary strong-identification conditions apply; exact finite-sample F additionally requires
conditional normality. Failure to reject does not prove exogeneity or IV validity.

The diagnostic uses auxiliary fits on the retained problem, with no 2SLS refit,
current-data scan or model/data mutation. Auxiliary predictors are norm-scaled
without changing their spaces. Numerically degenerate first-stage innovations
(norm at most 1e-12 relative to the endogenous regressor after included controls)
and augmented residuals (norm at most 1e-12 relative to centered outcome norm,
uncentered for noconstant) are errors, not p-values asserting exogeneity. Rank,
DF, constant-outcome and nonfinite cases also fail explicitly. The joint
homoskedastic Wu F and separate HC1/CR1 Wald below are installed; subset endogeneity and other variants remain separate work. This adds method
evidence, not a book replication.

## Robust joint regression-based endogeneity Wald

On the identical pinned 48-state sample and specification:

```sh
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/cigarettes_iv/robust_endogenous.td" > robust-endogenous.jsonl)
Rscript examples/cigarettes_iv/reference_robust_endogenous.R \
  "$work/CigarettesSW.csv" "$work/expected_robust_sample.csv"
python3 examples/cigarettes_iv/check_robust_endogenous.py "$work"
```

The checker and R are development-only, not runtime dependencies. Base R4.6.1
independently fits the full-instrument OLS first stage and augmented OLS. With
N=48, original K=3 and m=1, auxiliary K=4 and residual DF=44, it forms HC1
`48/44 * (A'A)^-1 sum(a_i*a_i' * e_i^2) (A'A)^-1` and tests the innovation
coefficient. Independent references are Wald chi-square(1)=**3.504844914950468**,
p=**0.06118957434570806**, correction=**1.090909090909091**. Native quantities
match within 1e-12 absolute, as do all 240 prepared numeric cells and the exact
48 ordered state identities. The exported sample preserves the parent design.

This is the joint residual-inclusion **asymptotic chi-square** convention, not
the classical F, HC0 contrast, robust score test, GMM C statistic or Stata finite-F
reference. All original regression coefficients remain unrestricted. Under the
joint null the first-stage nuisance estimation does not contribute at first
order; valid instruments, correct linear specification and conventional strong
identification are required. Rejection/failure to reject cannot identify invalid
instruments, prove exogeneity or resolve weak identification.

One-way parent `cluster(unit)` uses the same augmented regression with grouped
OLS scores and `G/(G-1)*(N-1)/(N-Kaux)` CR1 covariance, not the parent 2SLS
parameter count. Independent growing clusters are required; no few-cluster
reliability is claimed. This particular 1995 cross-section does not validate
cluster dependence: the five synthetic designs in
`crates/tabdat-stats/tests/fixtures/robust_endogeneity_reference.R` separately
validate HC1 and ten-group CR1 (single/multiple endogenous, just/overidentified,
intercept/no-intercept and no-control cases) at 1e-11 absolute statistic/p tolerance.
These are method references, not new Cameron–Trivedi empirical coverage.

The result is read-only on the retained complete-case design and cluster IDs,
even after edits/drop of current columns. Degenerate innovations/augmented
variance, rank/DF failures and singular innovation Wald covariance fail explicitly
without changing the dataset/fit; later script effects do not execute. No subset,
survey, multiway/HAC or weak-IV confidence-set extension is installed by this
endogeneity workflow. The separate overidentification score is described below.

## Independent-row robust overidentification score

Using the pinned source and fresh `work` directory from the initial acquisition
(the export path must not already exist):

```sh
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/cigarettes_iv/robust_overid.td" > robust-overid.jsonl)
Rscript examples/cigarettes_iv/reference_robust_overid.R \
  "$work/CigarettesSW.csv" "$work/expected_robust_overid_sample.csv"
python3 examples/cigarettes_iv/check_robust_overid.py "$work"
```

The new result is `RobustOveridScoreResult`, not `SarganResult` or Hansen J.
Let Z contain the intercept, included controls and excluded instruments, and
Xhat=P_Z X the instrument-projected original regressors. Residualize Z on Xhat;
its complement has rank q=L-K. Regress ones **without an intercept** on products
of q independent complement directions and structural 2SLS residuals u. The
Wooldridge statistic is N-RSS, evaluated as the fitted projection norm. Its
uncentered score OPG is **uncorrected** (factor1), not the coefficient HC1 factor
N/(N-K). Under the null this projection removes the first-order 2SLS nuisance
estimation effect. An arbitrary-weight quadratic of all raw IV moments is not
instead assumed to have a chi-square(L-K) law.

Independent base R4.6.1 uses SVD/lm.fit and directly confirms the equivalent
score moment quadratic. For N=48, L=4, K=3 and q=1, it returns
chi-square(1)=**0.3347358817061415**, p=**0.5628836468492445**. Native score/tail
match at 1e-12 absolute; the checker also verifies the pinned source checksum,
all 48 ordered state identities and all 240 prepared numeric cells. This is
method evidence, not a new validated Cameron–Trivedi empirical illustration.

`crates/tabdat-stats/tests/fixtures/robust_overid_score_reference.R` independently
checks five synthetic q1/q2 designs, mixed first stages, one/two endogenous,
intercept/no-intercept and no-control cases. Native pivoted, twice-reorthogonalized
complement directions match R SVD values at 1e-11 absolute, including invariance
to instrument reordering/invertible coordinate changes and residual units.
Unit-scaled rank checks reject singular complement or score spaces instead of
silently reducing q. Undefined structural residuals, just-identification,
insufficient auxiliary DF and full-instrument/projected-design rank failures
also fail without changing the retained fit/data; subsequent script effects stop.

Inference is **asymptotic chi-square(q)** under independent rows, correct linear
specification, valid instruments under the null and conventional strong
identification. It is not an exact F, cluster/survey, few-cluster or weak-IV
procedure. Cluster parent fits remain explicitly unsupported. Rejection may
reflect invalid instruments or model misspecification and does not isolate an
instrument; failure to reject proves neither validity nor identification.
R/Python here are optional development-only references/checkers, not runtime
dependencies. Equivalent validated score inference, not historical Stata
syntax or internal basis choices, is the acceptance criterion.
