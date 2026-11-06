# Chapter 24 individual shared-gamma Poisson RE whole-commune bootstrap

This bounded workflow delivers the whole-commune pairs bootstrap of the
individual shared-gamma random-effects column of Cameron–Trivedi (2005) §24.7.
It loads and audits the source, retains the original native RE fit, refits 500
whole-commune copies including dispersion, reports joint uncertainty and exports
original-fit marginal predictions. It is **not** a complete table/chapter,
normal-RE or GEE bootstrap, complex survey or causal inference. Source hashes
and interpretation are in the [README](README.md); the [original RE
analysis](individual_gamma_re.md) defines likelihood, model-based inference,
convergence and prediction semantics.

## Sample and statistical contract

Outcome `PHARVIS`, ten numeric predictors in order `LNHHEXP INSURANCE SEX AGE
MARRIED ILLDAYS ACTDAYS INJURY ILLNESS EDUC`, plus intercept and ln(alpha).
`COMMUNE` is both the shared-effect group and the resampling unit; `source_row`
is not time. The 27,766-record source includes one wholly missing record. The
joint-finite sample is **27,765 people in 194 communes**, including all 94
all-zero observations in commune64, with sizes51–206 and total count14,209.
The pinned author bootstrap reports the same N and group count. Unlike
conditional FE, zero-total groups remain in the sampling frame and likelihood;
positive or zero singletons are also retained.

For each replicate, sample 194 original groups uniformly with replacement.
Each group copy contains all its people and has a **distinct shared gamma
effect**. Repeated copies must not be merged into one longer source-ID panel:
that is a different integrated likelihood. Each refit jointly estimates all
12 parameters, including ln(alpha); it does not reuse a fixed dispersion or
substitute row-level NB2. The local continuous xoshiro256++/SplitMix64 stream
uses first-seen complete-case groups and u64 rejection mapping, with at most
G+1024 RNG words per replication. This script explicitly orders by `source_row`;
changing physical row order in the general command can change group-index mapping.

Report the original joint vector, bootstrap means/bias, full B−1 covariance/SE
and type7 marginal percentile95 intervals. Terminal text and CLI-v1 JSON agree;
JSON includes all group labels/sizes, zero-based ordered draws, copied sample
Ns and replicate parameter vectors. It is a read-only report: original data,
labels, named reports and retained model stay unchanged on success or failure.
Fit `xtpoisson ..., re id(commune)` separately for marginal population
predictions; these are not posterior effects. Finite B has Monte Carlo error;
there are no bootstrap analytic p-values, BCa/studentized intervals or few-group
coverage guarantees. No weights, offsets, noconstant, survey or causal inference
is provided.

The author program uses the older `bootstrap ..., cluster(COMMUNE)` form without
visibly specifying `idcluster()`. The historic treatment of repeated IDs is
unresolved, as in the [Chapter23 gamma bootstrap](../mma23p1pannonlin/gamma_re_bootstrap.md);
this implementation explicitly uses distinct effects and claims no historical
RNG, bootstrap uncertainty or repeated-ID parity. On the synthetic replicate1,
an independently optimized R counterfactual merging source IDs changed the
largest parameter by **0.386107** versus distinct copies. This illustrates the
importance of the convention, not what the historical software did.

Bounds: raw keys must be valid even on omitted rows; exact selected counts and
group totals 0..100000; 1..14 unique slopes, intercept and ln(alpha), B2..500,
G<=256 with G>slopes+2, G*largest_group_size<=40000 and
G*largest_group_size*B*(slopes+2)²<=3000000000 with checked arithmetic.
Original RE candidate-row, rank, scaling, dispersion, information and
convergence guards apply to every fit. ln(alpha) must be strictly inside
(−12,12); no pooled boundary fallback is silently substituted. Every failed
original or replicate fit aborts without redraw or partial result; the bounded
optimizer is not a general existence/separation test. Exact group labels use
internal key strings, e.g. DECIMAL `9007199254740993.1` becomes
`90071992547409931e-1` in JSON. Scripts are nontransactional: export collisions
preserve owner/source/current data and retained fit but do not undo earlier
script effects.

## Execute and independently audit

A development build with the numerical crate optimized makes the 500-refit
workflow practical without adding a production R/Python dependency:

```sh
cargo build --locked --bin tabdat-explore-rs --config 'profile.dev.package.tabdat-stats.opt-level=3'
repo=/absolute/path/to/tabdat-explore-rs
work=/absolute/path/to/empty/work-directory
python3 "$repo/examples/mma24p2poiscluster/prepare.py" /path/to/vietnam_ex2.asc "$work/vietnam_individual.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma24p2poiscluster/individual_gamma_re_bootstrap.td" > result.jsonl)
python3 "$repo/examples/mma24p2poiscluster/check_individual_gamma_re_bootstrap.py" oracle "$work" /path/to/vietnam_ex2.asc "$work/draws.csv"
Rscript "$repo/examples/mma24p2poiscluster/individual_gamma_re_bootstrap_reference.R" /path/to/vietnam_ex2.asc "$work/draws.csv" "$work/reference"
python3 "$repo/examples/mma24p2poiscluster/check_individual_gamma_re_bootstrap.py" check "$work" "$work/reference" /path/to/vietnam_ex2.asc
```

Python independently verifies **all 97,000 ordered group draws**, labels,
sizes and copied Ns. Independent base R 4.6.2 fits the integrated likelihood
using BFGS plus direct special-function score/information refinement, weighting
each complete original group's contribution by its draw frequency. This is
equivalent to distinct copies, not a merged shared effect. R and native check
all 500 twelve-parameter refits, full 12×12 covariance, summaries, every
27,766 source record and every 27,765 prediction record. Fixed absolute
1e-9 tolerance is used; indices and flags are exact. Maximum native/R absolute
refit/summary/prediction discrepancies: ASCII **1.19e-12 / 1.47e-13 /
7.43e-13**, precise DTA **1.19e-12 / 3.25e-13 / 1.17e-12**. Text and JSON
prediction CSVs are byte-identical, and all twelve displayed summary rows
match numerically on both sources.

An inspectable unbalanced synthetic sample has 279 source rows, 278 complete
people and 48 groups, including four zero-total groups and two singletons. All
24,000 draws, 500 R refits, covariance and predictions are audited; maximum
native/R differences are **9.71e-11 / 1.75e-12 / 2.66e-15**. Independent R
likelihood fits that expand actual rows and assign distinct IDs to copies
validate replicates1,250,500, including repeated groups, zero-total groups and
singletons, within **5.45e-15** of multiplicity refits. Twenty-five full-auditor
and one author-point forgeries were rejected. The fixture and reproduction
commands are in `tests/fixtures/mma24_individual_gamma_re_bootstrap/README.md`.

The pinned author program/log use B50 despite a comment mentioning B500. On
precise DTA, **10 of 12** printed bootstrap original-point facts match both R
and native to half the last printed digit. Two final-digit exceptions are
explicit: `INSURANCE` is **−0.16759534936** in R/native versus **−.1675954**
in the author's bootstrap, while the author's direct RE fit prints **−.1675953**;
`ILLNESS` is **0.58018724159** versus **.5801873**, while the direct fit prints
**.5801872**. The causes of differing author final digits remain unverified; no
tolerance was widened and historical B50 bootstrap uncertainty was not
compared. To reproduce the separate DTA and printed-author audit in a fresh
work directory:

```sh
Rscript "$repo/examples/mma24p2poiscluster/dta_reference.R" /path/to/vietnam_ex2.dta "$work/vietnam_individual.csv" --precise
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma24p2poiscluster/individual_gamma_re_bootstrap.td" > result.jsonl)
python3 "$repo/examples/mma24p2poiscluster/check_individual_gamma_re_bootstrap.py" oracle "$work" /path/to/vietnam_ex2.dta "$work/draws.csv" --dta
Rscript "$repo/examples/mma24p2poiscluster/individual_gamma_re_bootstrap_reference.R" /path/to/vietnam_ex2.dta "$work/draws.csv" "$work/reference" --dta
python3 "$repo/examples/mma24p2poiscluster/check_individual_gamma_re_bootstrap.py" check "$work" "$work/reference" /path/to/vietnam_ex2.dta --dta
python3 "$repo/examples/mma24p2poiscluster/check_individual_gamma_re_bootstrap_author.py" /path/to/vietnam_ex2.dta /path/to/mma24p2poiscluster.txt "$work/reference" "$work/result.jsonl"
```

The optional `--precise` conversion uses 17 digits to round-trip directly read
DTA binary64 values; the historical default remains unchanged and can round.
On the local M3 Max, optimized-stats JSON runs took **64.11 s / 162 MB peak RSS**
for ASCII and **63.98 s / 163 MB** for DTA; these are observations, not runtime
or platform guarantees. Ordinary debug can be much slower. R/Python and
publisher data are development inputs only.

Other GEE workflows, genuine survey design, remaining household RE outputs and
other textbook illustrations remain required; this slice does not complete the
chapter or roadmap.
