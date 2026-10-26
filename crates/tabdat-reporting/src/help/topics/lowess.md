# lowess

How to invoke:
`lowess y x, gen(newvar) [bandwidth=<0,1>]`

Omitting bandwidth preserves the existing parser default 2/3. Specify 0.8 explicitly
for a Stata-default-span source-data adaptation; the algorithms' historical tie
conventions are not thereby made identical.

What it does:
Append a DOUBLE column of nonrobust centred-rank local-linear tricube fits for
current training rows. No Python/R or external smoother is called.

Sample and ordering:
Both Y and X must be numeric. Joint NULL/nonfinite rows are excluded and receive
NULL in the new column. Analysis coordinates are float64 (wide integer/decimal
values may round into ties); original columns/types/labels and row order remain
unchanged. Stably sort the eligible pairs by numeric X, retaining eligible input
order within ties, and restore fits to original positions. Signed zeros tie.
**Equal X can have different fits; changing within-tie row order can change them.**
This is an ordered row smoother, not a unique out-of-sample function of X.

Method and limits:
The bandwidth is a fraction of sample ranks, not a fixed distance in X units.
For eligible N, k=floor((N*bandwidth-0.5)/2); use the clipped rank interval
[i-k,i+k] (it shrinks at sample endpoints). With d=max endpoint distance from X_i,
use z=(X-X_i)/d and tricube weights (1-(abs(z)/1.0001)^3)^3, and fit a weighted
intercept and linear slope to predict at z=0. Exact d=0 windows use their local
arithmetic mean; k=0 returns eligible Y exactly. Response scaling is window-local,
so a huge distant Y cannot erase small local responses. Ordinary float64 numeric
resolution within each window still applies.

Require 2 <= N <= 10,000, finite 0 < bandwidth < 1, N*bandwidth >= 0.5 and
N*min(N,2*k+1) <= 20,000,000. Unsupported bandwidth/work, nonnumeric columns,
unrepresentable local differences/fits and existing target names error; no
silent clipping, ridge, pseudoinverse or replacement. Calculate all fits before
typed staged column publication; validation/numerical/collision failures preserve
data, labels and any retained fit.

Interpretation and model state:
Use several spans to inspect smoothing sensitivity, not confidence bands or causal
effects. There are no robust reweighting iterations, interpolated fitted rows,
observation weights, uncertainty, degrees of freedom, covariance or p-values.
The command neither installs nor clears an estimation state: later `predict`,
`test` or `estat` still concern a PREVIOUS retained model, never this smoother.
`LowessResult` reports exact sample indices, span/k, coincident-window count,
tie/boundary/fallback conventions and unchanged model-state semantics in CLI-v1.
R `stats::lowess`, even iter=0/delta=0, uses different nearest-neighbour windows;
its defaults are not a numerical oracle for this centred-rank method.

Examples:
- `lowess wage educ, gen(wage_lowess)`
- `lowess wage educ, gen(wage_local) bandwidth=0.4`
- `export "smoothed.csv"`

Links:
- `examples/mma09p1np/lowess.md` — bounded PSID source-data adaptation and evidence;
  literal historical Stata tied-curve geometry is not certified.
- `docs/reference/command-results.md`
