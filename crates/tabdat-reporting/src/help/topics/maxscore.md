# maxscore

`maxscore binary_y continuous_x`

Fit a bounded binary maximum-score classifier with an intercept and exactly one
continuous predictor. The outcome must be exact numeric 0/1. Missing or
nonfinite outcome/predictor rows are excluded; a finite nonbinary outcome in an
otherwise usable row fails the fit. The estimator searches every split between
distinct observed predictor values in both slope directions and includes both
constant classifiers. It reports all tied empirical-score maximizers and a
deterministically selected unit-Euclidean-norm coefficient representative.

```text
maxscore charter lnrelp
predict predicted_charter, class
```

The fit report includes the exact empirical score, retained sample, confusion
counts and every maximizing region. `predict ..., class` applies the retained
threshold rule to current rows in their current order. Missing or nonfinite
predictors produce missing classifications; predictions are numeric 0/1.

This is point estimation and sign classification only. It does not provide
standard errors, tests, confidence intervals, probability predictions, weights,
multiple predictors, or causal effects. Ordinary nonparametric bootstrap
standard errors are not a valid general inference method for maximum score, so
no inferential fields are reported. The normalized coefficient scale is a
representative, not an identified scale parameter. Tied regions mean the
finite-sample maximizing classifier may not be unique.

See `examples/mma14p1binary/README.md` and `examples/mma14p1binary/maxscore.td`
for the source-derived MMA §14.7.5 workflow, independent Base R reference and
numerical audit.
