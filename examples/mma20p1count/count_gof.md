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

For an unweighted, one-way clustered NB2 fit without an offset, the same command
uses the cluster-summed M-test influence function documented in
[`cluster_count_gof.md`](cluster_count_gof.md). It retains the joint beta/lnalpha
score, uses the model-based observed-information bread, applies `G/(G-1)` to the
uncentered cluster moment covariance, and reports an asymptotic chi-square test
only when that covariance is full rank. Its `J-1` degrees of freedom are
therefore available only when all first-cell moments are identified. The
reference assumes independent clusters, a correctly specified conditional NB2
marginal distribution and many independent clusters. The multiplier is not a
few-cluster correction. Row-robust fits, weights, offsets and multiway clusters
remain unsupported.

The independent-row reference assumes independent observations and a correctly
specified NB2 conditional distribution. The source doctor-visit data contain
repeated person-years and the author program itself notes that clustering is
needed. The prepared author ASCII omits person identifiers, so its exact-sample
result remains method reproduction rather than panel-robust inference.

## Verification

An independent Base R/MASS reference (`R 4.6.2`, `MASS 7.3.66`) fits the synthetic
fixture and exact prepared author sample, polishes the joint NB2 likelihood
scores, and computes probabilities, likelihood scores and the auxiliary
regression independently. The reference validates the exact prepared ASCII by
SHA-256 `c8b1eee8ae1f637f4d4e4c6aa6b2dcbf2b8aa98f276b9369adcb4f02a528c58a`; no
raw author data are committed.

On the exact `N=20,186` sample the independent reference gives
`CGF=123.15399170935741`, `df=10`, and `p=1.1576503947859055e-21`. Rust gives
`123.15399170936661`, `df=10`, and `1.157650394780931e-21`. Across 47 compared
quantities (statistic, df, p-value, and observed count/fraction, fitted fraction
and expected count for each of 11 cells), maximum absolute error is `9.60654e-12`. The
November 2006 author program prints `CGF=123.15368` and uses `df=9`; its
statistic differs from the independently polished reference by about
`0.000312`. The cause of the statistic difference is unresolved. The separate
df discrepancy is documented above and the book's `J-1` rule is used.

The independently polished fit also matches Rust across 20 retained quantities
(18 coefficients, alpha and log likelihood), with maximum absolute error
`2.11003e-10`. Base R's maximum absolute likelihood score is `2.54e-11`; the
Rust fit reports scaled score `7.98e-15` and scaled Newton step `2.19e-14`.

For the clustered extension, `cluster_count_gof_reference.R` creates a
deterministic Base R fixture with 480 rows in 96 independent clusters. A shared
Gamma effect makes the row outcomes dependent within each cluster while
preserving their NB2 marginals. The independent R fit, analytic observed
information, score adjustment, cluster covariance and chi-square result match
Rust; R also checks the cell-probability derivative against finite differences.
The reported test is `3.72571408404506`, df=4, p=`0.444397009701926` for the
synthetic fixture cutoff 4. The original person key is added only after
`prepare_person.R` audits the exact ASCII order against the external DTA. On
that retained `N=20,186`, `G=5,908` source sample, an independent Base R
reference and the native `cluster_count_gof.td` workflow match 149 quantities;
the result is `M=62.0783423878654`, df=10,
`p=1.46222048715161e-9`. The author program does not print a clustered GOF
statistic; this remains a native method extension, not author-output parity.

Runtime tests cover retained-row use, read-only behavior after current-data
edits, clustered covariance and rank failures, unsupported row-robust/offset
fits, and state-preserving failures. Language, JSON CLI and terminal output
contracts cover the independent-row and clustered reports. The fixed QR work
bound prevents oversized independent-row auxiliary regressions.
