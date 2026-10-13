# TabDat Explore Rust: outcome-oriented roadmap

**Working plan, revised 2026-10-13.** Priorities are adjustable; this is not a
mandatory phase order or a commitment to full Python parity. Current support is
summarized in [SPEC.md](../SPEC.md). The historical checklist and recorded
acceptance decisions remain available in the [previous roadmap](https://github.com/SaehwanPark/tabdat-explore-rs/blob/333b97dd55524dbf5ce55fac510375df99e4649e/docs/TABDAT_RUST_PORT_ROADMAP.md).

## How work is selected

Choose the smallest vertical feature that improves a real workflow. Prioritize
user utility, correctness risk and dependency order, not the number of parser
forms migrated. It is acceptable to leave rare Python grammar/diagnostic cases
unsupported. Never waive statistical reference checks to speed delivery.

For each outcome, record only: **user behavior**, **2-5 focused acceptance cases**,
**important failure mode**, **what remains out of scope**, and **verification**.
No phase-wide checklist, repeated owner-review ritual or large handoff bundle is
required. Work independently of unrelated pending items.

## Now: finish one coherent core workflow

### 0. Resolve the ongoing script PR

**State:** [PR #195](https://github.com/SaehwanPark/tabdat-explore-rs/pull/195)
is a draft on the snapshot date; the initial script differential work has five
seed-integer mismatches.

- Decide and document a **useful seed contract** (supported format/range and,
  where relevant, deterministic RNG semantics); unrestricted Python integers
  are not an automatic requirement.
- Preserve ordinary canonical `.td` behavior, nested inclusion, ordering,
  macro/condition state and meaningful path/line diagnostics.
- Keep differential evidence without silently changing oracle expectations.
  Approved deviations get tests and a short migration note.
- Run script-focused and applicable workspace checks. Merge only if authorized
  and the selected acceptance criteria pass.

### 1. Deliver a minimal interactive shell

**Goal:** `tabdat` starts a usable session for an analyst, not a greeting.

- Interactive input, command execution, user-readable errors, and clean exit.
- Basic output for core inspection; simple command history if cheap.
- Handle EOF and interrupts safely without corrupting session state.
- Defer advanced completion, highlighting and rich editor behavior until usage
  demonstrates value.

**Done when:** A short `use` → `describe` → `summarize` → `exit` interaction works
in one process with readable results and appropriate failure behavior.

### 2. Close the core EDA loop

**Goal:** common local-data workflows are functional through the actual CLI/REPL.

- `use`, `describe`, `head`/`tail`, `summarize`, `tabulate`, a practical subset of
  `keep`/`drop`, `generate`/`replace`, and save/export.
- Preserve/verify missingness, schema, ordering and failure-state guarantees for
  every newly supported option.
- Make unsupported options fail clearly; do not pre-parse every Python feature.

**Done when:** Canonical small CSV/Parquet fixtures run end to end and expected
rows, values, metadata and outputs match the declared supported semantics.

### 3. Make linear statistics reliably usable

**Goal:** a complete, scientifically trustworthy linear-model workflow.

- `regress` with explicitly supported covariance modes, then `predict`, `test`,
  `lincom` and VIF where implemented.
- Confirm sample membership, coefficient/order conventions, degrees of freedom,
  covariance corrections, inferential values and post-estimation state.
- Add readable result tables and structured results for the supported subset.

**Done when:** Representative model fixtures agree with at least one trusted
independent reference at justified tolerances; invalid inputs and stale model
state are handled correctly.

## Next: improve ergonomics and automate reliably

- Extend machine-readable output **for executed commands**, not the entire
  speculative command catalog; version changes to stable public envelopes.
- Stabilize script reproducibility and practical error reporting, including
  canonical `.td` workflows and documented seed semantics.
- Add small performance benchmarks (cold start, simple command, representative
  data access, model fit). Optimize only measured regressions.
- Improve history/completion and display as needed for daily use.
- Add plotting or MCP transport only when a concrete workflow requires them.

## Later: optional estimator and backend breadth

Add estimators one at a time as **parse → execute → validate → report** slices.
Choose among GLM/count, IV/panel, causal, regularization, spatial/Bayesian, or
native backend integrations by actual demand and implementation cost. Syntax-only
coverage is not a milestone by itself. ReadStat/libgretl/plotting prototypes are
feasibility work until explicitly integrated and validated. Packaging/release
support must be evaluated for named platforms and dependencies.

## Always-on checks (risk-weighted)

| Risk | Minimum evidence |
| --- | --- |
| Routine docs/refactor | Diff, relevant local checks |
| Grammar, script or interface | Supported acceptance/diagnostic regressions |
| Data and file mutation | Data integrity, failure atomicity, lifecycle checks |
| Numerical inference | Sample/design + independent statistical reference |
| Native/FFI and distribution | Memory ownership, error/teardown, license/platform validation |

The root formatting, locked check/test, and Clippy commands are in
[CONTRIBUTING.md](../CONTRIBUTING.md). Run focused tests during iteration;
run the relevant full checks before sharing a completed code change. No need
to fuzz unrelated code or perform repetitive self-reviews by rote.

## Deliberate non-goals

- Exact Python parity for arbitrary error strings, whitespace and extremely
  large expressions/seeds unless they affect supported workflows.
- Full Stata/Python/R feature replacement.
- Building future backends, plugins, MCP resources or crates merely because
  they appear in an old proposal.
- Requiring entire early migration phases to be checked off before delivering
  useful runtime features.

## Keeping this roadmap useful

Maintain at most a handful of near-term outcomes. Replace completed outcomes
with the next highest-value work; link substantive test evidence rather than
adding PR-by-PR historical prose. Preserve old details in Git history and
existing evidence documents. Update [SPEC.md](../SPEC.md) when actual capability
changes, not merely when a syntax enum grows.
