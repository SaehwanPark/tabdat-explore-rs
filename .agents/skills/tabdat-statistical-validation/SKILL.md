---
name: tabdat-statistical-validation
description: Validate statistical meaning and trusted numerical outputs for estimators, inference, prediction, and post-estimation features.
---

# Statistical implementation and validation

Apply when statistical results or model state change. The required rigor here is
**not optional** merely because other migration gates have been simplified.

1. Specify supported estimator behavior: sample construction, exclusions,
   weights, design matrix, intercept/factors, and parameter naming/order.
2. Specify inference: covariance type/correction, finite-sample adjustment,
   degrees of freedom, reference distribution, uncertainty measures, and
   convergence rules where applicable.
3. Implement with owned Rust values and explicit model state; reuse established
   numerical libraries where appropriate.
4. Validate representative fixtures against **at least one trusted independent
   statistical reference**, in addition to Python TabDat when useful. Compare
   actual sample membership, not just N. Check coefficients, covariance, SEs,
   test statistics, intervals and predictions as applicable.
5. Diagnose disagreements in sample/design → parameterization → estimation →
   covariance/inference → numerics → rendering order. Never widen tolerances to
   mask unexplained differences.
6. Record relevant versions, fixtures, tolerances and remaining limitations in
   focused tests/notes; avoid a large document suite for each estimator.

Use independent review for novel estimators or delicate inferential changes when
available. Do not claim validated behavior from parser coverage, Cargo success,
or Python/Rust agreement alone. See [current scope](../../../SPEC.md) and
[AGENTS.md](../../../AGENTS.md).
