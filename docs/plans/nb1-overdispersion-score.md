# NB1-direction Poisson score test

## Scope

Extend `estat overdispersion` with an explicit NB1 variance direction while
preserving the current default NB2 test and its result contract. NB1 here means
the regression-based alternative `Var(Y|X) = mu + alpha*mu`, tested at
`alpha = 0` from a retained Poisson fit. This does not add NB1 maximum
likelihood estimation or claim that the conditional count distribution is
negative binomial.

For each retained row, use the one-sided score
`s_i = ((y_i - mu_i)^2 - y_i) / (2*mu_i)`. The model-based null information is
`N/2`. Row-robust variance uses `N/(N-1) * sum(s_i^2)`; cluster-robust variance
aggregates scores by retained cluster and uses `G/(G-1) * sum_g(s_g^2)`. The
score is tested against the upper tail of the standard normal. Fixed offsets
remain part of the retained means.

## Verification plan

- Validate the NB1 score, model information, row sandwich and clustered
  sandwich against independently calculated Base R quantities.
- Cover default NB2 compatibility, all covariance modes, retained-fit behavior,
  malformed/nonpositive means, group alignment, and failure-state preservation.
- Update command help, command-result documentation, the count coverage ledger,
  and the roadmap with the exact support boundary.
- Run focused stats/runtime contract tests, affected crate tests, Clippy, format
  check and diff check.

## Method source

Cameron and Trivedi, “Regression-Based Tests for Overdispersion in the Poisson
Model,” *Journal of Econometrics* 46(3), 1990, pp. 347–364. The paper's
regression-based tests specify the conditional mean and variance under the
alternative without requiring a fully specified alternative distribution.
See the authors' [paper page](https://faculty.econ.ucdavis.edu/faculty/cameron/research/je90.html).
