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

## Feasibility probe for full-pipeline subsampling

A development-only probe draws 20 simple random subsamples without replacement
at each of sizes 700, 1,000 and 1,400, refits the logit and repeats empirical
range selection and matching. It keeps every success and failure in its JSON
output; it does **not** compute a standard error, confidence interval or coverage
estimate. These few draws are a numerical feasibility check, not a justification
of subsampling or its asymptotic target. Run it with the pinned original source:

```sh
python3 /path/to/repo/examples/mma25p1treatment/nearest_subsample_probe.py /path/to/nswpsid.da1 /path/to/tabdat-explore-rs --verify-r > probe.json
```

With seed 3982, the initial strict-open-score version failed five of the 20
size-700 fits at the score guard. Separate inspection showed one to 18 scores
rounded to zero in each failure, all outside that replicate's empirical common
range. The draft now admits finite scores rounded to zero or one **only when
they are outside the selected range**; it rejects any boundary score inside the
range, and reports the total rounded and excluded counts. No saturated score is
used in a distance, no value is clipped, and this does not assert statistical
positivity. For the five affected samples, independent R also emitted numerical
boundary-score warnings and agreed on every selected nearest pair and outcome;
the retained treated counts were 18, 43, 35, 42 and 10. The full source sample
has no rounded boundary scores and its source-reference audit is unchanged.
After the change all 20 draws at each tested size completed; among the size-700
draws, the selected treated count ranged from 10 to 53. With `--verify-r`, the
probe reruns an independent R fit for every successful sample containing rounded
boundary scores, checks every retained pair, and records all R warnings. This
replay used Python 3.15.0 and R 4.6.1. Completion of these 60 fits is not a
proof that other subsamples will fit or that the sampling limit is valid.

This adjustment solves only a representational rejection of rows already
outside the comparison. It does not validate the fitted assignment model or an
inference procedure; several tiny retained samples and numerical warnings are
additional reasons to avoid presenting a generic resampling interval as reliable.
