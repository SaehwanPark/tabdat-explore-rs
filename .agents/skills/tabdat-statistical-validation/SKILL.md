---
name: tabdat-statistical-validation
description: Use when implementing or reviewing TabDat estimators, covariance and inference, prediction, post-estimation state, or Python/Rust/trusted-reference statistical validation.
---

# TabDat statistical contracts and validation

## When to use

Use for estimator semantics and numerical trust, including foundational inference,
IV/panel/causal workflows, regularization/DML, and optional spatial/Bayesian work
when explicitly in scope. Do not expand estimator breadth just because a backend
supports it. FFI safety belongs to `tabdat-native-backends`.

## Required inputs

- Public estimator/post-estimation contract and required parity subset.
- Fixture data and provenance, Python oracle revision, Rust revision, backend
  name/version, and trusted external reference with version/output provenance.
- Applicable proposal §§7, 9–10 and roadmap statistical phase plus Phase 13,
  reached through [AGENTS.md](../../../AGENTS.md).

If references are absent, separate implementation work from validation status.
Python/R may be used in external reference tooling, never as an accidental core
runtime dependency. Do not invent reference numbers or claim fixtures exist.

## Workflow

1. Recover the statistical contract before backend selection: formula/design
   columns and order, intercept/factor conventions, weights, filters, missing-row
   exclusion, sample identity, parameterization, and model-family requirements.
2. Make `EstimationSample` explicit and inspectable. Compare retained rows as well
   as N; identical sample sizes can conceal different samples. For panel, IV,
   spatial, or cross-fitted workflows, record alignment, instruments, group/time
   indexing, weights ordering, folds, and seeds as applicable.
3. Specify inference: covariance mode, robust variant, clustering, finite-sample
   corrections, degrees of freedom, reference distribution, confidence level,
   and diagnostics. Record optimizer/convergence criteria for nonlinear models.
   Backend defaults are inputs to normalize, not the TabDat public contract.
4. Design owned problem/result structures and family-specific model state.
   Keep sample construction, covariance labeling, `predict`, `test`, `lincom`,
   margins, common reporting, and compatibility checks TabDat-owned. Define state
   behavior after failed estimation or incompatible post-estimation calls.
5. Select or implement the numerical route without rebuilding mature kernels.
   If a candidate backend cannot honor the contract, record the mismatch and
   evaluate a bounded normalization, alternative, or documented deferral.
6. Validate the same fixture across Rust, Python TabDat, and a trusted reference.
   Compare coefficient order/names, sample, coefficients, covariance, SEs, confidence
   intervals, predictions, diagnostics, convergence, and failure behavior where
   relevant. Review typed results before terminal formatting.
7. Diagnose disagreement in order: sample/design → parameterization → estimator
   settings → covariance/corrections → optimizer/numerics → rendering. Agreement
   between Python and Rust does not overrule a reliable contradictory reference.
8. Record implementation status separately from reference-validation status.
   Update only evidence-supported roadmap items and run the applicable root checks.

## Comparison contract

Each validation record must include:

- Estimator/options and backend/version; Python and Rust revisions.
- Reference implementation/version and fixture path/provenance (include a hash
  when needed to establish identical data); execution command and platform/date.
- Sample identity and ordering; coefficient/covariance dimensions and labels.
- Per-quantity absolute/relative tolerances and rationale, declared before judging
  results; document the comparator rule, especially for near-zero quantities.
- Exact comparisons for discrete metadata, sample membership, and error categories;
  explicit handling of non-finite values rather than allowing NaNs to pass silently.
- Observed differences, pass/fail/blocked by quantity, known intentional deviations,
  and separate implementation and validation statuses.

Never silently loosen tolerances or regenerate expected values from the code under
validation. Preserve original outputs when investigating a suspected oracle bug.
For stochastic procedures distinguish seeded reproducibility from justified
sampling-error tolerances; identical seeds alone do not imply cross-backend draws.

## Expected outputs and validation

Include a well-conditioned nominal fixture plus applicable edge/failure cases:
missing rows, collinearity/rank deficiency, invalid weights, too few clusters,
separation, nonconvergence, unsupported covariance, and incompatible prediction
or post-estimation state. Do not invent desired behavior for these cases: recover
or explicitly decide it. Review sample and inference correctness before declaring
numerical success; high-risk changes need an explicit review step, even when one
engineer performs it separately from implementation.

Return the statistical contract, validation record, test commands/results, review
findings, and unresolved differences using the
[shared handoff format](../../../docs/harness/tabdat/team-spec.md) if durable.
Stop with `partial` or `blocked` when a required reference cannot run, numerical
mismatches remain unexplained, or a statistical decision needs approval. A backend
returning coefficients is implementation evidence, not proof of statistical trust.
