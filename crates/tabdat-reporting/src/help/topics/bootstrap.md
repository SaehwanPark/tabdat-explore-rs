# bootstrap

`bootstrap y xvars, seed(integer) [reps(200) noconstant]`

Compute a read-only IID pairs-bootstrap OLS report on CURRENT complete-finite
outcome/predictor rows, with an intercept unless noconstant is specified. At
least one slope is required. The existing OLS sample/type rules apply; there are
no weights, robust/cluster modes, factors or if clauses on this bounded command.
Do not pairs-resample person-years when independent units are people. For a
Between regression, first form the audited person means and resample those rows.

The explicit unsigned64-bit seed is REQUIRED; a script's context-only seed does
not initialize this command. Replications are integers2..5000 (default200), with
N*replications*K² <=100,000,000, K including the intercept. Original OLS must be
identified with N>K. Each draw samples N whole (y,X) records with replacement.
Any replicate rank/numerical failure aborts with its index: no discarded draws,
redraws or conditional-on-success uncertainty. Full-rank exact fits are allowed.

RNG contract: xoshiro256++ initialized from the seed via SplitMix64, pinned
rand_xoshiro0.7.0; each index uses next_u64(), rejecting values >=
(u64::MAX-u64::MAX%N), then taking remainder N. The local stream restarts on each
command. There is no global stream consumption or cryptographic use. Identical
seed/sample order/design/replications give the same draws under this versioned
contract and stable physical sample order; changing row order changes the sample-index mapping.
The canonical backend enables preserve_insertion_order; this command uses a
plain SELECT projection of the active materialized relation, with no joins/groups. Seed is decimal
text in CLI-v1 JSON, preserving exact identity beyond IEEE754's integer range.

Report original coefficients, replicate means, bias (mean-original), bootstrap
SE/full covariance with B-1 denominator (possibly singular) and fixed95% percentile intervals using
R type7 linear interpolation. There are no analytic normal/t p-values or BCa,
basic/studentized intervals. Finite B has Monte Carlo error; percentile intervals
are not a finite-sample or coverage guarantee. This is not residual/parametric,
cluster, weighted/survey or block resampling, nor generic command-prefix syntax.

Data/order/labels/current fit/named reports stay unchanged on success or failure.
The bootstrap report is not installed as a fit or covariance for predict/test.
Fit `regress` explicitly if you need predictions or analytic post-estimation.
JSON returns BootstrapOlsResult; terminal/text displays the same report.

See examples/mma21p1panel/README.md for the seeded Between person-mean workflow.
