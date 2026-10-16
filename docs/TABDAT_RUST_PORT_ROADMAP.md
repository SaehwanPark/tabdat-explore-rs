# TabDat Explore Rust: outcome-oriented roadmap

This is a revisable priority list, not mandatory phase gates or a promise of full
Python/Stata parity. [SPEC.md](../SPEC.md) summarizes current supported behavior;
source/tests are authoritative. Completed-slice details are retained in the
[archived delivery ledger](archive/delivered-workflows.md), not repeated here.

## How to select work

Choose one small, complete user-visible improvement based on utility, correctness
risk and dependency order. State the behavior, 2–5 acceptance cases, important
failure mode, exclusions and verification before implementation. Prefer finishing
load → inspect/transform → fit → predict/report/persist to adding parser-only
estimator families. Existing syntax is not an instruction to implement it.

## Now: make the core workflow dependable

The installed baseline includes local CSV/Parquet, a persistent edited shell,
readable and bounded JSON results, scripts, core EDA/transforms and linear
analysis. Improve the practical gaps within that workflow rather than growing a
command catalog.

### Core EDA and data preparation

**Outcome:** users can inspect, clean, combine and persist representative small
local datasets through the real CLI/REPL.

- Keep load/path handling, previews, summaries, frequencies and supported
  transformations discoverable and recoverable.
- Preserve missingness, native schema, ordering, labels and named-table ownership
  where promised; make unsupported options fail clearly.
- Keep ordinary finite summaries distinct from native grouped means and SQL-NULL
  missingness; do not silently reinterpret samples.

**Evidence:** focused CSV/Parquet end-to-end fixtures and failure/state tests for
changed behavior. Do not add general rollback or lossless-roundtrip claims.

### Trustworthy linear analysis

**Outcome:** supported regress → diagnostics/inference → predict → persistence
workflows have explicit statistical and retained-model semantics.

- Make supported estimator/covariance/weight options and recovery paths clear.
- Preserve selected-cohort membership, predictor/order/intercept conventions,
  covariance corrections, degrees of freedom and post-estimation state.
- Keep current-data prediction distinct from retained-fit inference; do not add
  automatic refits or guessed staleness/eligibility guarantees.

**Evidence:** changed numerical behavior requires an independent trusted reference
with exact sample/design checks and justified tolerances. Reporting-only changes
reuse owned results and existing reference fixtures.

## Next: reduce friction and automate reliably

- **Help and discovery:** teach executed workflows and distinguish parser-reference
  syntax from runtime support. Keep the README short; put detailed contracts in
  [the reference](reference/command-results.md) and command help.
- **Structured results:** extend versioned output for concrete executed workflows,
  not the entire speculative command catalog. Preserve stable envelopes and
  meaningful source-classified errors.
- **Interactive ergonomics:** extend history/completion/display only for demonstrated
  daily-use gaps. Current completion covers commands/help, local read paths, named
  relation targets and bare/backtick EDA variables; expressions/options/output
  paths, persistent history and multiline continuation remain deferred.
- **Script reliability:** protect ordered effects, nested paths, source locations,
  output-failure stopping and the documented context-only seed policy.
- **Responsiveness:** use the existing [measurement harness](performance/core-workflow.md)
  and comparable profiles before optimizing. Local observations are not latency
  thresholds or platform certification.

## Later: optional breadth

Add an estimator/backend as a complete **parse → execute → validate → report**
slice only for concrete demand. GLM/count, IV/panel, causal, regularization,
spatial/Bayesian, plotting, ReadStat/libgretl and MCP transport remain optional
breadth, not prerequisites for a useful core release. Prototype feasibility is
not production integration. Evaluate packaging/licensing for named platforms.

## Verification and scope boundaries

Follow [AGENTS.md](../AGENTS.md) and [CONTRIBUTING.md](../CONTRIBUTING.md):

| Change | Evidence |
| --- | --- |
| Docs or low-risk refactor | Diff and relevant checks |
| Grammar, script, CLI | Supported acceptance/error regressions |
| Data, session or file effects | Invariants, failure state and representative end-to-end workflow |
| Numerical inference | Independent reference, samples/design and justified tolerances |
| Native/FFI or distribution | Ownership, lifecycle, licensing and platform-specific checks |

Do not reinstate exhaustive historical parity gates, routine repeated reviews,
unrelated fuzz campaigns, speculative architecture or exact error-wording parity.
Commit/push/PR/merge only within current user authorization. Update this roadmap
when priorities change and SPEC when capabilities change; record detailed evidence
in tests, PRs or the relevant subsystem documents rather than growing this ledger.
