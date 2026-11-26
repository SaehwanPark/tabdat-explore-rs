# heckman

Fit an IID Gaussian sample-selection model. The bare form uses joint maximum
likelihood; `twostep` explicitly requests the conventional generated-regressor
estimator:

```text
heckman y x1 x2, selectdep(d) select(z1 z2)
heckman y x1 x2, selectdep(d) select(z1 z2) twostep
heckman y x1, selectdep(d) select(z1 z2) noconstant
```

At least one outcome and one selection predictor are required. `d` is numeric
exact0/1:1 means the outcome is observed/selected. `noconstant` removes the
intercept from both equations. Robust and cluster covariance, weights, factors,
named model storage, general tests and prediction uncertainty are unavailable.

## Sample and model

First require finite d and all selection regressors; reject nonbinary d on those
rows before f64 rounding of exact integers/decimals. A selected unit also needs
finite y and outcome regressors. Drop a bad selected unit from both stages.
Missing y/outcome-only X on unselected units does not exclude them. Unused
columns do not affect the sample. Ordered original row positions are retained
for the eligible selection sample and its selected outcome subset. Fitting does
not mutate data or labels; a failed fit clears stale estimation state.

The joint model is `d = 1[z'a + u > 0]`, `y* = x'b + v`, with standard-normal
u, Gaussian v with scale sigma, and correlation rho. Maximum likelihood uses
the full observed-data likelihood and a deterministic start from the explicit
two-step estimator. A result is returned only after strict optimizer convergence
and positive-definite full observed information. No rho clipping, ridge, or
pseudoinverse is used. At most10,000 eligible rows,32 parameters per equation,
66 joint parameters, and200,000,000 bounded likelihood-work units are accepted.

Selection and outcome designs may differ. An excluded selection predictor is
an assumption, not an automatic validity test. If none is present,
identification rests on Gaussian functional form and may be weak. Coefficients
describe latent-mean associations, not causal effects.

## Inference

Maximum likelihood reports the full joint inverse observed-information
covariance in `[outcome coefficients, selection coefficients, lnsigma, athrho]`
order, asymptotic-normal95 intervals, transformed sigma/rho intervals, the
delta-method uncertainty for lambda=rho*sigma, and the one-df LR test of
rho=0 against the same-sample independent-equations model. ML covariance uses
correction1 and includes cross-equation terms.

The explicit `twostep` option fits a probit on all eligible units, then OLS on
selected units with the generated inverse Mills regressor. Its probit and
corrected augmented-outcome covariance blocks are separate; there is no joint
likelihood, LR test, cross-equation inference, or scale/correlation SE. Sigma
and rho are moment points only. The two methods are not interchangeable.

## Current point predictions

```text
predict latent, xb
predict selected_probability, pr
predict inverse_mills, mills
predict selected_mean, ycond
predict zero_filled_contribution, yexpected
```

`xb` is x'b. `pr` is Phi(z'a); `mills` is phi(z'a)/Phi(z'a). `ycond` is
E[y*|d=1,x,z]=x'b+rho*sigma*phi/Phi. `yexpected` is the zero-filled
contribution E[d*y*|x,z]=Phi*x'b+rho*sigma*phi. It is not an imputation or a
causal effect, and may not equal the mean of the originally measured outcome
when zero has no meaning for an unobserved response. In the RAND illustration,
the author program explicitly cautions against interpreting zero-filled log
expenditure this way.

Current y/d are unnecessary. Each prediction requires only the relevant
outcome and/or selection regressors; missing/nonfinite required regressors
produce NULL. Predictions are point-only and limited to10,000 current rows.
Nonrepresented numerical values, missing/non-numeric columns, invalid targets,
or unsupported options fail before publishing a column and preserve the owned
fit, data and labels.
