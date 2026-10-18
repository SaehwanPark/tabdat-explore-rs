# probit

Installed syntax:
`probit y x1 x2 [, robust cluster(group) noconstant]`

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
are suppressed: the likelihood difference/pseudo-R² are descriptive, and no
robust omnibus Wald test is supplied.

With `, cluster(group)`, scores are summed within each group before forming
the sandwich, with G/(G-1) correction and the same converged observed-information
bread. Coefficient and retained-X delta-AME inference remain asymptotic normal,
**not** G-1 Student t. Clusters must be independent; few/imbalanced clusters can
make this approximation unreliable. At least two retained groups are required,
which is only a computability guard. Cluster IDs may be numeric, text or boolean;
NULL/nonfinite IDs are excluded jointly with model missingness before validating
the outcome. Wide integers/Decimals keep exact identity; floating -0 and +0 are
one group. `robust` and `cluster` cannot be combined. This is neither a
random-effects model nor survey/design-based inference.

Workflow (after loading numeric data):
- `probit insured age bmi`
- `estat margins` — continuous average predictor-column derivatives and delta SE/95% intervals
- `predict probability, pr`
- `predict index_b, xb`
- `head 8`
- `export predictions.csv`

Sample and state:
- Outcome and predictor columns must be numeric. SQL NULL/nonfinite model values
  are excluded jointly; retained y must be exactly 0/1 with both classes. Other
  finite outcomes are errors, not silently mapped to nonzero=1.
- The intercept is included unless `noconstant`; names/order and exact retained
  indices are recorded. Without an intercept the LR null is probability 0.5,
  not an estimated constant probability.
- A successful fit owns its original design/sample. `estat margins` remains
  unchanged after current-data edits. Prediction instead uses current predictor
  rows, without requiring the outcome; probability columns are computed by the
  same validated Gaussian function the fit used (`tabdat_gauss_phi`).
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
- Plain numeric predictor columns only: no automatic factor/interactions,
  discrete changes, or chain-rule interpretation of generated columns.
- Only one-way clustering is supported. Weighted/survey probit,
  prediction intervals and general `lincom`/`test` after probit are not
  installed. Normal coefficient inference and (for model-based fits only) the
  overall LR test are reported directly by `probit`; do not substitute linear
  t/F inference.
- Terminal/`--text`, CLI-v1 JSON and scripts share the fitted model and semantics.
  JSON returns `BinaryProbitResult`; `estat margins` returns
  `ProbitMarginalEffectsResult`. Prediction retains metadata-only `PredictionResult`.

Reference example: `examples/mma14p1binary/README.md` (Cameron–Trivedi 2005,
§14.2 fishing-mode binary illustration, independent R validation).
Clustered method example: `examples/contraception_cluster/README.md` (external,
not textbook coverage). See `help estat`, `help predict`, and
`docs/reference/command-results.md`.
