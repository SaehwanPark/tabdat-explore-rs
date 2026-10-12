# Poisson plugin feasibility contract

Status: **in progress; Phase 1 remains partial**. Next bounded loop after #169.

## Native prerequisite

Pinned gretl 2026b `count_model` dispatches `count_data_estimate` through
`plugins.c`; it maps to `poisson.so`, cached for process lifetime. Current static
libgretl-only builds do not provide that module. Upstream plugin modules link
libgretl; linking another static copy into a plugin risks duplicated native
process state. Evaluate an **opt-in** shared-core + Poisson-plugin build, leaving
the existing default static spike unchanged. Use one version/hash, explicit
build-owned plugin path, serialized initialization/calls/teardown and no cleanup.
No native module handles or public arbitrary plugin-path setter may escape.

## Numerical contract (declared before comparisons)

Classical integer-count Poisson MLE, log link, finite complete ordered synthetic
sample, intercept + two predictors. Owned names, coefficients, SEs, all covariance
entries, log likelihood and exact sample rows. Compare pinned Python TabDat and
independent R `glm(..., family=poisson())`. Predeclare absolute 1e-7 + relative
1e-6 for optimizer-derived values; discrete metadata/sample/labels exact.

Explicit limitations: no robust/cluster inference, offset/exposure, automatic
missing-row policy, prediction/model-state port or negative-binomial acceptance.
Python accepts nonnegative fractional outcomes; Gretl requires counts. Record
that mismatch rather than silently treating this integer slice as whole-command
parity. Invalid integer/negative/ragged/nonfinite input, native rank/convergence
errors, subsequent success and mixed independent thread calls need coverage.

Build/load tests on macOS first. Linux, relocatable distribution, sanitizer/leak
proof, complete native/plugin license review and remaining estimators stay open.
