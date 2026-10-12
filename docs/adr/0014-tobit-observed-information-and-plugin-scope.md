# ADR 0014: Tobit observed information and isolated interval plugin

- Status: Accepted for bounded feasibility, not a public command port
- Date: 2026-10-12

## Evidence

Pinned Python TabDat prefers `r:survival::survreg` for Gaussian Tobit, but falls
back to scipy BFGS/L-BFGS-B when the R bridge cannot load. On this host the pinned
rpy2 binary references missing R 4.5 libraries; ABI fallback also requires
`R_getVar`, absent in installed R 4.3.2. The actual executor therefore uses its
Python fallback. This environmental distinction is recorded, not hidden by
changing the pinned checkout, its lock or installed bridge.

Fallback coefficients match R/native within the predeclared MLE tolerance, but
its BFGS inverse-Hessian approximation yields SE discrepancies of approximately
0.00050 (left-only) and 0.00078 (two-sided). Those SEs fail the declared
`1e-7 + 1e-6*abs(reference)` rule. Preserve them as original output, not accepted
observed-information reference values. The intended R route uses observed
information, as does the pinned gretl interval module.

## Decision

Accept classical **observed-information** Tobit covariance for this spike, with
joint beta/log(sigma) covariance explicitly labeled. Validate it against standalone
R survival Gaussian interval2 and a separately labeled external Python probe.
That probe uses the same fallback likelihood, zero start and optimizer settings;
it replaces only quasi-Newton covariance extraction with symmetric finite-
difference observed Hessian evaluation. Do not call its output unmodified
executor results or claim fallback SE parity. Do not widen tolerances or select
a different data fixture. A future public port must document this covariance
correction and resolve environment-independent oracle authority explicitly.

Add opt-in `tobit-plugin`, building only source-matched `interval.so` against the
same verified shared core/lifetime boundary as ADR 0012. Internal `shared-plugins`
selects shared linkage; count and Tobit select their modules independently. Default
static and count-only capabilities remain unchanged. Both plugin features share
the same bounded private path and serialized process-lifetime loader; there is
no public path setter or unload operation. Source builds still exclude libR.

## Consequences

No Cargo dependency, advisory waiver, production estimator, runtime R/Python or
new redistribution approval. Absolute plugin paths remain spike-only and are
not relocatable release packaging. The C model owns the donated full covariance
matrix; the shim checks TOBIT family and shape before borrowing one scalar.
Rust receives owned copies and no foreign matrix/model ownership.

[Tobit evidence](../feasibility/tobit.md),
[original executor values](../../spikes/gretl-prototype/fixtures/tobit-original-python.csv),
[observed Python probe](../../spikes/gretl-prototype/fixtures/tobit_observed.py).
