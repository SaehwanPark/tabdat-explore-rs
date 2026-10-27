# WIP: nearest-neighbor matching

This draft slice is not ready for release or merge. It fits the Chapter 25 DW99
assignment score, selects the inclusive empirical common score range, matches
retained treated people to all exactly equidistant nearest controls, averages
exact ties, and reports weighted control reuse and a descriptive point contrast.
Its normal uncertainty field is deliberately null: no valid estimated-score
matching variance or confidence interval has yet been implemented.

The original 2,675-person source replay and independent base-R brute-force
reference agree on all nearest pairs. The empirical common range retains 179
treated people; 57 distinct controls are used, 3 treated people have exact ties,
and the descriptive contrast is 515.906. This is **not** a reproduction of the
author program's random tie handling or its historical support rule; the author's
logged DW02 and DW99 variants also use different score specifications. The 13
predictors, source SHA256 and unverified observational assumptions are documented
in [README.md](README.md). No causal identification follows from this point value.

Inference needs an explicit statistical decision before this draft can be marked
ready. Abadie and Imbens (2016), [Matching on the Estimated Propensity
Score](https://doi.org/10.3982/ECTA11293), show that the first-stage adjustment
for ATT can increase or decrease variance; treating estimated scores as known is
not guaranteed conservative. Their proposed estimators require covariate-space
and propensity-space local matching for conditional variances, covariances and
the derivative of the ATT target. Ordinary bootstrap inference for a fixed
number of nearest neighbors is not generally valid (Abadie and Imbens, 2008,
[On the Failure of the Bootstrap for Matching Estimators](https://doi.org/10.3982/ECTA6474)).

Their continuous-score regularity conditions and fixed-number matching do not
straightforwardly establish inference for this draft's empirical min/max support
selection and exact-tie averaging. The source sample has 351 people sharing an
exact fitted score with another person and 271 people with zero earnings in both
pretreatment years. This is evidence that ties and possible score atoms must be
studied; it does not by itself prove the population distribution has atoms.
The largest selected treated-to-control score gap is about 0.087 and the maximum
fractional control reuse is 27, so even a theoretically appropriate asymptotic
approximation needs careful support diagnostics. Do not label a conditional or
known-score standard error as full ATT uncertainty.

Next: derive a tie-aware method with an explicit population target and support
rule, including the estimated-score adjustment and treatment of data-dependent
selection; establish assumptions under mixed/discrete scores; independently
validate all variance components against a trusted numerical implementation;
then add appropriate failure and state tests. If those conditions cannot be
justified for the original sample, retain the point-only result as explicitly
partial and choose a separately justified analysis rather than inventing an SE.
