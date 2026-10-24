# tobit

Installed syntax:
`tobit y x1 x2, ll(number) [noconstant]`

Fit the classic lower-censored Gaussian model Y*=X beta+epsilon,
epsilon IID N(0,sigma^2). This is censoring, not truncation: recorded y<=lower
contributes P(Y*<=lower|X), evaluated at the **limit**, not the recorded value;
y>lower contributes its Gaussian density. Recorded responses are preserved.
No observations are deleted or altered by fitting.

Use when an underlying continuous outcome is observed only above a known limit.
Distributional/homoskedasticity assumptions are substantive; this does not by
itself identify causal effects. Upper censoring, robust/cluster inference,
weights, factors, generic test/lincom and named-model storage are not installed.
The legacy parser accepts ul(), robust and cluster(), but runtime rejects them,
never silently ignores them. Do not remove an inference option merely to obtain
a fit when its assumptions are inappropriate.

Example (after loading numeric y/x):
- `tobit y x, ll(0)`
- `predict latent, xb`
- `predict uncensored, pr`
- `predict observed_mean, mean`
- `estat margins`
- `head 8`
- `export tobit_predictions.csv`

Sample/design:
- Joint complete finite y/X rows, in original order; unrelated columns do not
  affect inclusion. Missing/nonfinite model inputs are excluded, not filled.
- Intercept included unless noconstant. Finite nonzero lower limits work.
- Entirely uncensored samples are allowed; all-censored, unidentified,
  degenerate-scale, nonconverged or nonpositive-information fits fail explicitly.
- Retained N<=10000, location K=1..32 including any intercept, N>K+1,
  N*(K+1)^2<=10000000. Names must be unique. No ridge/pseudoinverse fallback.

Inference/reporting:
- Full joint inverse observed information for beta and ln(sigma), correction1;
  coefficients use asymptotic normal95, not historical Stata Student-t intervals.
- Sigma has a delta-method SE and strictly positive exponentiated log-Wald95
  interval. There is **no sigma=0 Wald test**.
- Refit same-sample, same-bound null: intercept+scale if a constant is included,
  otherwise fixed-zero latent location+scale. LR df is the number of slopes;
  intercept-only fits have no LR p-value. No pseudo-R2 label is supplied.
- CLI-v1 `TobitResult` includes counts, sample indices, full covariance and
  beta-then-lnsigma order, scale/null/LR and numerical convergence metadata.
  It does not emit full internal fitted vectors; inspect/export predictions.

Interpretation/post-estimation:
- Bare predict/xb = latent mean; pr = P(Y*>lower|X), **not censor probability**;
  mean = E[max(lower,Y*)|X], **not E[Y*|Y*>lower,X]** or count n.
- Current X only, including rows excluded at fit; no current y required.
  Missing/nonfinite X gives NULL. Unsupported options, missing/nonnumeric
  predictors, existing target or numerical failure leave data/labels/fit intact.
  At most10000 current prediction rows; use a fresh target name, not replacement.
- `estat margins` reports continuous observed-mean AMEs beta_j*mean(Phi(z_i))
  over retained estimation X. Full joint beta/lnsigma delta inference is
  conditional on that design. These are not discrete changes or causal effects.
  Retained gradient summaries preserve effects after current data edits/drops.
- Successful fits replace the active model. A failed new fit clears stale
  estimation but preserves data. Successfully refit before predicting again.

Original-source reference work: `examples/mma16p1tobit/README.md`.
This bounded command does not complete the remaining Chapter16 methods.
