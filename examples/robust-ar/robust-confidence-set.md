# Robust scalar Anderson–Rubin confidence set

## Scope and contract

This is a bounded graduate-IV method extension, not a newly replicated
Cameron–Trivedi illustration. The planned workflow extends `estat ar, ci` to a
robust or one-way clustered 2SLS fit only when there is exactly one endogenous
regressor and exactly one excluded instrument. Included exogenous regressors
remain unrestricted nuisance parameters.

For each candidate coefficient, the auxiliary regression uses the original
retained sample and fits `y - beta*x` on the included exogenous regressors and
the excluded instrument. The null tests whether the single excluded-instrument
coefficient is zero. HC1 uses `N/(N-k)`; one-way CR1 uses
`G/(G-1)*(N-1)/(N-k)`, where `k` is the auxiliary parameter count. The
reference is asymptotic chi-square(1). Cluster inference assumes independent
groups and many clusters; there is no few-cluster guarantee.

The confidence set is the non-rejection set of this robust AR test. With one
excluded instrument, its test inversion is a quadratic inequality and can be
solved without a search grid. Report the full set, including bounded, disjoint,
ray, all-real, empty or singleton shapes when they occur. Undefined auxiliary
variance, rank, residual degrees of freedom, cluster count or numerical range
must return an error without changing the fit or data.

Multiple excluded instruments, multiple endogenous regressors, robust
projection, HAC/survey covariance and few-cluster inference are outside this
slice. The homoskedastic confidence-set JSON shape remains unchanged.

## Acceptance evidence

- Compare sample membership, the auxiliary test statistic, its covariance
  correction, critical value and every finite endpoint with an independent
  base R reference using direct `lm.fit` and sandwich calculations.
- Check set membership against the robust AR test at interior, exterior and
  endpoint values for both HC1 and clustered covariance.
- Exercise degenerate variance, rank, degrees-of-freedom and cluster failures,
  plus retained-state behavior through the actual script/CLI path.

This file records the implementation target for its draft PR; it is not evidence
that the capability is installed or validated.
