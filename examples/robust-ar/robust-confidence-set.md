# Robust scalar Anderson–Rubin confidence set

## Scope and contract

This is a bounded graduate-IV method extension, not a newly replicated
Cameron–Trivedi illustration. `estat ar, ci` accepts a robust or one-way
clustered 2SLS fit only when there is exactly one endogenous regressor and
exactly one excluded instrument. Included exogenous regressors remain
unrestricted nuisance parameters.

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

## Validation evidence

`robust_ar_ci_reference.R` uses base R 4.6.2, direct `lm.fit`, explicit HC1/CR1
score sandwiches and `polyroot`. It checks both 90% and 95% levels. Source rows
2 and 8 are excluded for missing outcome/instrument values; the clustered case
also excludes source row 17 for a missing group. The Rust tests compare retained
source indices, auxiliary K/DF, correction, critical value, endpoints, test
statistics and set membership. They also check instrument rescaling and all-real
and disjoint sets under weak relevance.

| Covariance | Level | N / groups | Correction | Critical χ²(1) | Confidence set |
| --- | ---: | ---: | ---: | ---: | --- |
| HC1 | 90% | 78 | 1.04 | 2.70554345409542 | [1.41099036473887, 1.91644856882503] |
| HC1 | 95% | 78 | 1.04 | 3.84145882069413 | [1.36445194119086, 1.96845342740648] |
| CR1 | 90% | 77 / 10 | 1.14114114114114 | 2.70554345409542 | [1.27525817391537, 2.05603819984647] |
| CR1 | 95% | 77 / 10 | 1.14114114114114 | 3.84145882069413 | [1.20537141708336, 2.13637942939771] |

Script/CLI tests verify JSON and terminal output, repeat inference after current
data changes, and check that unsupported multiple-instrument fits fail before
export. Statistics tests exercise invalid covariance/levels/designs, rank and
residual-DF failures, too few clusters and degenerate auxiliary covariance.
The robust result is additive; the existing homoskedastic confidence-set JSON
shape stays unchanged.
