# Cluster robust NB2 grouped goodness of fit

## Installed workflow

The existing `estat countgof [, max(10)]` command selects this calculation when
the retained NB2 fit is one-way clustered. It reports the group variable/count,
`G/(G-1)` covariance multiplier, full moment covariance and chi-square result in
JSON. The terminal report names the cluster method and assumptions. The
model-based unclustered command and its JSON fields remain unchanged.

## Design contract

This slice extends `estat countgof [, max(10)]` to an unweighted, clustered
NB2 ML fit without an offset. The existing unclustered Andrews auxiliary
regression and its output remain unchanged. Row robust fits, weights, offsets,
Poisson fits, and multiway clusters remain unsupported. The test reads the
retained NB2 sample and does not refit or mutate session state.

For a cutoff `K`, the grouped count table has `J=K+1` cells. Let `m_i` contain
the first `J-1` observed-minus-fitted cell probabilities, and let `s_i` contain
the full row NB2 score for the fitted mean coefficients and `lnalpha`. Define

```
A = N^-1 sum_i d s_i(theta) / d theta'
C = N^-1 sum_i d m_i(theta) / d theta'
h_i = m_i - C A^-1 s_i
H_c = sum_(i in cluster c) h_i
V_m = [G / (G - 1)] N^-1 sum_c H_c H_c'
M = N mbar' V_m^-1 mbar
```

The NB2 evaluator's positive observed information is `I=-N*A`, so the
implementation form is `h_i = m_i + C I^-1 s_i`. It must retain the complete
beta/`lnalpha` score and the beta-dispersion cross terms. `I` is model-based
observed information even when the fitted result stores a cluster sandwich.
The statistic uses the uncentered cluster meat, the existing NB2
`G/(G-1)` finite-sample multiplier, and an asymptotic chi-square reference with
`rank(V_m)` degrees of freedom. A singular or rank-deficient covariance is a
reported estimation failure; no pseudo-inverse or `G-1` t/F reference is used.

Cameron and Trivedi's §8.2.3 gives the independent-observation M-test influence
function and §8.2.6 gives the grouped count test and its `J-1` cell restriction.
The cluster sum above is an extension of that influence function to independent
clusters, not a cluster formula stated verbatim in the book. Its reference
requires independent clusters, a correctly specified conditional NB2 marginal
distribution, and many independent clusters. `G/(G-1)` only scales the
estimated covariance; it does not provide a few-cluster guarantee.

## Independent reference design

Use a deterministic Base R fixture with many independently drawn clusters. A
shared `U_c ~ Gamma(r, rate=r)` and conditionally independent
`Y_ic ~ Poisson(mu_ic U_c)`, `r=1/alpha`, induces within-cluster dependence
while retaining NB2 row marginals. Fit beta and `lnalpha` jointly from the NB2
likelihood, compute fitted cell probabilities, row scores, the full observed
information, `C`, cluster influences, covariance and test independently in R.
Check `C` separately against finite differences of the cell probabilities.

The synthetic fixture validates parameter estimates and every reported GOF
quantity. Contract checks cover same-sample use, cluster count and correction in
structured/terminal output, missing or misaligned group identifiers, row-robust
fits, weights and offsets, rank-deficient cluster meat, and state-preserving
failures.

## Independent validation

`cluster_count_gof_reference.R` is a Base R 4.6.2 reference generator. It writes
the synthetic input and expected values to
`crates/tabdat-stats/tests/fixtures/nb2_cluster_count_gof_data.csv` and
`nb2_cluster_count_gof_reference.csv`. The 480-row sample has 96 clusters and
uses tail cutoff 4. R fits beta and `lnalpha` jointly, calculates the full
observed information and score contributions, derives `C` from the NB2 cell
probability scores, and checks it against finite differences of `dnbinom`.
R then calculates the adjusted cluster moments, covariance and chi-square
reference independently.

The reference result is `M=3.72571408404506`, df=4,
`p=0.444397009701926`, with covariance multiplier `96/95`. Rust matches the
coefficients, alpha, log likelihood, every entry of the 4-by-4 moment
covariance, observed/fitted cell counts and test statistic. Runtime and CLI
checks verify retained-sample use, state preservation, rank-deficiency failure,
JSON metadata and terminal assumptions.

The source workflow `cluster_count_gof.td` uses the independently audited
20,186 person-year sample and 5,908 original person IDs from
`prepare_person.R`; no author data are committed. The separate
`cluster_count_gof_author_reference.R` reads this ID-enriched CSV, fits beta and
`lnalpha` jointly in Base R, checks the cell-probability derivative by finite
differences, and independently computes the person-cluster covariance. The
checker `check_cluster_count_gof_author_reference.py` compares its output with
the native workflow. Across149 quantities, maximum absolute difference is
`2.11003e-10` (log likelihood); `M=62.0783423878654`, df=10,
`p=1.46222048715161e-9`, with covariance multiplier `5908/5907`. This validates
the specified independent-person extension on the source rows. It does not
establish household/site dependence, few-cluster behavior, or a result printed
by the original author program.
