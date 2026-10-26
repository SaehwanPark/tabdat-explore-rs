# streg

Installed syntax:
`streg time_var xvars, failure(event_var) dist(weibull) [robust noconstant]`
or `streg time_var xvars, failure(event_var) dist(gompertz) [robust noconstant]`.
At least one explicit predictor is required; no syntax-only new family is implied.

Native unweighted entry-zero/single-spell Weibull proportional-hazards ML:
H(t|X)=exp(X beta)*t^a, h(t|X)=a*exp(X beta)*t^(a-1), S(t|X)=exp(-H), a>0.
Time and X/event must be numeric. Joint missing/nonfinite rows are excluded
before support checks; remaining times must be positive and events exact0/1,
with at least one failure. Original numeric event values are checked before f64
rounding: near-one DECIMAL codes are invalid on eligible rows, but do not reject
rows already excluded by joint missingness. No silent support deletion or shape clipping.

For Weibull, full joint beta/lnshape observed covariance (model-based) or independent-spell
score sandwich with N/(N-1) (robust, NOT residual-DF HC1). Normal coefficient95
and positive exponentiated-logshape95 intervals. Shape=1 tests constant hazard
within Weibull, not model adequacy. Same-sample null refits intercept+shape, or
shape only with PH index fixed0 for noconstant. Robust LR p-values withheld;
rank-guarded slope Wald remains separate. Time units affect density likelihood;
the reported Stata-normalized value adds the fixed sum of event log times.

Gompertz: h(t|X)=exp(X beta+gamma*t), H(t|X)=exp(X beta)*t*exprel(gamma*t),
S=exp(-H), exprel(z)=expm1(z)/z with exprel(0)=1. Gamma is unrestricted,
not a log/positive shape; gamma0 is the exponential limit. Full joint beta/gamma
observed inverse or independent-spell score sandwich N/(N-1); all cross terms,
natural normal95 gamma CI/z(gamma0). This tests exponential within Gompertz,
not adequacy. Same-sample null refits intercept+gamma, or gamma only at index0
for noconstant; robust LR p withheld, selected slope Wald separate. Gamma<0
implies positive survival mass at infinity: do not force the tail to0 or call it
identified causal cure. Hazard units are inverse recorded time; gamma rescales
inversely with time units. Density/normalized likelihood conventions match above.

Limits: N<=10000, P=number of location coefficients+one ancillary parameter<=64, N>P,
N*P*P<=20000000, full-rank regular information and bounded Newton convergence.
A failed new runtime fit clears stale inference, not data/labels.

Point predictions on CURRENT rows (event never needed):
- `predict index, xb`: PH X beta, also the bare predict default; time not needed.
- `predict current_hazard, hazard`: current instantaneous hazard.
- `predict cumulative_hazard, cumhaz`: current cumulative hazard.
- `predict net_survival, survival`: current net survival.

Timed modes require the fitted time name and X. Any required missing/nonfinite
input gives NULL. For complete finite inputs, negative time (or zero for Weibull)
or unrepresentable requested calculations fail before publication; representable
survival0 is valid. Gompertz time0 has h=exp(xb), H0/S1; xb is X beta, not
X beta+gamma*time. Finite-time extrapolation does not validate the infinite tail.
At most10000 current rows. Columns are DOUBLE/NULL, no replacement, no point
uncertainty, no count/binary/mean/residual prediction or auxiliary options.
Failed prediction preserves data order/labels/owned model.

Event0 includes right-censoring for the selected cause. Cause-specific net
survival is NOT all-cause survival or competing-risk incidence. Conditional UI
points are not causal effects; independent-spell/model/censoring assumptions apply.

Native exponential/cluster still parse but explicitly fail. No weights, delayed
entry/left/interval censoring, frailty or generic tests/margins/named storage.
Native Cox/Breslow is a separate `stcox` workflow, not a streg distribution.
The exponential source workflow uses a separate existing offset-Poisson composition.
See examples/mma17p4duration/weibull.md for the original Table17.8 full-design
workflow, independent full-covariance/source audit and limitations. The bounded
Gompertz Table17.8 workflow/status is examples/mma17p4duration/gompertz.md. R/Python are
development reference tools, not application dependencies.
