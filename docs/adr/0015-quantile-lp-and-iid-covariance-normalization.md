# ADR 0015: Quantile LP and canonical iid covariance

- Status: Accepted for bounded estimator feasibility, not product qreg support
- Date: 2026-10-12

## Evidence

Pinned executor uses statsmodels QuantReg IRLS (max_iter 1000, p_tol 1e-6), iid
Epanechnikov density and Hall-Sheather bandwidth by default. Native gretl uses a
mature regression-quantile LP and a different iid sparsity covariance estimator.
Returning native SEs as Python-compatible would silently change inference.

On fixed seed-21307, original IRLS stops after 57/25/34 iterations at q=.25/.5/.75,
but coefficients differ from independent HiGHS/R LP solutions by approximately
5.25e-7 / .00614 / .00589. Check-loss objective excess is 8.20e-7 / .00150971 /
3.25e-5, so the difference is not only a different equally optimal coefficient
representation. All three fail the declared abs 1e-8 + rel 1e-7 coefficient rule.
Original output is preserved. R Barrodale-Roberts emits a possible-nonuniqueness
warning; independent HiGHS and native solutions nonetheless agree within declared
tolerance and the original IRLS objective is strictly worse. Do not erase warnings
or infer general uniqueness from this one fixture.

## Decision

Use source-matched gretl quantreg for LP coefficients and retain explicitly labeled
`native_iid_covariance` as feasibility diagnostic only. Normalize the exposed iid
covariance in safe Rust to the pinned **Epanechnikov/Hall-Sheather** convention:
alpha .05, population outcome SD, type-7 linear residual IQR/1.34 scale,
q(1-q)/density² times inverse X'X. Reuse checked owned classical QR/OLS covariance
and its sigma² to recover inverse X'X; do not add another runtime inversion engine.
Normal inverse probabilities come through the existing serialized scalar-only
native math boundary. Reject nonfinite/degenerate density or invalid bandwidth.

Validate with independent R quantreg and R density/linear algebra, and a separately
labeled external HiGHS LP probe using the same canonical covariance convention.
Do not call it the unmodified executor or claim original default IRLS parity.
No tolerance change, fixture reselection or pinned checkout modification.
The q=.75 native sparsity covariance also differs from R's iid estimator by
~.00059 on this fixture, despite near-identical coefficients; this remains a
recorded diagnostic difference, not the canonical result.

Add independent `quantile-plugin` selecting only quantreg.so under ADRs 0012/0014
shared-core/source/provenance/lifetime controls. One-element tau matrix is owned
and freed inside the shim; single-quantile model metadata is a scalar, not a borrow.
No domain/native model pointer escapes.

## Consequences

No runtime Python/R, Cargo dependency, waiver, robust/alternate kernel, missing
sample, post-estimation or product integration acceptance. The extra translated
rq code carries University of Illinois/NCSA copyright 2002 Roger Koenker. Its
permissive license permits modification/sublicensing, with source/binary notice,
disclaimer and nonendorsement obligations; those are not optional when shipping.
It is compatible in principle with the proposal's AGPL intent alongside gretl's
GPL context, but complete dependency/native distribution approval remains open.

[Quantile evidence](../feasibility/quantile.md),
[original values](../../spikes/gretl-prototype/fixtures/quantile-original-python.csv),
[external LP probe](../../spikes/gretl-prototype/fixtures/quantile_lp.py).
