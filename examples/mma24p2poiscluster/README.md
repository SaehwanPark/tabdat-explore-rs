# Chapter 24 individual Poisson cluster comparison — in progress

This bounded slice targets Cameron–Trivedi (2005) §24.7/Table 24.6 pooled
Poisson columns 1–4, from
[`mma24p2poiscluster.do`](https://cameron.econ.ucdavis.edu/mmabook/mma24p2poiscluster.do)
(SHA256 `746da2eca6d5970a045c137f106cfe33514e2f6e9595f26cb5451b3eb4bd2376`).
It fits an intercept and `LNHHEXP INSURANCE SEX AGE MARRIED ILLDAYS ACTDAYS
INJURY ILLNESS EDUC` to the individual `PHARVIS` count; model-based,
independent-row robust and commune-clustered Poisson covariance are separate.
It is unweighted pooled Poisson, not a complex-survey design, causal model,
fixed/random-effects model, bootstrap or complete Table 24.6 replication.

The publisher [`vietnam_ex2.asc`](https://cameron.econ.ucdavis.edu/mmabook/vietnam_ex2.asc)
SHA256 `e876f8ee17791881c6378de2761436e1e72e1eda84347089a9dc365ed290c85d`
contains 27,766 twelve-field records, including one entirely missing record
(source row 27,734). The author's sample has 27,765 individuals in 194 communes.
The original [`vietnam_ex2.dta`](https://cameron.econ.ucdavis.edu/mmabook/vietnam_ex2.dta)
SHA256 `284a5dd732926fe6ee02157012b17d8751523b89f1dd2f652a6f4dd021f3dac3`
retains more binary precision; source notes that Table 24.6 differs from the
program output despite the sample size. Do not imply exact book-table parity.

## Acceptance contract

- Prepare the wrapped ASCII in original order, preserving zero counts, all 12
  source fields and the all-missing row; reject changed source and output
  collisions. State the exact complete finite sample, numeric design, cluster
  correction, normal inference and retained-model/prediction semantics.
- Compare coefficients, full covariance matrices, tests, effects and predictions
  under all three covariance conventions against independent R GLM/score/delta
  formulas, and audit original source/cohort rows. Investigate deviations using
  separate author-DTA and printed-log evidence, never widened tolerances.
- Test a small nontrivial independent numerical fixture, invalid counts and
  cluster keys, rank/convergence and one-cluster failures, missing rows, state,
  file collisions and text/JSON parity. Distinguish nontransactional effects.

**Status:** contract only; no executable or validated workflow yet.
