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

## Controlled subsampling pilot

[`nearest_subsampling_simulation.R`](nearest_subsampling_simulation.R) is a
**development-only** check of a candidate without-replacement subsampling
interval; it does not add inference to the workflow. The toy population has
`X` uniform on [-1,1], with either no atom or a 40% atom at zero;
`T|X ~ Bernoulli(logit^-1(-0.4+0.9X))`; and
`Y=1.5T+2X+0.5X^2+epsilon`, with independent standard-normal errors. The true
treatment effect is 1.5 at every X, including after support selection. The
score model is correctly specified and true scores lie well inside (0,1).
This is a favorable, one-covariate illustration, not the NSW data-generating
process. Each full and subsample fit re-estimates the logit, selects the empirical
common score range and averages exact nearest ties. A separate scalar brute-force
calculation checks the matching implementation on a small tied fixture.

For each interval the pilot takes 100 without-replacement subsamples, each of
size `m=floor(n^a)`, and uses type-1 empirical 2.5% and 97.5% quantiles of
`sqrt(m)*(estimate_m-estimate_n)` scaled by `1/sqrt(n)`. With R 4.6.1, seed
356011, 200 independently generated full samples per setting, the observed
nominal-95% coverage was:

| Atom probability | n | m | Completed | Coverage | Monte Carlo SE |
| --- | ---: | ---: | ---: | ---: | ---: |
| 0 | 300 | 40 | 200 | 0.835 | 0.026 |
| 0 | 300 | 95 | 200 | 0.840 | 0.026 |
| 0 | 600 | 63 | 200 | 0.880 | 0.023 |
| 0 | 600 | 166 | 200 | 0.900 | 0.021 |
| 0.4 | 300 | 40 | 199 | unavailable | unavailable |
| 0.4 | 300 | 95 | 200 | 0.885 | 0.023 |
| 0.4 | 600 | 63 | 200 | 0.890 | 0.022 |
| 0.4 | 600 | 166 | 200 | 0.835 | 0.026 |

The incomplete setting had one subsample warning that its fitted probabilities
were numerically zero or one; no coverage is reported after that failure. The
pilot also prints an exploratory division by `sqrt(1-m/n)`, a finite-population
factor exact for the subsampled **sample mean**. It is not established for this
non-smooth matching statistic and is not applied in the product. This limited
pilot suggests poor finite-sample calibration in these settings; Monte Carlo
uncertainty and the small number of inner draws remain material. It does not
prove subsampling invalid in general or determine coverage for NSW. General
subsampling results require an appropriate known rate and limiting law; see
[Politis, Romano and Wolf (2001)](https://www3.stat.sinica.edu.tw/statistica/j11n4/j11n49/j11n49.htm).
Neither those conditions nor a correct finite-sample adjustment are established
for the current source workflow.

Reproduce the pilot with:

```sh
Rscript /path/to/repo/examples/mma25p1treatment/nearest_subsampling_simulation.R 200 100
```

The draft must continue to withhold an interval until the selected estimator's
population target, mixed-score and support-selection asymptotics, numerical
failure policy, and independent inference validation are resolved. The pilot
is risk-focused research, not a release or coverage gate.

## Untrimmed continuous-score variance prototype

[`nearest_variance_prototype.R`](nearest_variance_prototype.R) is a separate,
base-R implementation of the proposed ATET variance components in section 4
of Abadie and Imbens (2016). It is a development calculation, not the NSW
workflow or production inference. It fits a logit using `x1`, `x2` and an
intercept, matches each treated person to one control with replacement, and
uses the full sample without empirical-range selection. It rejects duplicate
scores, distance ties at a selection boundary, noninterior scores and invalid
input; these checks cannot establish the paper's population assumptions.

The calculation uses two same-arm observations including the focal person for
local variances and two same- or opposite-arm observations for conditional
covariances. It uses one same-arm neighbor excluding the focal person and one
opposite-arm neighbor for the propensity-space conditional means. For the
covariate-space matching used in the target derivative, distance is Euclidean
after dividing `x1` and `x2` by their sample standard deviations. The script
prints the known-score variance, the estimated-score subtraction and target
adjustment separately. A smoke check exercises permutation invariance, finite
components and rejection of duplicate scores and invalid data:

```sh
python3 /path/to/repo/examples/mma25p1treatment/check_nearest_variance_prototype.py
```

As an external comparison, the University of Wisconsin's [propensity-score
matching example](https://ssc.wisc.edu/sscc/pubs/stata_psmatch.htm) publishes
both [its input data](https://ssc.wisc.edu/sscc/pubs/files/psm.dta) and Stata
`teffects psmatch (y) (t x1 x2), atet` output. The data file used here has
SHA256 `5e02c85e9f0fb56afded6b8696bd7b46644e740a785e41c2e352e1971f97d4c1`.
Converting that file to CSV with pandas `read_stata` and running this prototype
with R 4.6.1 gives effect 0.9603507171 and standard error 0.1203081854. The
published Stata output gives effect 0.9603507 and AI robust standard error
0.1204748. The effect agrees at the published precision, but **the standard
errors do not**. The [Stata manual](https://www.stata.com/manuals/causalteffectspsmatch.pdf)
says its default robust calculation uses two matches to estimate standard
errors; the paper permits different local-neighbor choices. The source of the
difference has not been established and must not be explained away by widening
a tolerance. Matching this independent result or independently justifying a
different numerical reference remains necessary before any inferred interval
can be released. Neither result establishes validity for NSW's ties and
empirical support selection.
