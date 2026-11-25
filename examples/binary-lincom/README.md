# Linear combinations after logit and probit

## Planned scope

Enable the existing `lincom expression` command after retained binary logit and
probit fits. The target is the affine coefficient combination
`k + c' beta` on the model's linear index (log odds for logit, latent probit
index for probit), using the covariance selected by the fit. Report its
estimate, standard error, normal z test, two-sided p-value and 95% normal
interval with no residual degrees of freedom.

This command does not exponentiate logit coefficients into odds ratios and does
not calculate probability contrasts or marginal effects; use `estat margins`
for probability-scale effects. It must preserve the retained fit and current
data. Unknown coefficients and nonlinear coefficient expressions fail without
changing either.

The independent reference will use the deterministic mixed-factor binary design
already documented in `examples/binary-factor-design/README.md`: logit and
probit with model-based, row-robust and one-way clustered covariance. Base R
will independently evaluate selected affine combinations from its coefficient
and covariance results. Runtime and JSON/text CLI contracts will also cover
unknown names, nonlinear expressions, retained factor coefficients, zero
standard errors and no-mutation behavior.

This is synthetic method evidence for a known Chapter 14 gap. It does not claim
another Cameron and Trivedi illustration or an interaction effect.
