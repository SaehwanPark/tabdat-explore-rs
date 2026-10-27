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
is a bounded source workflow, not a general panel-GMM command. Stacked
instruments, two-step GMM and dynamic-panel methods remain unimplemented here.

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
