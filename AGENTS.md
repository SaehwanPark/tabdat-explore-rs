# Working on TabDat Explore Rust

Build a small, fast, terminal-first statistical application for graduate
econometrics. The long-term analytical target is the empirical analyses in
Cameron and Trivedi, *Microeconometrics: Methods and Applications* (2005).
Deliver a usable load/prepare → specify → fit → diagnose/interpret → report
workflow for the selected analysis.

## Default development workflow

For repository development tasks, follow this workflow unless the user gives
different instructions. A question, read-only review or planning request does
not authorize changes.

1. Check the working tree, fetch current `main` and inspect open PRs. Resume
   matching work or branch from current `main` for one cohesive change. Preserve
   unrelated edits and other contributors' branches.
2. Open a draft PR before substantive implementation, or at the earliest
   practical point. A minimal initial commit is fine. Describe the change,
   rationale and verification approach.
3. Make routine technical decisions autonomously. Commit and push meaningful
   increments. Record decisions, changed scope, failures and verification in
   the PR when they help a reader understand or resume the work.
4. Inspect the diff and validate the affected contracts. When the slice is
   complete, mark the PR ready, satisfy required checks and approvals, merge to
   `main`, and remove its temporary branches after confirming the merge.

This workflow supplies standing authorization for those actions within the task.
Seek input for a consequential change of product meaning, an irreversible action
outside this workflow, or a blocker that requires the user. Continue independent
work while blocked. Honor repository protections and platform limits.

## Evidence and scope

- Start with the relevant source, tests and [capability map](SPEC.md). Use the
  [roadmap](docs/TABDAT_RUST_PORT_ROADMAP.md) for priorities and the
  [architecture](ARCHITECTURE.md) for crate boundaries. Consult historical
  material when it answers a specific question.
- For textbook work, select an illustration, sample/design, independent
  reference and failure case from the [coverage ledger](docs/reference/microeconometrics-coverage.md).
  Update its evidence alongside the workflow. Distinguish executable support,
  reference validation and outstanding coverage.
- Define statistical samples, missingness, design, covariance corrections,
  degrees of freedom and retained state. Check changed numerical results against
  an independent reference and explain tolerances.
- Preserve promised row order, types, labels, file and session state. Exercise
  failure paths for mutations. Preserve supported user workflows and documented
  machine contracts. See the [migration policy](docs/migration/README.md) for
  compatibility decisions.
- Prefer existing crates and safe Rust. Keep DuckDB as the canonical data
  engine, native unsafety local and documented, and production free of Python/R.
  Use two-space indentation, respect format conventions and avoid non-breaking
  spaces.

## Test execution policy

Inspect the branch diff, working tree and affected callers before selecting
tests. Run new or changed tests explicitly and check the selected count. Start
with the smallest relevant case, then cover affected crates and integration
boundaries in proportion to risk. Reuse passing evidence while the relevant
code and inputs are unchanged. Broaden checks for shared APIs, cross-cutting
changes or uncertain impact. See [local verification](CONTRIBUTING.md#impact-scoped-testing)
for runner commands and specialized checks.

Work in one context by default. Delegate a bounded independent question when
it saves time or brings relevant expertise. Use [specialist skills](docs/harness/tabdat/team-spec.md)
as needed. If a real limit interrupts work, use the draft PR for a concise
handoff with the remaining step.
