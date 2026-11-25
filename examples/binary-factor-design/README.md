# Binary model factor design

## Bounded workflow

Add additive categorical main effects to `logit` and `probit` using the
established factor syntax:

```text
logit outcome income i.region
probit outcome income ib(West).region
```

Continuous predictors remain numeric columns. Factor predictors may be finite
numeric codes, text or booleans. The default omitted level, explicit reference
level, treatment-column ordering and coefficient names follow the existing
`regress i.var` / `ib(level).var` design contract. With `noconstant`, every
observed level contributes a column and an explicit reference is invalid.

The fit uses complete finite outcome/predictor rows in source order. Missing
outcomes or predictors are excluded; a complete row whose outcome is not
exactly 0 or 1 fails. Robust and one-way clustered fits retain their current
covariance conventions. The retained fit stores typed factor levels so
`predict ..., xb` and `predict ..., pr` score current data without refitting.
Missing values and factor levels absent from the estimation sample produce
NULL predictions. An explicit reference absent from the estimation sample
fails the fit.

`estat margins, factor(var)` reports probability contrasts for a retained
factor. Interactions remain outside this slice. Bare continuous margins and
numeric `discrete(var)` changes remain available only for continuous-only
designs. This synthetic method fixture does not claim a textbook illustration
replication.

## Factor probability contrasts

`estat margins, factor(region)` reports one retained-sample average
probability contrast for each non-baseline level of the selected factor. With an
intercept, the baseline is the fit-time omitted reference; with `noconstant`, it
is the first sorted retained level. Other continuous and factor terms stay at
their observed values in each retained row. The average is over the original
estimation sample, not current data or a covariate-mean profile.

For level `l` against baseline `b`, the estimand is
`N^-1 sum_i [F(x_i, l) - F(x_i, b)]`. Its gradient is the corresponding average
difference in link-density-weighted design rows. The full retained model-based,
robust or one-way clustered covariance supplies a delta-method SE, two-sided
normal z test and 95% interval. These are conditional-on-observed-X
probability contrasts, not causal effects or interaction effects. Missing or
unseen factor levels, factors absent from the retained fit, weighted/multiway
inference and simultaneous intervals remain outside this slice.

## Independent validation

`Rscript reference.R` uses R 4.6.2 and seed 84523 to generate
`binary_factor_data.csv` plus independent fit, covariance, prediction, summary
and factor-contrast references. The fixture has 240 rows; IDs 5, 12 and 18 are
excluded from fitting for missing region, x and y respectively, leaving N=237
with 110 events.
Current-row prediction does not require y, so ID 18 still receives a
probability/index.

The runtime tests compare exact retained row indices, parameter names/order,
coefficients, standard errors, log likelihood and full model-based covariance
for both links under five designs: default text reference, explicit `West`
reference, `noconstant`, numeric-code factor and boolean factor. Default-design
`xb` and probabilities are checked for every current row. Robust and clustered
covariance matrices are independently formed in R from individual/cluster score
meat and the documented `N/(N-1)` or `G/(G-1)` correction; coefficient estimates
are unchanged across covariance choices. Factor contrasts are checked for both
links under default, explicit-reference and `noconstant` designs, plus numeric
and boolean factors. Robust and cluster covariance contrasts are checked for the
text factor. The Base R `glm` starting values are polished to score norm below
1e-13. Probit covariance is independently computed as inverse observed
information with R's `pnorm`, `dnorm` and matrix operations; Base R `vcov(glm)`
uses Fisher/IRLS information instead.

Coefficient, standard-error, covariance, likelihood and prediction comparisons
use absolute tolerance 2e-8. The runtime and CLI contracts also cover missing
factor/continuous/outcome rows, complete-row nonbinary outcomes, default and
explicit references, `noconstant`, robust/cluster fits, unseen current-data
levels returning NULL, prediction after data edits, retained Wald/GOF use,
factor metadata in named estimates, JSON/text script execution and average
factor probability contrasts. A complete row with y outside {0,1}, an absent
explicit reference and a factor-margin request for an absent term fail clearly;
failed fits clear stale estimation state. Interactions remain unsupported.

Run the executable workflows from this directory with
`tabdat-explore-rs --json -f logit.td` or `tabdat-explore-rs --json -f probit.td`;
use `--text` for terminal reports. The scripts include grouped GOF, factor
probability contrasts, current-row `xb`/`pr` predictions and named estimate
reports. These synthetic method contracts do not claim a Cameron–Trivedi
illustration replication.
