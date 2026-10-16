# Working on TabDat Explore Rust

## Mission and priority

Build a small, fast, dependable, terminal-first statistical/EDA application in
Rust that is practically usable for **graduate-level econometrics**. The minimum
long-term analytical target is the empirical analyses in Cameron and Trivedi,
*Microeconometrics: Methods and Applications* (2005). Current EDA/linear support
is a foundation, not completion of that target. Prefer **complete, usable
workflows** to syntax scaffolding or exact Python/Stata implementation parity.

The source code and tests define implemented behavior. [SPEC.md](SPEC.md) is the
short, current capability map; the [roadmap](docs/TABDAT_RUST_PORT_ROADMAP.md)
is a revisable priority list, not a sequence of mandatory phase gates. The
[architecture](ARCHITECTURE.md) explains boundaries. Historical proposal, ADRs,
old migration evidence, and Git history are references **when relevant**, not
required reading for every task.

## Non-negotiable engineering rules

- **Statistical trust:** Define estimation samples, missingness, formula/design
  semantics, covariance corrections, degrees of freedom, and model state
  explicitly. Validate changed numerical results against a trusted independent
  reference when appropriate; never make unexplained differences pass by widening
  tolerances.
- **Data integrity:** Preserve ordering, types, labels, and state where promised.
  Test failure paths for operations that mutate data, session state, or files.
- **Safe boundaries:** Prefer safe Rust; keep approved native/FFI unsafety local,
  documented, and out of domain/application APIs. DuckDB remains the canonical
  initial data engine. No production Python or R dependency.
- **Honest status:** Distinguish parser acceptance, executable behavior, tested
  behavior, and future intent. Do not claim whole-command parity, release
  readiness, or platform support from a passing unit test.
- **Compatibility by impact:** Preserve supported user workflows, data/statistical
  semantics, documented machine contracts, and meaningful errors. Exact Python
  wording, obscure lexical behavior, and unsupported features may diverge after
  a short, explicit decision; see [migration guidance](docs/migration/README.md).
- **Minimal architecture:** Reuse existing crates/modules. Introduce abstractions,
  dependencies, configuration, ADRs, or new crates only for a demonstrated need.

## Everyday development loop

1. Read the immediate request, relevant source/tests, and the short capability
   status. Select the **smallest end-to-end user-visible outcome** that fits.
2. Define a few observable acceptance cases and important failure cases. Do not
   build an exhaustive requirements matrix for a routine change.
3. Implement, run focused tests, then the applicable checks in
   [CONTRIBUTING.md](CONTRIBUTING.md). Keep test fixtures small and inspectable.
4. Check the diff, report what was and was not verified, and update the current
   specification/roadmap only if public capability or priority changed.
5. Commit, push, create PRs, or merge **only within the user's actual authorization**.
   A general instruction to continue developing is not a grant to merge or
   change unrelated work. Draft PRs are useful for a substantial WIP handoff,
   not required for every small change.

For straightforward changes, a concise PR explanation and focused tests are
sufficient. Do **not** require three self-review passes, a failed test observed
before implementation, four handoff documents, or full Python differential
replays by default. Escalate evidence proportional to risk:

| Change | Evidence expected |
| --- | --- |
| Docs, formatting, low-risk refactor | Diff and relevant checks |
| Parser, script, CLI, command dispatch | Focused acceptance, error, and regression tests |
| Data mutation, persistence, state | Invariants, failure/rollback, representative end-to-end case |
| Estimator, covariance, inference | Independent numerical/reference comparison and documented tolerances |
| Unsafe/FFI, native distribution | Ownership/safety, platform, licensing and specialized testing |

## Scope control

- Deliver complete load/prepare → specify → fit → diagnose/interpret → report
  workflows in the CLI/REPL, not syntax-only estimator families. Preserve the
  working EDA/linear core while advancing the roadmap's required graduate methods;
  do not indefinitely substitute editor/help polish for missing econometrics.
- Use the [coverage ledger](docs/reference/microeconometrics-coverage.md) when
  selecting textbook work. Specify one exact illustration, sample/design,
  reference and failure case before implementing; update its status with evidence.
  IV/panel, binary/count, resampling and later textbook methods are deferred
  required work, not optional breadth. Do not demand the full book inventory or
  every foundation feature before delivering one bounded analysis.
- Do not turn a rare Python edge case into a dependency or public API expansion
  without a real user/reproducibility need. Document supported limits clearly.
- Avoid speculative security hardening. Keep reproduced parser-crash regressions
  and sensible input bounds; run extended fuzz/sanitizer campaigns when parser or
  native risk warrants them, not for unrelated changes.
- Implement future features only when requested or prioritized. A checkbox in an
  old roadmap is not an instruction to implement it now.

## Agent operation and handoff

Work single-agent by default. Delegate only a bounded, independent task with a
clear benefit, returning conclusions and file/test evidence. See
[optional skill routing](docs/harness/tabdat/team-spec.md); select only relevant
skills. No particular harness, subagent topology, quota-monitoring tool, or
model route is required.

Use two-space indentation; preserve format-specific conventions. Keep functions
organized so dependencies precede callers where practical. Never add non-breaking
spaces to source or docs.

For AI subscription quota/budget policy, refer to [ai-usage-policy](docs/ai-usage-policy.md)
