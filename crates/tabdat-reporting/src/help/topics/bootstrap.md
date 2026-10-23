# bootstrap

`bootstrap y xvars, seed(integer) [reps(200) noconstant] [cluster(unit) | quantiles(values) [contrast(variable)]]`

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

## Joint conditional quantiles

`bootstrap y xvars, quantiles(.25 .5 .75) [contrast(x)] seed(integer) [reps(200) noconstant]`

With quantiles(), fit unweighted conditional quantile regressions instead of OLS.
Supply 1..19 strictly increasing finite values from .05 to .95. Clustering is
not supported in this mode. The same joint complete-finite sample and explicit
local RNG contract apply. Each replication draws N whole rows ONCE, then fits
all requested quantiles on those identical rows. Adding quantiles does not
consume extra random draws. There is no shared global RNG state.

Report QuantileBootstrapResult: coefficients/covariance in quantile-major order,
intercept first within each quantile, with names such as q0.25:intercept and
q0.25:x. The full joint covariance includes cross-quantile blocks and uses B-1.
Marginal percentile95 intervals use R type7; they are NOT simultaneous coverage
bands. Without contrast(), no analytic coefficient p-values or tests are added.
Covariance may be singular (rank at most B-1); the report does not silently
regularize it.

Optional `contrast(variable)` requires exactly two requested quantiles and one
unique coefficient name from the fitted design. It reports the higher-quantile
coefficient minus the lower-quantile coefficient. This includes `intercept`
when present; with `noconstant` only predictors can be selected. Unknown or
ambiguous names fail before bootstrap fits. The point difference uses the
original fits. Its standard error is the sample standard deviation (B-1) of
the shared-replicate coefficient differences, computed directly for numerical
stability. The zero-difference statistic and two-sided p-value use an
asymptotic standard normal reference; its 95% interval is a normal-Wald
interval. B-1 is the variance divisor, not Student-t degrees of freedom. This
single contrast is not a general restriction interface, multiplicity
adjustment, simultaneous test, or finite-sample coverage guarantee.

If the contrast variance is zero or unrepresentable, the valid marginal
bootstrap report remains available and the contrast carries an explicit
unavailable reason with null normal inference fields. No retained bootstrap
fit or automatic figure is installed.

Pairs inference permits heteroskedastic conditional outcomes under IID row
sampling and regular identified quantiles with positive conditional density.
It is not survey/design-based inference for the source's household survey, and
supplies no finite-sample coverage guarantee. Quantile LP bounds are N>K,
N<=10000, K<=32 and N*K²<=10000000; additionally
N*K²*number_of_quantiles*(replications+1)<=100000000. The latter counts original
fits as well as all resampled fits, not worst-case LP iteration cost. Each coefficient fit retains the existing solved/identification/numerical-certificate
requirements. A failed first solve may refine the SAME LP at stricter solver
tolerance once; all active-constraint/gap/score checks remain unchanged. This
never redraws a sample. Iteration/time limits apply per attempt.

Bootstrap fits require point coefficients only; no IID residual-spacing SE is
estimated on a replicate. Ambiguous or numerically unresolved coefficients,
rank deficiency, lack of representable response variation or failed convergence
abort with original/replicate index and quantile. No draws are discarded/retried.
The original data and retained model stay unchanged. Fit qreg explicitly for
current-row predictions; the bootstrap report does not replace that fit.
See examples/mma04p2qreg/bootstrap.md for the original 19-quantile workflow.
