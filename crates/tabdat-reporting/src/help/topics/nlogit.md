# nlogit

Explicit **earlier non-normalized** two-level nested choice, not the preferred
common-utility RUM normalization:

```text
nlogit chosen price catch, group(person) alternative(alt) nest(branch) upper(shore shore_income) nonnormalized
predict probability, pr
```

`nonnormalized` is mandatory: no preferred-normalization fit is silently substituted.
There are exactly two nests, each with at least two alternatives. Every case
contains the same4..16 alternatives, each once, with an identical alternative/
nest map. Group keys may be exact numeric/text/boolean; alternative/nest keys
are exact numeric integers within ±(2^53−1). Missing/nonfinite keys, duplicate
pairs, ragged sets or changed maps fail **before missingness exclusion**, including
otherwise excluded sets. `chosen` is numeric0/1 with one success per retained set.
A missing/nonfinite response or any predictor excludes its entire set; other
finite outcome-domain errors are checked only on retained complete sets.

The lower utility is `x*beta`, conditional probabilities normalize its exponentials
within each nest, and upper utility is
`z*gamma + lambda_nest*logsumexp(x*beta)` with positive free nest scales. `upper()`
is optional; its numeric columns must be constant inside each retained case/nest.
Only explicit numeric designs are supported; there is no automatic intercept,
factor, interaction or raw-income-to-interaction update. Crucially, **case-common
lower utility shifts do not cancel when scales differ**. Do not interpret these
slopes as the preferred common-marginal-utility RUM specification.

G choice sets, not long rows, are the independent units. Covariance is full joint
inverse **observed** information for coefficients and natural nest scales,
correction1, with asymptotic normal95 inference. Slope/upper tests use zero;
scale diagnostics explicitly test **one**, not the excluded zero-scale boundary.
Natural-scale Wald intervals may extend outside the positive domain. The fitted
lambda=1 conditional restriction re-estimates slopes; its LR uses chi-square2
only after checking local scale identification at that restriction. No regular
uniform-null omnibus chi-square is reported: scales can be unidentified there.
The fitted IV-bound check `(0,1]` is descriptive, not a composite RUM hypothesis
test; a higher likelihood does not establish random-utility consistency.

Bounds:1..16 lower and0..16 upper columns, total P≤32 including two scales,
G>P, G≤10000, candidate rows≤40000 and retained rows*P²≤20million. Two deterministic
positive-scale starts and damped Newton/Fisher optimization are not a global
maximum proof. Reported covariance is observed information, never the optimization
fallback Fisher matrix. Both averaged score and full scaled observed-Newton step
must be≤1e-10 with positive observed information. Rank failure, unidentified
scales/restriction, separation or nonconvergence fail without ridge/pseudoinverse
inference. Failed fitting clears prior inference but leaves data unchanged.

Prediction reads **current** complete group/alternative/nest keys and numeric
predictor columns, not outcomes. New group IDs are allowed; the retained map is
required. Missing/nonfinite predictors NULL the whole set. Structural, constancy,
schema or arithmetic errors publish no column and retain data/model. Only
`predict newvar, pr` is supported, not xb/interval/std/saving/outcome selectors.
Manual before/after contrasts are point associations, not analytic AMEs, causal
effects or uncertainty intervals. `estat margins`, general tests/named model
comparisons, preferred normalization, more than two nests, singleton nests,
weights, robust/cluster/survey, ragged availability and automatic factors are
not implemented in this bounded model.
