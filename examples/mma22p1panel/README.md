# WIP: Chapter 22 first-difference panel IV

Target the base-instrument 2SLS column of Cameron and Trivedi §22.3,
Table 22.2, using the authors' `MOMprecise.dat` and `mma22p1pangmm.do`. The
precise data are distinct from Chapter 21's two-decimal `MOM.dat`. The author
program treats all five differenced regressors as instrumented, despite an
earlier comment that describes only wage as endogenous. Reproduce the actual
command: no intercept, five regressors, nine base instruments, 1981–1988,
4,256 person-years and 532 people. Preserve individual/time keys while forming
lags; reject gaps and duplicates rather than borrowing another person's rows.

The intended slice uses the existing multiple-endogenous `ivregress 2sls`
command with classical and person-clustered covariance. Independently compute
sample membership, 2SLS coefficients and covariance in R and compare the author
output at its printed precision. Include duplicate keys, missing years and
rank-deficient instruments as failure cases. Two-step GMM, stacked instruments,
serial-correlation diagnostics and general dynamic-panel commands are out of
scope for this first base 2SLS column.

This is an implementation contract, not a claim of current executable coverage.
