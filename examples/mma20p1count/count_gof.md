# NB2 grouped count goodness of fit

This bounded diagnostic follows the additional NB2 analysis in Cameron and
Trivedi, *Microeconometrics: Methods and Applications* (2005), §20.7, p. 690,
and the November 2006 `mma20p1count.do` program. It uses the NB2 fit's retained
complete finite estimation rows, fitted means and individual maximum-likelihood
scores; it does not refit or change data/model state.

For tail cutoff `K`, the cells are `0, …, K-1, >=K`. The command reports the
observed count, mean conditional fitted probability, expected count and
difference for every cell. With `J=K+1` cells, it drops the final cell and
computes the Andrews chi-square goodness-of-fit statistic as `N * R²_u` from
the no-intercept auxiliary regression of one on the first `J-1` cell residuals
`1[y_i in cell j] - P(Y_i in cell j | X_i)` and all NB2 likelihood score
columns, including `lnalpha`. The reference is asymptotic `chi-square(J-1)`;
the default `K=10` therefore has 11 cells and 10 degrees of freedom.

The author program computes the same auxiliary statistic but passes `K-1=9`
degrees of freedom to its tail calculation. The book's §8.2.6 states that one
of `J` cells is dropped, giving `J-1` degrees of freedom, so this workflow will
use 10 for the default 11-cell partition and preserve the author-code mismatch
as an explicit source discrepancy.

The reference law assumes independent observations and a correctly specified
NB2 conditional distribution. It does not provide a cluster-robust goodness-of-
fit reference. The source doctor-visit data contain repeated person-years and
the author program itself notes that clustering is needed; source-sample output
is therefore reported as method reproduction, not dependable panel inference.

## Verification

The Rust implementation will be checked against a separate Base R calculation
of the auxiliary regression on independent synthetic NB2 samples and the exact
prepared MMA sample. Runtime coverage will check retained-row use, text/JSON
results, unsupported fit families, clustered-fit rejection and state-preserving
failure behavior. The author data remain external and are not redistributed.
