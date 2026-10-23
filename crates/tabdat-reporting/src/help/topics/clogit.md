# clogit

Fit unweighted single-choice McFadden conditional logit on long-format data:

```text
isid person alternative
clogit chosen price catch, group(person) alternative(alternative)
predict probability, pr
```

Every row represents one available alternative in one choice set. `chosen` must
be numeric and exactly0 or1, with exactly one selected alternative per retained
set. `group()` and `alternative()` are required; group keys may be exact numeric,
text or boolean values. Alternative codes must be exact integers within
±(2^53−1). Every group must contain the same2..16 alternatives, each once.
Missing/nonfinite keys, duplicate pairs and ragged choice sets fail explicitly.
There is no implicit available-alternative deletion or denominator change.

After structural validation, a missing/nonfinite outcome or predictor on **one
row excludes its entire choice set**. Other finite outcome-domain errors are
checked on retained complete sets. A common group intercept cancels from the
conditional likelihood; none is estimated. A common intercept/within-set
constant predictor or a singular within-set design cannot identify a slope.
Only explicit numeric design columns are supported; alternative constants or
interactions are not automatically generated.

Independent units for likelihood/inference are the **G choice sets**, not the
long-row count N. Full joint conditional Fisher covariance has correction1,
asymptotic normal95 coefficient inference, and no N−K/G−K adjustment. The null
sets every utility coefficient to zero, giving equal probabilities within each
set; the LR reference is chi-square(K). McFadden R² compares those likelihoods.
Interpretations assume independent decisions and the model's IIA substitution
structure; these are conditional associations, not identified causal effects.

Bounds: G>K, G≤10000, candidate/retained rows≤40000, K≤32 and rows*K²≤20million.
A scaled damped Newton fit must satisfy both averaged per-set score and full
undamped step≤1e-10. Separation-like numerical degeneracy, rank failure and
nonconvergence fail without ridge/pseudoinverse inference. A failed new fit clears
the old inference cache, but does not mutate data.

`predict newvar, pr` uses **current** group/alternative keys and numeric predictors;
current outcomes are unused and need not be present. New group IDs are allowed,
but every current group must contain exactly the complete retained alternative
universe. A missing/nonfinite predictor produces NULL for **every row of that
group**. Structural/current-schema errors or nonfinite utility arithmetic fail
atomically, preserving the fit and data. Prediction has the same row/group caps.
Bare/xb, outcome-selector, interval and other prediction kinds are not supported.

Explicit before/after predictions can report finite price/catch shifts under the
fitted model. These are point contrasts, not analytic AMEs, causal effects or
uncertainty intervals. `estat margins`, general `test`/`lincom`, named fit reports,
weights, robust/cluster/survey covariance, IIA tests, ragged availability, automatic
factors and multi-success matched case-control conditional logistic regression
are not implemented in this bounded slice. Dataset labels are not reinterpreted
as choice codes; native terminal/JSON results retain numeric alternative IDs.
