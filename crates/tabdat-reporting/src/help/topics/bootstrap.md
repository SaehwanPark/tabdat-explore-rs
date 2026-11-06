# bootstrap

`bootstrap y xvars, seed(integer) [reps(200) noconstant] [cluster(unit) | quantiles(values) [contrast(variable)]]`

By default, compute a read-only pairs-bootstrap OLS report on CURRENT complete-finite
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
G*largest_group_size*replications*K²<=200,000,000 before fitting; the IID
work cap remains100,000,000.
ClusterBootstrapOlsResult adds first-seen cluster_sizes/count/order and replicate
observation counts to the common report; IID BootstrapOlsResult is unchanged.

Supplied transformations are fixed: this is OLS on current columns, not an
automatic within/RE fit or re-estimation of variance components. For the bounded
manual-program within workflow, joint-case person means are computed first and
whole persons are resampled afterward. No retained model or analytic test
covariance is installed.

## Whole-cluster pooled Poisson

`bootstrap y xvars, model(poisson) cluster(unit) seed(integer) [reps(200)]`

This explicit model selects a read-only unweighted pooled Poisson report, not
OLS or a command prefix. An intercept and cluster(unit) are required;
noconstant, quantiles, contrast, offsets and weights are not supported. Counts
must be exact nonnegative integers at most2^53−1 on the joint complete-finite
outcome/predictor/cluster sample, using the same loaded-type checks as poisson.
Fit the original pooled Poisson and then refit all coefficients on each of B
whole-group copies. Repeated groups are duplicated blocks; group membership is
not a fixed/random effect. Any original/replicate rank, zero-outcome,
numerical-boundary or convergence failure aborts without redraw or a partial
report; the bounded solver is not a general analytic separation/existence test. The original data and any retained fit/named reports remain unchanged.
Fit poisson explicitly before this command if predictions are needed.

Bounds: B=2..500 (default200), G=2..256, K<=16 including the intercept,
G*largest_group_size<=40,000 and G*largest_group_size*B*K²<=2,500,000,000,
using checked arithmetic. This is a conservative work bound, not a runtime
promise; the underlying Poisson iteration limit applies to every refit.
The existing continuous local xoshiro stream and first-seen cluster ordering
apply; this mode additionally aborts if a replication needs more than G+1024
RNG words. Independent clusters are assumed. Bootstrap covariance is B−1 and
intervals are marginal type7 percentile95; finite B has Monte Carlo error. No
normal/t p-values, BCa/studentized intervals, few-cluster guarantee, survey
weights/design or causal inference are supplied.

CLI-v1 `ClusterBootstrapPoissonResult` includes the full covariance, original
and replicate coefficients, replicate sizes and ordered zero-based cluster
indices for every draw. `cluster_labels` and `cluster_sizes` are in first-seen
complete-case order; each index selects the corresponding label and size.
Labels are exact internal key strings, not necessarily display formatting: for
example DECIMAL `9007199254740993.1` is `90071992547409931e-1`.
The seed is decimal text. Terminal output shows the original/mean/bias/SE and
intervals; the full matrix and draw audit are available in JSON. See
examples/mma24p2poiscluster/individual_pooled_bootstrap.md for the bounded
Chapter24 individual workflow and independent evidence.

## Whole-group conditional Poisson FE

`bootstrap y xvars, model(conditional_poisson) cluster(unit) seed(integer) [reps(200)]`

This read-only report refits the existing conditional Poisson fixed-effects
model: cluster(unit) is BOTH the FE identifier and the resampling unit. There is
no common intercept; noconstant, quantiles, contrast, offsets and weights are
unsupported. As in xtpoisson, every raw unit key must be nonmissing and finite,
including keys on rows omitted for missing y/X; supported numeric, boolean and
text types preserve exact group identity. Joint-finite selection happens before
checking exact counts 0..100000. Groups with zero selected count totals are
excluded BEFORE defining the bootstrap frame; positive singletons remain but
contribute no slope information. Group order is first-seen among retained
positive-total complete cases. Current data and any retained model/named reports
are unchanged; fit xtpoisson separately for prediction.

Each replication samples G original retained groups with replacement and
refits all slopes with a distinct FE identity for each copied group. B=2..500
(default200), G<=256 with G>K, K=1..16 slopes, G*largest_group_size<=40000,
and G*largest_group_size*B*K²<=2500000000. Checked limits are conservative;
the underlying FE fit also enforces its original sample, rank, numerical and
convergence guards. Any failure aborts the entire report without redraw; the
bounded optimizer is not a general proof of existence or identification.
At most G+1024 RNG words may be used per replication. The same local stream,
B−1 covariance, type7 marginal percentile95 and seed serialization as pooled
Poisson apply. Finite B has Monte Carlo error; no BCa/studentized interval,
analytic bootstrap p-value, RE, few-group coverage, survey or causal inference
is claimed.

CLI-v1 `ConditionalPoissonBootstrapResult` includes original slopes, full
covariance, every replicate's slopes and N, and ordered zero-based group draws.
`cluster_labels`/`cluster_sizes` are in first-seen positive-total order, and
`zero_total_group_count`/`zero_total_observation_count` disclose the exclusion.
Labels use exact internal key strings, not necessarily display formatting.
Terminal output gives slope/mean/bias/SE/interval summaries. See
examples/mma24p2poiscluster/individual_conditional_fe_bootstrap.md for the
bounded Chapter24 workflow and its independent evidence.

## Whole-group shared-gamma Poisson RE

`bootstrap y xvars, model(gamma_poisson_re) cluster(unit) seed(integer) [reps(200)]`

This read-only report refits the existing shared-gamma Poisson random-effects
model. The cluster(unit) is BOTH the shared-effect group and resampling unit;
every sampled copy gets a distinct shared gamma effect, so copies of the same
source group are not merged. Joint-finite cases define the frame; zero-total
and singleton groups remain. Every raw group key must be nonmissing and finite,
including on rows omitted for missing y/X. Numeric, boolean and text types
preserve exact group identity. Selected counts and group totals must be exact
integers 0..100000. Group order is first-seen among complete cases. All
intercept/slopes and ln(alpha) are refit; this is not independent-row NB2,
conditional FE, a posterior-effect forecast or a retained bootstrap model.
Fit xtpoisson ..., re separately for marginal predictions.

B=2..500 (default200), G<=256 with G>K, K=slopes+2 joint parameters including
intercept and ln(alpha), slopes1..14, G*largest_group_size<=40000 and
G*largest_group_size*B*K²<=3000000000. Checked limits are conservative; the
underlying RE fit also enforces its original sample, count-total, rank,
dispersion, information and convergence guards. Every failed original or
replicate fit aborts without partial report, redraw or pooled-model fallback;
the bounded optimizer is not a general existence or separation test. At most
G+1024 RNG words may be used per replicate. The same local stream, B−1
covariance, marginal type7 percentile95 and seed serialization as the pooled
Poisson bootstrap apply. Finite B has Monte Carlo error; no BCa/studentized
interval, analytic bootstrap p-value, survey inference, few-group guarantee or
causal inference is claimed. Offsets, weights and noconstant are unsupported.

CLI-v1 `GammaPoissonBootstrapResult` includes original joint parameters, full
covariance, every replicate's coefficients/N and ordered zero-based group
indices; `cluster_labels`/`cluster_sizes` follow first-seen complete-case order.
`zero_total_group_count` and `singleton_group_count` disclose original groups.
Labels are exact internal key strings, not necessarily display formatting.
Terminal output gives the same parameter/mean/bias/SE/interval summaries.
Data, labels, named reports and retained fit remain unchanged on success or
failure. See examples/mma24p2poiscluster/individual_gamma_re_bootstrap.md for
the bounded Chapter24 workflow and independent validation evidence.

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
