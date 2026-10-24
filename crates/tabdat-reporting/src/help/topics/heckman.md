# heckman

Fit IID Gaussian sample selection by the **explicit two-step** method:

```text
heckman y x1 x2, selectdep(d) select(x1 z) twostep
heckman y x1, selectdep(d) select(z1 z2) twostep noconstant
```

At least one outcome and one selection predictor are required. `d` is numeric
exact0/1:1 means the outcome is observed/selected. `noconstant` removes the
intercept from BOTH equations. The bare form remains parser-only/deferred ML;
it is not silently treated as two-step. ML, robust/cluster covariance, weights,
factors, named model storage, general tests and margins are not implemented.

## Sample and identification

First require finite d and all selection regressors; reject nonbinary d on
those rows before f64 rounding of exact integers/decimals. Selected units also
require finite y and outcome regressors. Drop a bad selected unit from BOTH
stages. Missing y/outcome-only X on unselected units does not exclude them.
Unused columns do not affect the sample. Original eligible and selected row
identities are retained separately; fitting does not mutate data or labels.
A failed explicit two-step fit clears stale estimation state.

Selection probit is followed by outcome least squares with the generated Mills
regressor phi(Z gamma)/Phi(Z gamma). An excluded selection predictor is a user
assumption, not an automatic validity test. With none, a warning explains that
identification relies on Gaussian functional form and may be weak. Associations
are not causal effects. Inspect the design and uncertainty, not only N.

## Inference and limits

Selection coefficients use inverse observed probit information; augmented
outcome coefficients use the conventional Heckman/Greene generated-regressor
correction, NOT ordinary/HC second-stage OLS SEs. Both use correction1 and
asymptotic normal95. The Mills coefficient is alpha=rho*sigma. Sigma/rho are
moment points only: no scale/correlation SEs, cross-equation tests or joint ML
likelihood/LR are supplied. Any displayed likelihood belongs ONLY to selection.

Require both selection classes, full rank, more rows than stage parameters,
positive finite corrected covariance and abs(rho)<1. No clipping, ridge or
pseudoinverse is substituted. Relative sigma<=1e-12 is numerically degenerate
(scale: selected outcome range with intercept, max(abs(y)) without intercept).
At most10000 eligible rows,32 parameters per stage including intercept/Mills,
and N*q²+N_selected*p²<=20million are supported.

## Current point predictions

```text
predict latent, xb
predict selected_probability, pr
predict inverse_mills, mills
predict selected_mean, ycond
predict contribution, yexpected
```

`xb` is X beta. `pr` is Phi(Z gamma); `mills` is phi/Phi. `ycond` is
E[Y*|D=1,X,Z]=X beta+alpha*phi/Phi. `yexpected` is the zero-filled contribution
E[D Y*|X,Z]=Phi*X beta+alpha*phi, not an imputation or causal effect. These are
not Tobit censoring/truncation or count means; `mean`/`n` are unavailable here.

Current y/d are unnecessary. xb requires only outcome X; pr/Mills only selection
Z; ycond/yexpected require both. Relevant missing/nonfinite regressors produce
NULL; irrelevant missing columns do not interfere. Predictions are point-only
and limited to10000 current rows. Nonrepresented Mills/numerical overflow,
missing/non-numeric columns, invalid target or unsupported options fail before
publishing a column and preserve the owned fit, data and labels.
