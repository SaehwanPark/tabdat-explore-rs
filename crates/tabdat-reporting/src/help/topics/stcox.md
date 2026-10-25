# stcox

`stcox time [xvars], failure(event) [robust]`

Unweighted entry-zero single-spell Cox proportional hazards with Breslow ties.
No intercept; omitting xvars fits a baseline-only Nelson–Aalen cumulative hazard.
Its exp(-H) survival is not the Kaplan–Meier product estimate. At a tied time all
failures and censors remain at risk. Efron/exact ties are not installed.

Joint finite time/event/X rows enter the sample before positive-time and exact
binary event validation; invalid retained support errors rather than silently
deleting observations. At least one failure and full centered-design rank are
required; constant design directions are not identified. N<=10000, K<=64, N>K,
N*K*K<=20000000; robust additionally needs N>=2. Bounded strict Newton convergence
and regular undamped information are required; separation/identification failures
do not publish a fit. Executed fit attempts clear stale inference, not data/labels.

Model-based inverse partial-likelihood observed information or robust full
efficient martingale-score sandwich times N/(N-1), independent-spell asymptotic
normal95 coefficient intervals. This is not failure-only score covariance, OLS
HC1, clustered or survey inference. Zero-slope Wald uses the selected covariance
and withholds rank-limited references. The same-sample zero-beta null gives a
partial-likelihood ratio; robust fits withhold its p-value/reference. Partial
likelihood is not a duration density or a cross-family AIC comparison.

The owned Breslow baseline reports failure knots, risk/event counts, log/natural
increments and cumulative H, and exp(-H). Its reference vector is ALL retained
predictor means (including dummy means), in predictor order, not dummy-zero R
centering. Unrepresentable natural quantities are NA/JSON null; log values remain
finite. Baseline uncertainty is unavailable.

`predict target [, xb | cumhaz | survival]` uses CURRENT X; cumhaz/survival also
use CURRENT time but never event. The retained baseline is right-continuous,
including ties; time0/before first failure gives H0/S1 and times after the last
failure hold the final step, not a validated tail extrapolation. Missing/nonfinite
required inputs yield NULL; negative finite time, schema/support/overflow or
existing-target errors publish nothing and preserve fit/data/labels/order.
Survival0 is valid even if cumulative H is unrepresentable. xb needs no time or
event column. At most10000 current rows; typed DOUBLE/NULL output.

There is no instantaneous `hazard` from baseline jumps, prediction/baseline
uncertainty, residual/margins/general tests/named reports, weights/cluster/strata,
delayed entry, other censoring/truncation, frailty or PH-adequacy diagnostic.
Net cause-specific survival is not all-cause survival or competing-risk incidence;
conditional UI associations are not causal effects. See
examples/mma17p4duration/cox.md for the exact bounded source-analysis contract.
