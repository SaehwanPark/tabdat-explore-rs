# Linear combinations after logit and probit

## Supported contract

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

Use the displayed coefficient names, including retained factor dummy names.
Model-based, row-robust and one-way clustered inference all use the covariance
stored by the successful fit. The normal reference is asymptotic; no
finite-sample or few-cluster correction is implied. Undefined zero-SE tests
retain their point interval and display unavailable statistic and p-value.

The independent reference uses the deterministic mixed-factor binary design in
`examples/binary-factor-design/README.md`. Base R 4.6.2 evaluates slope plus
offset, a factor-dummy combination and an intercept/slope combination for
logit and probit under model-based, row-robust and one-way clustered covariance.
Results are stored in
`examples/binary-factor-design/binary_factor_lincom_reference.csv` and generated
with `Rscript reference.R` from that fixture directory. Runtime and JSON/text
CLI contracts cover retained factor coefficients, data edits, state-preserving
unknown/nonlinear failures, zero standard errors and null residual degrees of
freedom.

This is synthetic method evidence for a known Chapter 14 gap. It does not claim
another Cameron and Trivedi illustration or an interaction effect.
