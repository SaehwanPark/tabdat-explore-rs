# Review: bounded linear-regression VIF runtime

## Review record

- **Producer / consumer:** task owner and focused code reviewer / merge decision-maker.
- **Rust base:** implementation commit `7eb6ea4d400c53e83c9940511b7644ec75dacc9d` plus the documentation-closeout diff on `feat/runtime-vif`.
- **Scope:** parser subcommand, statistics helper, retained regression design, runtime dispatch/result, focused tests, and claims in `01-contract.md`, `02-evidence-migration.md`, `SPEC.md`, `ARCHITECTURE.md`, and the Phase 7 roadmap.
- **Review type:** contract-driven owner review using the `code-reviewer` checklist; review was performed against the pinned executor and the current code/callers/tests.
- **Outcome:** **partial**, by design; no actionable issues found in the implemented full-rank runtime boundary. The implementation head's hosted Rust baseline, runtime-boundary, dependency/unsafe-policy, and configured feasibility workflows passed; the final documentation head is rechecked before merge.

## Findings

**No actionable issues found** within the selected full-rank VIF slice.

## Reviewed invariants

- Only the new `Vif` subcommand dispatches; other `estat` forms retain their explicit unsupported runtime behavior. Parser option rejection and case/quote handling remain covered.
- Runtime prerequisite order is active dataset, then prior regression. VIF is read-only and does not access DuckDB, change model state, or publish a relation.
- Regression stores the already-filtered ordered predictor design only after a successful fit. Failed fits leave prior model state intact; later active-relation projections do not alter the fitted design used by VIF.
- Each auxiliary fit uses the same intercept convention and unit weights. WLS/GLS observation weights affect the retained sample but are not applied to the auxiliary VIF regressions, matching the oracle's `model.exog` call.
- Table headers, predictor order, nullable values, infinity preservation, and mean-row presence follow the recovered Python contract. Auxiliary statistics failures normalize to the pinned generic runtime error.
- The helper validates its input dimensions, returns optional values for undefined R², preserves infinity for exact auxiliary dependence, and adds no backend/dependency boundary or unsafe code.
- Finite VIF values are checked against statsmodels 0.14.6 and independent closed-form values. No-intercept, one-predictor, OLS/WLS/GLS, missing-weight sample selection, and post-fit projection are covered.

## Residual scope / validation

- Exact-collinear end-to-end VIF is not supported because the pre-existing Rust regression rejects a rank-deficient main design. The contract, evidence, SPEC, architecture, and roadmap preserve this parity gap; the broad roadmap item remains unchecked.
- CLI/JSON/MCP rendering and other estimator families or `estat` subcommands are not covered.
- Local locked format/check/Clippy and focused Rust tests pass. The Windows full-workspace test run reaches unrelated existing `export_contract` failures caused by backslash command parsing. Hosted Linux Rust baseline, runtime-boundary, dependency/unsafe-policy, and configured feasibility workflows passed at implementation head `7eb6ea4`; rerun all final PR-head checks after this documentation closeout.
