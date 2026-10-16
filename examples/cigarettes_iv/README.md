# Overidentified cigarette-demand IV example

This reproduces the **AER `ivreg` example**, not an empirical illustration from
Cameron–Trivedi (2005). It validates Sargan and classical Wu–Hausman methods needed
by the graduate IV workflow; it does not advance the book's empirical-illustration status.

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
refitting or mutating state. Just-identified or robust/cluster parent fits and
undefined residual variance/rank/DF fail explicitly. Robust/cluster Hansen J and
GMM remain separate required work.

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
endogeneity or automatic covariance substitution is implied. Ordinary strong-
identification conditions apply; exact finite-sample F additionally requires
conditional normality. Failure to reject does not prove exogeneity or IV validity.

The diagnostic uses auxiliary fits on the retained problem, with no 2SLS refit,
current-data scan or model/data mutation. Auxiliary predictors are norm-scaled
without changing their spaces. Numerically degenerate first-stage innovations
(norm at most 1e-12 relative to the endogenous regressor after included controls)
and augmented residuals (norm at most 1e-12 relative to centered outcome norm,
uncentered for noconstant) are errors, not p-values asserting exogeneity. Rank,
DF, constant-outcome and nonfinite cases also fail explicitly. Only the joint
homoskedastic Wu F is installed; robust/cluster/subset endogeneity and other
variants remain separate work. This adds method evidence, not a book replication.
