# Household mixed-fit named comparison: implementation contract

**Status: preimplementation; no new execution capability yet. Risk: high**
(public command/result contracts and immutable reporting state).

## Exact illustration and outcome

Chapter24, Table24.4 p.851, H-REPORT-MIXED in the [source inventory](inventory.md),
May2005 companion program lines139–140: compare `pa fe refgls remle`. Deliver a
native four-fit reporting workflow, not literal Stata formatting or unexplained
printed ML SE parity. The existing GEE, FE, equal-group-between GLS and marginal
Gaussian ML workflows supply fitting and independent numerical references.

Use the pinned ASCII and separately precise DTA preparations: numeric
`lnexp12m ~ 1 + lnhhexp + age + sex + hhsize + farm + educ`, commune groups,
source-row ordering/key audit, exclude NULL outcomes only after raw-key and
finite-predictor validation. Original5999/selected5006,194 communes, sizes1..39,
one singleton. Do not treat source-row keys as calendar time or factor-expand
numeric FARM/EDUC/SEX. No weights, offsets or survey interpretation.

## Observable reporting and compatibility decisions

- Reuse immutable `EstimateSnapshot`, existing named registry/table and CLI-v1
  results. No fit restoration, new estimator framework or production dependency.
- Add reporting support only for model-based FE with available classical
  normalized-level inference, model-based native RE-GLS/ML and explicitly chosen
  Gaussian GEE covariance. Other currently supported snapshot families stay
  unchanged; panel cluster/robust snapshot expansion is outside this slice.
- GEE has **no ambiguous covariance default**. Add
  `estimates store pa, covariance(working)` and `covariance(sandwich)`; plain store
  on GEE must explain the required choice. The selector is GEE-only; other fits
  reject it without registry/model/data mutation. Preserve existing typed/wire
  `EstimatesCommand::Store(String)`; add a concrete selector-bearing variant.
- Snapshot GEE working or uncorrected unit-sandwich coefficients/full covariance,
  normal inference, and label the selection explicitly. Never silently copy one
  while labelling the other. Sandwich uses commune clusters with multiplier1,
  not G−1 Student t or CR1.
- FE uses existing native normalized common-intercept/slopes covariance and
  slope coefficient inference, Student t(N−G−p). Evaluate only its existing
  retained common-intercept linear combination as native FE reporting does;
  never refit or reconstruct from current data. Canonicalize this normalized
  `_cons` to `intercept` for comparison with the other models, keeping ordered
  six slopes. Reject ambiguous names or unavailable normalized inference rather
  than inventing a common-intercept SE.
- RE copies the selected native beta coefficients/covariance and normal
  inference, not transformed-OLS Student t. Label Swamy–Arora versus explicit
  equal-group-between versus Gaussian marginal ML. Source `refgls` uses explicit
  equal-group-between. ML copies actual marginal log likelihood only, with an
  explicit convention; GLS/GEE/FE do not acquire pseudo likelihoods.
- Existing generic `r_squared` remains OLS-labelled. All four new snapshot
  families leave it unavailable; do not substitute FE within/conditional R²,
  quasi-GLS R², pseudo-R² or squared correlation. No common-R² claim.
- Add optional panel provenance on snapshots (absent/omitted for old snapshots):
  entity variable, optional explicit time/key variable, group count and common-
  intercept convention. GEE has no time variable; FE uses observation-weighted
  zero-effect normalization, the other models population-mean intercepts.
  Existing sample provenance remains candidate positions, not a cross-dataset
  identity/fingerprint or a promise that differing fits share the same cohort.
- Comparison is a first-seen parameter union in requested model order, with
  unavailable cells, explicit covariance/reference/DF/N and panel provenance.
  Preserve existing table fields/format when no panel snapshot is present.
  Source script requests exactly `pa fe refgls remle`; no artificial model order.

## Acceptance cases and failure boundary

1. Actual script on pinned ASCII/precise DTA and small balanced/unbalanced controls
   fits/stores all four models, reports the requested order/full matrices and
   correct normal-versus-FE-t inference. Independently audit every selected
   coefficient/SE/statistic/p/covariance, N/sample, FE normalization and ML
   likelihood against the existing R QR/GEE/marginal-ML references. Source
   rounding gates are separate; known ML SE discrepancies remain explicit.
2. Both GEE selectors faithfully snapshot their own native matrices/inference.
   Plain GEE store and selectors on non-GEE fits fail atomically with actionable
   errors. Invalid/duplicate options and malformed typed requests fail before
   effects. Legacy plain store parsing and wire shape remain supported.
3. Named snapshots survive current-data replacement, later model fits and missing
   active data. Table/store/drop must not refit, alter retained predictions,
   labels or data. Source workflow ends with RE-ML-owned population predictions;
   JSON/text exports are byte-identical and existing owners are not clobbered.
4. Old snapshot serde defaults omit panel provenance; new snapshots roundtrip
   losslessly. Verify missing FE normalized inference, ambiguous parameter names,
   missing/inconsistent sample/model provenance, registry capacity and unsupported
   covariance failures leave registry and fit/data unchanged.
5. JSON/text/help/discovery agree on installed forms, inference and unavailable
   statistics. Regression checks preserve OLS/binary/Poisson/NLS named reporting.

Fixed tolerances inherit independently justified existing references (typically
absolute1e-9, nonzero-p relative1e-8, exact source fields/keys/order/membership).
Investigate discrepancies before deciding tolerances; no widening for convenience.
Choose synthetic inputs that genuinely identify all four fits and verify them
with independent R before freezing controls; stop if this assumption fails.

## Ordered implementation and verification scope

1. Inspect direct consumers of `EstimatesCommand`/`EstimateSnapshot`, existing
   native FE common-intercept reporting and R reference layouts. Confirm source
   and suitable synthetic four-fit cohorts before implementing schema changes.
2. Extend domain/parser with the additive GEE selector and optional panel
   provenance; adapt execution snapshots and conditional reporting rows. Likely
   files: `crates/tabdat-{domain,language,execution,reporting}/src/estimates*`,
   language command parsing, reporting help/catalog and direct match consumers.
3. Add executable `household_mixed_report.td`, source/reference auditor, frozen
   controls and focused parser/domain/runtime/CLI actual-script tests; expand
   to affected crates and reverse consumers for shared API changes. Explicitly
   select all new/modified tests before broader testing. No empty selections.
4. Reconcile SPEC/roadmap/ledger/inventory, document evidence in the draft PR,
   review final diff/checks, then merge and safely prune only the complete slice.

Run focused named tests first (`cargo nextest run --locked -p <package>
--test <target> --no-tests fail`), then affected packages/integration consumers,
relevant Clippy, fmt and diff checks. Workspace all-target Clippy currently has
an unrelated baseline excessive-precision literal in
`tests/mma05_table57_comparison_workflow.rs:54`; do not change that fixture merely
to make a broad check green. Expand checks proportionally; no new Rust doc examples
means no automatic doctest rerun requirement.

No broader covariance-selector language, arbitrary panel/named-model testing,
restoration, printed ML SE repair, bootstrap/GEE/survey expansion or whole-table/
chapter completion. Keep unresolved required inventory entries required. Stop
on contradictory inference/reference semantics, essential missing data/reference
or a required product-scope decision rather than improvising.

Implement exactly this plan. Do not broaden scope. If the plan conflicts with
the codebase, stop and report the conflict instead of improvising. Report files
changed, tests run, deviations and unresolved risks in the PR/handoff.
