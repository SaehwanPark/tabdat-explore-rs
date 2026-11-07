# Household mixed-fit named comparison: implementation contract

**Status: native selector/snapshot adapters implemented; complete source workflow,
auditor, help/status reconciliation and broader validation pending. Risk: high**
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
  `_cons` and GEE's `_cons` to `intercept` for comparison with RE, keeping ordered
  six slopes and the corresponding matrix order. Reject ambiguous names or
  unavailable normalized inference rather than inventing a common-intercept SE.
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

## Implementation checkpoint

The additive typed/parser GEE selector, optional four-field panel provenance,
model-based FE/RE and selected-GEE snapshot adapters and conditional text rows
are implemented. Legacy plain-store wire and non-panel snapshot omissions are
preserved. Domain3/parser3/execution2 focused tests and runtime11 named-report
tests pass (including the two new panel tests). The new snapshots pass1218
frozen independent R gates across all three controls: both GEE selections,
normalized FE full covariance/inference, GLS and ML beta inference/matrices and
actual ML marginal likelihood. Private tests reject24 missing/malformed/weighted/
negative-variance variants atomically, exercise owned store without active data/
backend, and table after clearing the active model. Scoped all-target Clippy passes for domain, language,
execution, reporting and runtime; fmt/diff checks pass.

This is not the finished illustration: unified actual-script JSON/text exports,
original-source/forgery auditor, remaining failure/wire/help tests, affected-crate/
reverse-consumer suites and SPEC/ledger/help reconciliation remain required
before ready/merge. Do not count the kernel/adapter checkpoint as source workflow
completion. Printed ML SE discrepancies are still required and not repaired here.

## Feasibility checkpoint (before schema implementation)

Existing native scripts and all four independent R generators succeed on three
controls: balanced192 and unbalanced199/200-source with singleton (the existing
joint Hausman dyadics), plus179/180-source with16 groups and positive ML unit
variance (existing RE-interior input rounded to multiples of1/64).
An ad hoc native/reference audit passes496 numerical gates per control: GEE198,
FE103 (fit and common intercept, not separate linear restrictions), GLS97 and
ML98 (coefficient inference/full matrices, counts/scales and marginal likelihood).
Fixed absolute1e-9 and nonzero-p relative1e-8, exact-zero underflow; no widening.
These are existing-fit feasibility checks, not evidence for the not-yet-installed
named comparison. ML prediction exports also match R on all192/199/179 rows, with exact selected
source fields/order and absolute1e-9 prediction gates. New unified workflow,
original-publisher source and state/wire checks remain to implement. Frozen
controls and reproduction are in the [fixture README](../../tests/fixtures/mma24_household_mixed_report/README.md).

Both Hausman-derived controls have negative admissible GEE correlation and
zero-boundary ML unit variance; the third has GEE rho about.67688 and interior
ML unit variance about1.70115. Keep these distinct, rather than forcing GEE to
match an RE covariance or silently replacing boundary fits. GEE and FE both use
native `_cons` names, so both require explicit common-row canonicalization.
Optional R `geeM0.10.1` was installed in a dedicated validation library; set
`R_LIBS` when reproducing. R4.6.2; no production dependency change.

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
