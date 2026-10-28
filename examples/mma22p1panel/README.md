# Chapter 22 first-difference panel IV

This workflow reproduces the base-instrument 2SLS column of Cameron and Trivedi
§22.3, Table 22.2, using the authors' [MOMprecise.dat](https://cameron.econ.ucdavis.edu/mmabook/MOMprecise.dat)
and [mma22p1pangmm.do](https://cameron.econ.ucdavis.edu/mmabook/mma22p1pangmm.do).
The precise data differ from Chapter 21's two-decimal `MOM.dat`. The source
SHA256 is `332c875eaa00ecc08a16c75424e4c56aa3fcf13c6ee5049768ed3d08a2bfef9c`.
The author program instruments all five differenced regressors, despite an
earlier comment describing only wage as endogenous; this workflow follows the
executable command.

[`base_2sls.td`](base_2sls.td) forms differences and lags within person, sorts by
person and year, drops 1979 and 1980, and fits no-intercept 2SLS to 4,256
person-years from 532 people. The dependent variable is the change in log
hours. The endogenous regressors are changes in log wage, children, age, age
squared and disability. The nine instruments are levels of the four other
regressors lagged one and two years, plus log wage lagged two years. No current
differenced exogenous variables are added to the instrument list. The script
prints classical and person-clustered covariance results and conventional
first-stage diagnostics, then exports the exact design and fitted/residual
values for the final fit.

The log-wage coefficient is 0.209109; its classical SE is 0.388633 and
person-clustered SE is 0.374170. The classical covariance uses residual
variance divided by `N-K`; the clustered sandwich uses person score sums times
`G/(G-1) * (N-1)/(N-K)` with a `t(G-1)` reference. Conventional first-stage F
for differenced wage is approximately 2.38. With multiple endogenous variables,
this statistic alone does not establish identification strength; ordinary 2SLS
Wald inference is not weak-instrument robust. Instrument validity and panel
independence assumptions are not established by reproducing the table.

The script requires each person to have exactly one finite observation per year
1979–1988; duplicates, missing years, nonfinite data and rank failures stop the
workflow before export. The export refuses to overwrite an existing file; prior
session effects are not rolled back. The converter below verifies the source
hash and publishes without clobbering an existing file. Python and R are needed
only for source conversion and reference checking, not the application. This
is a bounded source workflow, not a general panel-GMM command. Two-step GMM and dynamic-panel methods remain unimplemented here.

From a fresh directory:

```sh
python3 /path/to/repo/examples/mma22p1panel/prepare.py /path/to/MOMprecise.dat MOMprecise.csv
/path/to/tabdat-explore-rs --json -f /path/to/repo/examples/mma22p1panel/base_2sls.td > result.jsonl
Rscript /path/to/repo/examples/mma22p1panel/reference.R /path/to/MOMprecise.dat reference
python3 /path/to/repo/examples/mma22p1panel/check_reference.py /path/to/MOMprecise.dat result.jsonl panel_iv_design.csv reference
```

R 4.6.1 independently constructs all lags and differences and computes 2SLS
and both covariance matrices by QR projection. The check compares every exported
row and fitted/residual value, coefficients, standard errors, p-values and both
full covariance matrices at `1e-9 + 1e-9*abs(reference)`. It also compares the
15 coefficient/SE values at Table 22.2's three-decimal precision. The author's
Stata log records input variables as `float`, so its longer printed values are
not claimed to match the present double-precision input exactly. Focused CLI
tests cover complete panels, reverse row order, duplicated/missing years, rank
failure and no-clobber export; the converter was checked against a modified
source and an existing destination.

## First-difference OLS

[`first_difference_ols.td`](first_difference_ols.td) reproduces the Table 22.2
no-intercept OLS column on the same 4,256 person-years and five differenced
regressors. It reports classical, HC1 and person-clustered covariances and exports
all design rows, fitted values and residuals. The log-wage coefficient is
0.1115114; its three SEs are 0.0230566, 0.0791673 and 0.0960926. Classical
uses RSS/(N-K), HC1 uses N/(N-K), and cluster uses
G/(G-1)*(N-1)/(N-K); the t reference has N-K, N-K and G-1 degrees of
freedom respectively. Person clustering allows within-person dependence, not
cross-person dependence. This is an associational OLS fit, not evidence of
exogeneity or a causal effect. It does not implement GMM or complete the table.

Run the base R reference above first, then in a fresh directory containing
`MOMprecise.csv`:

```sh
/path/to/tabdat-explore-rs --json -f /path/to/repo/examples/mma22p1panel/first_difference_ols.td > ols-result.jsonl
Rscript /path/to/repo/examples/mma22p1panel/ols_reference.R reference/design.csv ols-reference
python3 /path/to/repo/examples/mma22p1panel/check_ols.py /path/to/MOMprecise.dat ols-result.jsonl fd_ols_design.csv ols-reference
```

R 4.6.1 independently reconstructs the source design in `reference.R` and
uses QR OLS and independently formed covariance matrices. The checker compares
all 4,256 exported rows and predictions, coefficients, standard errors,
p-values and all three full covariance matrices at `1e-9 + 1e-9*abs(reference)`,
plus the 20 author coefficient/SE facts at three-decimal table precision. The
historical Stata log used float storage; longer printed digits are not claimed
as identical. JSON and text exports are byte-identical on the original data.
The same complete-panel, finite-data and no-clobber rules apply; prior session
changes are not rolled back. Focused CLI controls cover reversed input, missing
or duplicate years, singular design and an existing export.

## Base-instrument wage first-stage diagnostic

[`base_first_stage.td`](base_first_stage.td) reproduces the author program's
wage relevance diagnostics on the same 4,256 person-years: regress differenced
log wages on an intercept and nine base instruments, then jointly test the nine
slopes. Person-clustered F(9,531) is 2.379005 (p=0.012093); classical
F(9,4246) is 2.800243 (p=0.002811), with centered R²=0.00590049. Cluster
covariance uses G/(G-1)*(N-1)/(N-K) and requires independent people. The author
program explicitly notes that the classical F was mistakenly reported in the
book text in place of the clustered F.

The diagnostic's intercept follows the author program, unlike the no-intercept
2SLS fit above. Its clustered F is numerically close to, but **not** the same as,
the no-intercept 2SLS first-stage classical F (2.375352, F(9,4247)). These
individual-equation relevance tests do not resolve identification strength with
five endogenous regressors, establish instrument validity, or provide weak-IV
robust structural inference. They do not implement GMM or the stacked diagnostics.

After creating `MOMprecise.csv` and the independent base R `reference` directory
as above, run in a fresh directory:

```sh
/path/to/tabdat-explore-rs --json -f /path/to/repo/examples/mma22p1panel/base_first_stage.td > first-stage-result.jsonl
Rscript /path/to/repo/examples/mma22p1panel/first_stage_reference.R reference/design.csv first-stage-reference
python3 /path/to/repo/examples/mma22p1panel/check_first_stage.py /path/to/MOMprecise.dat first-stage-result.jsonl first_stage_design.csv first-stage-reference
```

R 4.6.1 reconstructs the source design separately and uses QR, independently
formed covariance and joint Wald calculations. The checker compares all 4,256
exported design/prediction rows, both sets of coefficients/SE/p-values and full
10×10 covariance matrices, R², F and p-values at `1e-9 + 1e-9*abs(reference)`;
it also checks the author-log R² and F values at `1e-5` to account for its float
storage. Original JSON/text exports are byte-identical. The complete-panel and
no-clobber rules above apply; the focused test exercises reversed input, missing
years, rank failure and existing export. Scripts are not transactional.

## Year-stacked instruments

[`stacked_2sls.td`](stacked_2sls.td) uses the same 4,256 observations and five
regressors, but puts the nine base instruments into separate year blocks for
1981–1988, giving 72 excluded instruments. Other year blocks are zero for each
observation. It reports classical and person-clustered covariance and exports
the full stacked design and predictions. The log-wage coefficient is 0.542827,
with classical SE 0.169135 and clustered SE 0.208522. The conventional wage
first-stage F is approximately 1.30; many instruments and multiple endogenous
regressors complicate interpretation, and this does not provide weak-IV robust
inference. The same complete-panel and no-clobber rules apply.

After creating `MOMprecise.csv` and the base R `reference` directory as above,
run in a fresh directory or after removing the earlier stacked export:

```sh
/path/to/tabdat-explore-rs --json -f /path/to/repo/examples/mma22p1panel/stacked_2sls.td > stacked_result.jsonl
Rscript /path/to/repo/examples/mma22p1panel/stacked_reference.R reference/design.csv stacked-reference
python3 /path/to/repo/examples/mma22p1panel/check_reference.py --stacked /path/to/MOMprecise.dat stacked_result.jsonl stacked_iv_design.csv stacked-reference
```

Independent R constructs each of the 72 instrument columns from the independently
prepared base design and uses QR projection for the two covariance estimators.
The check compares every design, fitted and residual value, coefficients,
p-values and both covariance matrices at `1e-9 + 1e-9*abs(reference)`, plus
the author coefficient and SE values at table precision. Focused tests cover
block placement, missing years, the two fits and no-clobber export. This adds
one stacked 2SLS column; it does not implement two-step GMM.
