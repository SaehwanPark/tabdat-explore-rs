# ADR 0017: Proper two-step selection covariance and failed-fit cleanup

- Status: Accepted for bounded feasibility, not product heckman support
- Date: 2026-10-12

## Contract and inference decision

Pinned Python command performs probit on all complete observations, then selected
OLS with inverse Mills ratio. Native OPT_T implements the same two-step point
estimator and matches original coefficients. Distinguish total selection sample
from selected outcome sample; do not relabel native selected N as original total N.

Pinned second-stage ordinary-OLS SEs ignore estimated first-stage Mills uncertainty.
Native adjusted outcome covariance differs by ~8.20e-4 in SE on seed-44107. Accept
explicitly labeled proper two-step covariance for feasibility, validated by separately
labeled external Python correction and independent R probit/QR/observed-information/
two-step algebra. Preserve original values, do not claim old outcome-SE parity,
modify the oracle or widen abs 1e-7 + rel 1e-6 tolerance.

Let W be selected outcome design plus Mills, Z selected selection design,
delta=lambda*(lambda+Z gamma), C=(W'W)^-1 and B=W' diag(delta) Z.
Use observed probit Vp, sigma²=mean(u²)+MillsCoefficient²*mean(delta), rho=lambdaCoef/sigma,
and outcome covariance
`sigma² C [W'W-rho²(W' diag(delta) W-B Vp B')] C`.
Do not substitute glm expected/Fisher covariance silently. R uses independently
evaluated observed probit Hessian. Correlation-boundary/clipping cases remain
unsupported. No unrequested MLE or robust/cluster extension.

Native composite VCV cross-equation block contains zero placeholders, not an
estimated cross covariance. Expose separate outcome/selection blocks only; no
cross-equation independence or full joint Wald inference claim. Sigma/rho are
point diagnostics, not additional coefficient-SE rows.

## Native boundary decision

Independently select heckit.so under existing verified shared-core/source/lifetime
controls. Native temporary Mills series can leak on early second-stage failure:
heckit_init returns a failed model, and heckit_estimate destroys container then
returns before its normal dataset cleanup. Shim records original dataset variable
count and restores it on success/failure. Keep all cleanup inside serialization;
no borrowed Rust slice mutation or public unload/path setter.

A direct low-level probe confirms successful first-stage probit followed by too-few-
selected second-stage failure, actual native variable count restored, and a later
successful fit on the same dataset. Scalar shape observation exposes no foreign
ownership. MODEL coefficient/VCV/name storage is owned and remains independent of
the temporary series; failed buffers and metadata are guarded before access.

No Cargo dependency, runtime Python/R, waiver, missing/unselected-Y-NULL support,
no-intercept, MLE, post-estimation/session or distribution approval. Cross-platform
native safety/sanitizer/leak and source/notice/relocation acceptance remain separate.

[Heckman report](../feasibility/heckman.md),
[original values](../../spikes/gretl-prototype/fixtures/heckman-original-python.csv),
[external correction](../../spikes/gretl-prototype/fixtures/heckman_corrected.py).
