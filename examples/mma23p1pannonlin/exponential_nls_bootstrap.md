# Chapter 23 exponential-mean NLS whole-firm bootstrap — WIP contract

## Bounded outcome

Reproduce the firm-pairs bootstrap method for the §23.3/Table 23.1 NLS
illustration in the [author program](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.do)
and [log](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.txt), not
add a general bootstrap or clustered `nl` command. The pinned
`patr7079.asc` SHA-256 is
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`.
Use the existing source preparation: all 346 source-order firms/1,730 rows in
1975–79, including 22 zero-patent firms and missing nonmodel ARDSSIC metadata.
The two-parameter unweighted mean is `mu_it=exp(a+b*LOGR_it)` for PAT; no
extra regressors, time effects, weights or offsets. The original native NLS fit
and predictions are already independently validated in [exponential_nls.md](exponential_nls.md).

For each of 500 replicates draw 346 whole firms with replacement and keep all
five rows per copied firm; refit both parameters by unweighted NLS. Seed 10001
uses the established per-replicate xoshiro256++/SplitMix64 stream and u64
rejection, not Stata's RNG or TabDat's native single stream. Report the fitted
original estimate, replicate means/biases, full B−1 covariance and SEs, and
R-type7 percentile 95% intervals; export ordered draw IDs/steps and coefficients.
No bootstrap p-values, BCa intervals, few-cluster guarantee, causal claim,
general nonlinear bootstrap API or restorable bootstrap model state is claimed.
Whole-firm resampling assumes independent firms. A failed replicate is an error,
not a silent discard or redraw.

The author log reports original intercept/slope `2.528729`/`.5084673` and
bootstrap intercept/slope SE `.198022`/`.0541264`. Because the explicit draws
differ, these historical bootstrap SEs and intervals are context, not exact
parity gates. The original-sample estimator, design, and independent resample
fits must still be validated at fixed documented numerical tolerances.

## Before implementation

NLS may have multiple stationary points; a profiled slope score is **not**
automatically monotone. Investigate original and all 500 proposed resamples
against independent R, specify a bounded local-solution/initialization rule,
positive observed-loss curvature and convergence checks, and document numerical
bounds before claiming a usable workflow. Do not rely on a bisection bracket
alone as proof of uniqueness, silently accept a boundary/stationary maximum, or
widen tolerances. If the bounded solver cannot independently validate every
replicate, keep this PR in draft and record the blocker.

Acceptance: (1) exact source/sample and all 173,000 draw IDs/steps against the
independent Python oracle; all 500 intercept/slope refits, full covariance and
summaries against an independently coded R reference and representative second
method; (2) small inspectable fixture with zero-total firms, original data and
all draws/refits, text/JSON and input-order checks; (3) invalid, duplicate,
missing, singular, bounded-nonconvergence and failed-replicate cases with
session/file/no-clobber invariants and no redraw; (4) explicit earlier native NLS
state when later SQL fails; (5) current SPEC/roadmap/ledger and proportionate
focused/affected tests. R/Python and source data remain development-only.

Poisson RE, GEE/RE bootstraps, general GEE, Figure 23.1 and the rest of the
empirical illustration inventory remain required.
