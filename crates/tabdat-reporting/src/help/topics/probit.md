# probit

Installed syntax:
`probit y x1 c.x#c.z i.group [, robust cluster(group) noconstant]`

Fit an unweighted independent-Bernoulli binary probit model. Reports model-based
inverse observed-information (OIM) covariance from Newton steps, coefficient
standard errors, normal z tests/95% intervals, log likelihood, LR chi-square and
McFadden pseudo-R². No N-K covariance scaling or Student-t inference is used
for the default model-based fit.

With `, robust`, the same MLE uses an unclustered score-sandwich covariance
with N/(N-1) scaling (the ML convention, **not** linear HC1's N/(N-K)). Its
bread is inverse observed information at the converged fit, not Fisher
information or a penultimate-iterate Hessian. Coefficient and retained-design
`estat margins` inference use this covariance with normal 95% intervals.
Observations must be independent; this is not cluster/survey inference or
protection against a wrong conditional-mean interpretation. Ordinary LR p-values
are suppressed: the likelihood difference/pseudo-R² are descriptive. An
omnibus Wald test uses the selected covariance to jointly test all
non-intercept coefficients = 0, with asymptotic chi-square(q), not F.

With `, cluster(group)`, scores are summed within each group before forming
the sandwich, with G/(G-1) correction and the same converged observed-information
bread. Coefficient and continuous/discrete retained-X effect inference remain
asymptotic normal,
**not** G-1 Student t. Clusters must be independent; few/imbalanced clusters can
make this approximation unreliable. At least two retained groups are required,
which is only a computability guard. Cluster IDs may be numeric, text or boolean;
NULL/nonfinite IDs are excluded jointly with model missingness before validating
the outcome. Wide integers/Decimals keep exact identity; floating -0 and +0 are
one group. `robust` and `cluster` cannot be combined. This is neither a
random-effects model nor survey/design-based inference.

The omnibus Wald test also uses clustered covariance after `cluster(group)`.
Without an intercept it tests every coefficient. Rank-limited/singular slope
covariance makes this test unavailable without discarding the fit; no
pseudoinverse or reduced-q test is substituted. This is not a specification
test or small-cluster reliability guarantee. General binary `test` remains
unsupported.

Workflow (after loading numeric outcomes/continuous predictors and any typed
categorical columns):
- `probit insured age bmi`
- `probit insured age i.region`
- `probit insured age ib(West).region`
- `estat margins` — original continuous-variable derivatives, including the
  product rule for `c.x#c.z`, with delta SE/95% intervals
- `estat margins, discrete(treated)` — average probability change for an
  original continuous numeric 0/1 predictor, including in additive factor
  designs without interactions
- `estat margins, factor(region)` — average probability contrasts of retained
  levels versus the fit-time reference
- `estat classification` — retained-sample class counts and descriptive
  percentages at the default probability cutoff of 0.5
- `predict probability, pr`
- `predict probability_se, pr se`
- `predict probability_interval, pr interval`
- `predict index_b, xb`
- `head 8`
- `export predictions.csv`

Sample and state:
- Outcomes and continuous predictors must be numeric. Explicit factors use
  `i.var` or `ib(level).var` and accept finite numeric codes, text or booleans.
  A continuous interaction uses `c.x#c.z` and adds only the product column;
  include `x z` separately to add main effects. Missing/nonfinite model values
  and nonfinite products are excluded jointly; retained y must be
  exactly 0/1 with both classes. Other finite outcomes on complete rows are
  errors, not silently mapped to nonzero=1.
- `i.var` omits the first sorted level in the retained sample; `ib(level).var`
  selects a reference explicitly. With `noconstant`, every observed level is
  included and `ib(level)` is invalid. Expanded names use `<var>_<level>` with
  stable collision suffixes. The retained design records typed levels and the
  reference used.
- The intercept is included unless `noconstant`; names/order and exact retained
  indices are recorded. Without an intercept the LR null is probability 0.5,
  not an estimated constant probability.
- A successful fit owns its original design/sample. Available `estat margins`
  results remain unchanged after current-data edits. Prediction instead scores
  current predictor rows, without requiring the outcome. Factor predictions use
  the retained typed levels; missing values or levels absent from the fit yield
  NULL. Probability columns use the same validated Gaussian function the fit
  used (`tabdat_gauss_phi`).
- A new fit attempt clears the previous model, including on failure. Refit
  successfully before requesting subsequent inference/prediction.

Convergence and supported limits:
- Newton steps on the per-observation observed information require a small
  score AND full undamped step (absolute 1e-10 in unit-column-norm coordinates)
  within 100 information evaluations; iterations with a nonpositive diagonal
  fall back to Fisher scoring for that iteration only. Separation/numerical
  singularity/nonconvergence cannot publish a fit merely because likelihood
  looks flat. This is not an analytic separation classifier; extreme finite
  designs may also fail numerical-resolution bounds.
- Additive one-way factor main effects are supported. Bare `estat margins`
  reports original continuous-variable derivatives, including in mixed designs,
  and product-rule effects for continuous interactions; all factor and
  interaction coefficients enter the full delta gradient. `factor(var)` reports
  retained-level probability contrasts. `c.x#c.z` adds one product without
  implicit main effects. Factor interactions and `##` expansion are unsupported.
  The bounded `discrete(var)` form accepts one original continuous term with
  exact 0/1 values and both levels in additive designs; it explicitly rejects
  interaction models because changing one input also changes its product term.
  Expanded factor dummy columns are not selectable. See `help estat` for effect
  estimands and inference.
- Only one-way clustering is supported. Weighted/survey probit is not installed.
  Current-row `pr se` and `pr interval [level(95)]` use the selected retained
  covariance, conditional on current predictors; intervals transform normal
  latent-index bounds through the probit link. They are not outcome prediction
  intervals or few-cluster guarantees. `lincom` combines retained latent-index
  coefficients with the selected covariance and asymptotic normal inference;
  probability effects remain under `estat margins`. `test` supports general
  linear restrictions on retained probit coefficients with an asymptotic Wald
  chi-square reference; see `help test`. Normal coefficient inference and, for
  model-based fits only, the overall LR test are reported directly by `probit`;
  do not substitute linear t/F inference.
- Terminal/`--text`, CLI-v1 JSON and scripts share the fitted model and semantics.
  JSON returns `BinaryProbitResult`; continuous `estat margins` returns
  `ProbitMarginalEffectsResult`, `discrete()` returns
  `BinaryDiscreteChangeResult`, and `factor()` returns
  `BinaryFactorContrastsResult`; `estat classification` returns
  `BinaryClassificationResult`. Prediction retains metadata-only `PredictionResult`.

Reference example: `examples/mma14p1binary/README.md` (Cameron–Trivedi 2005,
§14.2 fishing-mode binary illustration, independent R validation).
Clustered method example: `examples/contraception_cluster/README.md` (external,
not textbook coverage). See `help estat`, `help predict`, and
`docs/reference/command-results.md`.
