# bootstrap

`bootstrap y xvars, seed(integer) [reps(200) noconstant cluster(unit)]`

Compute a read-only pairs-bootstrap OLS report on CURRENT complete-finite
outcome/predictor rows, with an intercept unless noconstant is specified. At
least one slope is required. The existing OLS sample/type rules apply; there are
no weights, analytic robust covariance modes, factors or if clauses on this bounded command.
Without cluster(unit), do not pairs-resample person-years when independent units are people. For a
Between regression, first form the audited person means and resample those rows.

The explicit unsigned64-bit seed is REQUIRED; a script's context-only seed does
not initialize this command. Replications are integers2..5000 (default200), with
N*replications*K² <=100,000,000 for IID mode, K including the intercept. Original OLS must be
identified with N>K. Without cluster(unit), each draw samples N whole (y,X) records with replacement.
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
weighted/survey, wild or time-block resampling, nor generic command-prefix syntax.

Data/order/labels/current fit/named reports stay unchanged on success or failure.
The bootstrap report is not installed as a fit or covariance for predict/test.
Fit `regress` explicitly if you need predictions or analytic post-estimation.
JSON returns BootstrapOlsResult; terminal/text displays the same report.

See examples/mma21p1panel/README.md for the seeded Between person-mean workflow.

With cluster(unit), first select joint complete-finite outcomes/predictors and
nonmissing keys; floating NaN/infinite keys are excluded. Numeric/text/boolean
keys are supported; wide integer and decimal identities are exact, floating
signed zeros share a group. Other key types fail. Group order is first appearance
in the retained sample, with original within-group row order. At least two
groups are required; no few-cluster accuracy guarantee is supplied.

Each replication draws G groups uniformly with replacement and appends all
member (y,X) rows for each selected group, including duplicated blocks. RNG
rejection uses G in place of N. Unbalanced group sizes imply variable replicate
N, reported for every replication. The work cap uses
G*largest_group_size*replications*K²<=100,000,000 before fitting.
ClusterBootstrapOlsResult adds first-seen cluster_sizes/count/order and replicate
observation counts to the common report; IID BootstrapOlsResult is unchanged.

Supplied transformations are fixed: this is OLS on current columns, not an
automatic within/RE fit or re-estimation of variance components. For the bounded
manual-program within workflow, joint-case person means are computed first and
whole persons are resampled afterward. No retained model or analytic test
covariance is installed.
