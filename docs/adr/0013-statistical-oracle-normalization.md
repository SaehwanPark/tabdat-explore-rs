# ADR 0013: Normalize NB2 dispersion and validate the optimizer target

- Status: Accepted for estimator feasibility; not a public command port
- Date: 2026-10-12

## Context / evidence

Pinned Python TabDat 16b45d9 labels statsmodels NegativeBinomial's final **raw
alpha** coefficient `lnalpha`; its actual `model.lnalpha` is log(raw alpha).
On the fixed seed-1267 fixture the executor reports alpha 0.7908691787178671,
while log(alpha) is -0.23462271210122043. Treating that value as log dispersion
would change Var(Y)=mu+alpha*mu^2 incorrectly.

Default Python BFGS reports convergence but has maximum score 0.00135934;
its parameter difference from independent R reaches 8.78e-6. Initial Rust tests
against these values failed the predeclared abs 1e-7 + rel 1e-6 rule. The original
output and failure are preserved; no tolerance was widened or data reselected.

## Decision

The spike returns explicit NB2 family and variance-scale coefficient **alpha**.
Normalize the known Python naming defect in the behavioral contract rather than
reproduce a false parameterization. Preserve original executor output separately.
For numerical feasibility, compare the same statistical objective against R MASS
and a separately labeled **external Python refinement** using Newton, identical
data/start point, tol 1e-12 and explicit score/convergence checks. Do not describe
the refined output as the unmodified executor's result.

Use full observed joint beta/alpha information, including cross terms; MASS's
conditional beta-only GLM covariance is not a substitute. Predeclared numerical
tolerances remain unchanged. Native/refined-Python/R all agree within them.

## Consequences

Python naming and stopping-precision differences are recorded deviations, not
unresolved reasons to halt other slices. No pinned checkout is modified and no
Python/R runtime dependency is added. A future product port should expose correct
parameter metadata and explicit convergence diagnostics; it must not claim exact
old default-optimizer parity. Other families need their own semantics/references.

[NB2 report](../feasibility/negative-binomial.md),
[original executor output](../../spikes/gretl-prototype/fixtures/nb2-python-default.csv),
[external probe](../../spikes/gretl-prototype/fixtures/nb2_python.py),
[R reference](../../spikes/gretl-prototype/fixtures/nb2_reference.R).
