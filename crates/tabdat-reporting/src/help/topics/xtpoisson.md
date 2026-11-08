# xtpoisson

Installed bounded syntax:
`xtpoisson y xvars, fe|re id(group) [distribution(gamma|normal) for re]`

Choose exactly one of fe or re and one structural group variable. No persistent
panel declaration or time key is needed. Predictors are numeric columns, not
factor expansions. All three methods use joint-finite model cases, exact integer
counts0..100000, and model-based inference only. Raw group keys must be
nonmissing/finite numeric, text or boolean, even on omitted model rows; exact
wide integer/decimal group identity is preserved. No weights or offsets.

## Conditional FE

`xtpoisson visits income age, fe id(commune)`

Conditions on each selected group's count total. Joint-finite selection precedes
removal of all-zero groups. Positive singletons remain but supply no slope
information. No common intercept. Covariance is inverse conditional information,
without a DF multiplier; normal95 coefficients, conditional same-total
zero-slope LR and joint slope Wald chi-square inference are reported.

`predict fitted, n` uses the retained positive group's normalization at current
X, NOT a fresh normalization of the current subset. Group totals are reproduced
on the fitted design, not promised after changing/subsetting data. Current group
logical type must match the fit. Unseen, missing and zero-total IDs give NULL
means/residuals. `predict index, xb` is slope-only and requires no ID.
`predict residual, residuals` is current y minus the conditional mean.

Bounds: candidate rows<=40000, slopes1..32, rows*slopes²<=20000000, raw
groups<=10000 and positive-total groups>slopes. Group-anchored/RMS-scaled fitting
requires full-rank information and both scaled score/full-step tolerances1e-10.

## Shared-gamma RE

`xtpoisson visits income age, re id(commune)`

Counts are conditionally independent Poisson(u_g*exp(intercept+X beta)) given a
shared u_g~Gamma(shape=1/alpha,rate=1/alpha). E(u)=1, Var(u)=alpha. This induces
within-group dependence; groups are independent and effects are assumed
independent of regressors. It is NOT independent-row NB2 mixing. All selected
groups, including all-zero groups and positive/zero singletons, remain.

The intercept, slopes and ln(alpha) are jointly estimated. Covariance is inverse
FULL joint observed information including coefficient/dispersion cross terms,
without DF correction. Coefficients and ln(alpha) use normal95 inference; alpha
has delta SE and exponentiated log-scale CI. The ln(alpha) z test has null
alpha=1, NOT zero variance. The separate alpha=0 LR compares the same-sample
refitted pooled Poisson and uses the conventional 50:50 chi-square(0,1) mixture.
Slope Wald has a chi-square reference. These references assume the declared model.

`predict fitted, n` is the marginal population mean exp(intercept+X beta).
`predict index, xb` includes the intercept. `predict residual, residuals` is
current y minus the marginal mean. No current group key is required. These are
not posterior group effects or conditional-on-estimated-effect forecasts.

Bounds: candidate rows<=40000, slopes1..32, rows*(slopes+2)²<=20000000,
raw groups<=10000, selected groups>slopes+2 and selected group count totals
<=100000. The total bound limits stable gamma-recurrence work/cache. ln(alpha)
must be strictly inside(-12,12). RMS-scaled design and per-group score/information
require scaled score and full undamped-step tolerances1e-10, finite full-rank
design and positive final joint information. Intermediate search damping is not
a covariance adjustment. Boundary/nonconvergent fits fail explicitly; no clipped
alpha or pooled fit mislabeled as RE. This bounded command does not supply an
automatic zero-variance fallback.

## Normal random-intercept RE

`xtpoisson visits income age, re distribution(normal) id(commune)`

Default `re` is still gamma; `distribution(gamma)` makes that choice explicit.
Distribution selection is invalid with FE. Normal RE has a shared mean-zero
Gaussian **log** effect u_g~N(0,variance), independent of X, with conditional
mean exp(intercept+X beta+u_g). It is not mean-one gamma or independent-row NB.
All selected zero-total and singleton groups remain; unbalanced groups are
supported without a time key.

Joint beta and lnsigma2=ln(variance) use inverse full observed information,
including cross terms, without a DF multiplier. Coefficients/lnsigma2 use
normal95; sigma has delta SE=(sigma/2)*SE(lnsigma2) and positive log-Wald CI.
The lnsigma2 z test tests variance=1, NOT variance=0. The separate zero-variance
LR refits full-X pooled Poisson on the same sample with a 50:50 chi-square(0,1)
reference. Slope Wald uses chi-square. These are model/asymptotic references,
not robust or few-group corrections.

`predict index, xb` returns X beta including the intercept, NOT log(n).
`predict fitted, n` returns the population mean exp(X beta+variance/2).
Residuals are current y minus this mean; n/xb require no current y or ID.
No posterior effects are estimated. Population residuals need not sum to zero.

Bounds: candidate rows<=40000, slopes1..14, raw groups<=10000, selected
G>slopes+2, group count totals<=100000,176*rows*(slopes+2)^2<=20000000.
Adaptive paired80/96-node GH checks log integrals and centered moment units at
every evaluation; relative discrepancy<=1e-9 is a diagnostic, not an error
bound. RMS-scaled, undamped SPD joint Newton steps use at most100 iterations,
32 halving trials per iteration, per-group score<=1e-10 and relative step<=1e-12. No ridge,
covariance repair or optimizer/pooled fallback. lnsigma2 is strictly(-8,8),
fit X beta[-40,40]; marginal log means[-700,700]. Pooled LR must exceed1e-8 to
numerically distinguish the boundary, not as a significance cutoff. Boundary,
rank, quadrature, unrepresentable inference or convergence failure is an error;
use `poisson` explicitly if a zero-variance model is appropriate.

## State, prediction and limits

A failed new fit clears stale estimation state without changing data; a parser
failure leaves the previous model. Reporting and successful/failed export retain
the model. Script effects before an export failure are not rolled back; export
refuses to overwrite an owner file. Use a fresh output directory.

Only xb, n and residuals are supported, without auxiliary/uncertainty options.
Missing/nonfinite required current inputs yield NULL. Residuals require finite
current y. Gamma RE finite-input predictions that are not representable fail
atomically; finite xb remains available without computing an overflowing mean.
Zero means from floating-point exponential underflow are permitted. Prediction
uses rows<=40000 and the corresponding parameter-dimension squared work bound;
normal prediction needs no quadrature/176 multiplier and checks marginal log
means[-700,700]. It never refits.

No robust/cluster/survey covariance, weights/offsets, factor syntax,
posterior effects/uncertainty, generic margins/test/lincom, automatic bootstrap
or general Stata compatibility is claimed. See the Chapter24 individual FE/RE
workflows for exact samples and independent reference evidence. Existing
Chapter23 scalar SQL compositions remain separate supported workflows.
