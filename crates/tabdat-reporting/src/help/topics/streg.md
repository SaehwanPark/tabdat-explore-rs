# streg

Installed syntax:
`streg time_var xvars, failure(event_var) dist(weibull) [robust noconstant]`

Native unweighted entry-zero/single-spell Weibull proportional-hazards ML:
H(t|X)=exp(X beta)*t^a, h(t|X)=a*exp(X beta)*t^(a-1), S(t|X)=exp(-H), a>0.
Time and X/event must be numeric. Joint missing/nonfinite rows are excluded
before support checks; remaining times must be positive and events exact0/1,
with at least one failure. No silent support deletion or shape clipping.

Full joint beta/lnshape observed covariance (model-based) or independent-spell
score sandwich with N/(N-1) (robust, NOT residual-DF HC1). Normal coefficient95
and positive exponentiated-logshape95 intervals. Shape=1 tests constant hazard
within Weibull, not model adequacy. Same-sample null refits intercept+shape, or
shape only with PH index fixed0 for noconstant. Robust LR p-values withheld;
rank-guarded slope Wald remains separate. Time units affect density likelihood;
the reported Stata-normalized value adds the fixed sum of event log times.

Limits: N<=10000, P=number of location coefficients+shape<=64, N>P,
N*P*P<=20000000, full-rank regular information and bounded Newton convergence.
A failed new runtime fit clears stale inference, not data/labels.

Point predictions on CURRENT rows (event never needed):
- `predict index, xb`: PH X beta, also the bare predict default; time not needed.
- `predict current_hazard, hazard`: current instantaneous hazard.
- `predict cumulative_hazard, cumhaz`: current cumulative hazard.
- `predict net_survival, survival`: current net survival.

Timed modes require the fitted time name and X. Any required missing/nonfinite
input gives NULL. For complete finite inputs, nonpositive time or unrepresentable
requested calculations fail before publication; representable survival0 is valid.
At most10000 current rows. Columns are DOUBLE/NULL, no replacement, no point
uncertainty, no count/binary/mean/residual prediction or auxiliary options.
Failed prediction preserves data order/labels/owned model.

Event0 includes right-censoring for the selected cause. Cause-specific net
survival is NOT all-cause survival or competing-risk incidence. Conditional UI
points are not causal effects; independent-spell/model/censoring assumptions apply.

Native exponential/cluster still parse but explicitly fail. No weights, delayed
entry/left/interval censoring, frailty/Cox, generic tests/margins/named storage.
The exponential source workflow uses a separate existing offset-Poisson composition.
See examples/mma17p4duration/weibull.md for the original Table17.8 full-design
workflow, independent full-covariance/source audit and limitations. R/Python are
development reference tools, not application dependencies.
