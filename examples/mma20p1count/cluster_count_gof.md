# Cluster robust NB2 grouped goodness of fit

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

## Independent reference plan

Use a deterministic Base R fixture with many independently drawn clusters. A
shared `U_c ~ Gamma(r, rate=r)` and conditionally independent
`Y_ic ~ Poisson(mu_ic U_c)`, `r=1/alpha`, induces within-cluster dependence
while retaining NB2 row marginals. Fit beta and `lnalpha` jointly from the NB2
likelihood, compute fitted cell probabilities, row scores, the full observed
information, `C`, cluster influences, covariance and test independently in R.
Check `C` separately against finite differences of the cell probabilities.

The fixture should validate parameter estimates and each reported GOF quantity.
Contract checks should cover same-sample use, cluster count and correction in
structured/terminal output, missing or misaligned group identifiers, robust
row fits, weights and offsets, too few groups, rank-deficient cluster meat, and
state-preserving failures. This synthetic validation does not establish
household/site or source-panel inference.
