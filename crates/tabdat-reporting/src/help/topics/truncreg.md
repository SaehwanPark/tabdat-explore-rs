# truncreg

Installed syntax:
`truncreg y x1 x2, ll(number) [noconstant]`

IID homoskedastic Gaussian regression conditional on Y*>lower. The continuous
outcome is observed only in the retained population above a KNOWN fixed limit.
This is **truncation, not censoring**: do not replace unavailable outcomes with
zero or the bound. Use tobit for a censored response, or explicit Heckman twostep
for its distinct Gaussian selection model. Distributional assumptions are
substantive; these coefficients/effects do not establish causality.

Example (numeric y/x; missing outcomes left missing):
- `truncreg y x, ll(0)`
- `predict latent, xb`
- `predict retention, pr`
- `predict conditional_mean, mean`
- `estat margins`
- `head 8`
- `export truncreg_predictions.csv`

Sample/model:
- Joint finite y/X rows are selected in original order FIRST; missing/nonfinite
  model inputs are excluded, unrelated columns ignored. Every retained y MUST
  be strictly above lower. At/below-limit values cause an error, not clipping,
  censoring mass or silent deletion. Data, types, ordering and labels are kept.
- Location mu=X beta; sigma>0; conditional log likelihood is the sum of
  log(phi((y-mu)/sigma))-log(sigma)-log(Phi((mu-lower)/sigma)).
- CLI requires at least one numeric predictor, with an intercept unless
  noconstant. Intercept-only is kernel/API support, not accepted CLI syntax.
- N<=10000, location K=1..32 including any intercept, N>K+1 and
  N*(K+1)^2<=10000000. Names unique, lnsigma reserved. Full rank required.
- Known finite nonzero limits and entirely inactive bounds are supported.
  Boundary/unidentified/nonconverged/nonpositive-information or unrepresented
  tail/moment fits fail; no clipping, ridge, pseudoinverse or boundary inference.
  Concave canonical eta=beta/sigma^2,precision=1/sigma^2>0 Newton optimization
  uses effective response/RMS predictor units, not an extreme inactive bound.

Inference/reporting:
- FULL joint inverse observed beta/lnsigma information, correction1.
  Coefficient SE/z/p/95% intervals are asymptotic normal, not Student t.
- Sigma has a delta SE and strictly positive exponentiated log-Wald95 interval;
  no sigma=0 Wald test, residual-DF multiplier or invented pseudo-R2.
- Same-sample/same-bound null: refit intercept+scale if a constant is included;
  otherwise fix latent location zero and refit scale. LR df counts slopes;
  kernel/API intercept-only fits have no LR p-value.
- CLI-v1 TruncatedGaussianResult reports exact original sample indices, full
  beta-then-lnsigma covariance, scale/null/LR and numerical certificates.
  The latter use normalized CANONICAL coordinates, not raw coefficient scores.

Interpretation/post-estimation:
- Bare predict/xb = latent mu; pr = POPULATION retention Phi(t), not probability1
  conditional on observation; mean = mu+sigma*phi(t)/Phi(t), the conditional
  truncated mean, not Tobit E[max(lower,Y*)|X] or count n. t=(mu-lower)/sigma.
- Predictions need only current original X names, never current y, including
  rows excluded during fitting. Relevant missing/nonfinite X gives NULL;
  unsupported options, missing/nonnumeric predictors, target collisions or
  numerical failures preserve data/labels/fit and publish no column.
  At most10000 current rows; choose a fresh target, no replacement.
- estat margins uses RETAINED X: beta_j*mean(1-lambda*(lambda+t)), where
  lambda=phi(t)/Phi(t). Full joint beta/lnsigma delta inference is conditional
  on that design. Effects survive current data edits/drops; these are continuous
  derivatives, not discrete contrasts or causal effects.
- A supported failed new fit clears stale estimation, not data/labels. Refit
  successfully before predicting again. Syntax failures never reach fitting.

Not installed: upper/two-sided/variable limits, robust/cluster covariance,
weights, factors, generic test/lincom, named storage or prediction intervals.
No production R/Python dependency. The Chapter16 author source describes but
DOES NOT execute correct truncated MLE: the original-data example is a validated
source-intended extension, not a historical coefficient/table replay.
