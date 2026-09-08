# TabDat skill routing and handoffs

## Design and source inventory

This is a small **expert pool**, normally used by one engineer/agent. Select only
skills needed for a bounded task; there is no required worker service, model,
plugin, or runtime adapter. The repository began with a Rust binary scaffold and
two proposed planning documents. Repo guidance is now present, and the initial
Python oracle revision is recorded in [docs/migration/](../../migration/); individual
slices still require their own contract and comparison evidence. The proposed crate
tree is not an installed architecture.

[AGENTS.md](../../../AGENTS.md) contains repo-wide rules and checks. The
[proposal](../../TABDAT_RUST_PORT_PROJECT_PROPOSAL.md) supplies domain intent; the
[roadmap](../../TABDAT_RUST_PORT_ROADMAP.md) supplies phased gates. Skills translate
those documents into repeatable work, not a second feature-status tracker.
Update guidance when its linked contracts change; do not copy the roadmap here.

## Routing

| Request | Primary skill | Add only when needed |
| --- | --- | --- |
| Recover/port syntax, scripts, commands, CLI/JSON/MCP or state behavior | [Migration](../../../.agents/skills/tabdat-migration/SKILL.md) | Data for relation effects; statistics for estimator semantics |
| DuckDB load/transform/query/export, lazy relations or metadata | [Data semantics](../../../.agents/skills/tabdat-data-semantics/SKILL.md) | Migration for unresolved public contracts; native for I/O adapters |
| Estimator, covariance, inference, prediction or reference comparison | [Statistical validation](../../../.agents/skills/tabdat-statistical-validation/SKILL.md) | Migration for command surface; native for backend integration |
| Native feasibility, FFI, dependency/linkage, packaging or initialization costs | [Native backends](../../../.agents/skills/tabdat-native-backends/SKILL.md) | Statistics for numerical claims; data for relation/label conversion |

For ambiguous tasks, start with migration contract recovery; route again once the
owned boundary is clear. Do not load every skill or create every proposed crate.

## End-to-end flow and ownership

1. **Scope/contract:** the task owner inspects current truth, selects a roadmap
   slice and skills, and records acceptance checks, sources, and missing inputs.
   Output: bounded contract (including Python/Rust/test/mapping sections for ports).
2. **Specialist work:** the same owner implements or investigates the selected
   boundary. Output: changed code/tests or feasibility/validation evidence, not a
   claim that an untested capability works.
3. **Review/verification:** inspect against the original contract, run applicable
   checks, and explicitly review numerical and unsafe changes. Output: findings,
   commands/results, supported scope, and unresolved gaps. Re-run affected checks
   after each correction; if an unexplained failure remains, report it rather than
   indefinitely changing implementation or acceptance tolerances.
4. **Synthesis:** the task owner reconciles evidence, updates only proven roadmap
   items/decisions, and reports `complete`, `partial`, or `blocked`. Output: final
   summary plus authorized commit/push/PR status. The active roadmap loop permits
   automatic merge only under the review/check gates in AGENTS.md; no automatic release.

Delegation is optional and shallow (owner → worker). Use it only for independent
research/review or clearly isolated implementation. The owner retains integration
and acceptance. Each assignment names inputs, selected skills, output, read/write
paths, acceptance checks, and required permissions. Workers default to read-only;
shell, network, repository writes, and spawning require explicit task/runtime
permission. No role-specific model is required; use the caller's available model.

Prefer isolated worktrees for concurrent writes; otherwise declare non-overlapping
paths and serialize shared files, test datasets, native build directories, and
stateful services. These ownership rules are advisory unless the runtime enforces
them; never claim exclusivity merely because this document requests it.

If worker spawn/model/tool/permissions/workspace setup is unavailable, continue
serially only with available authority, or mark the task blocked. On resource
conflicts, pause and serialize. On communication failure use the durable handoff
below. Missing worker branches/results or partial failures remain visible gaps:
preserve usable evidence and never synthesize an invented pass. Escalate contract,
statistical, safety, and license decisions to the task owner/user with the needed
input; workers do not silently relax constraints.

## Durable artifact contract

Use an in-thread summary for short tasks. When resuming or transferring work needs
files, create only the required artifacts under `_workspace/<task-slug>/`:

| Path | Producer → consumer | Required sections |
| --- | --- | --- |
| `01-contract.md` | Task owner → implementer/reviewer | Scope, acceptance, source revisions, selected skills, input gaps; Python/Rust/test/mapping contracts for ports |
| `02-evidence-<boundary>.md` | Specialist → reviewer/owner | Boundary, changed paths, inputs/versions, method/commands, raw-result locations, expected/observed results, deviations, blockers |
| `03-review.md` | Reviewer (or explicit owner review) → owner | Original acceptance checks, evidence reviewed, findings by severity, required fixes, unchecked scope |
| `04-summary.md` | Owner → next maintainer/user | Outcome, implemented vs validated scope, checks, decisions, remaining work, handoff status |

`<boundary>` is `migration`, `data`, `statistics`, or `native`. Each file identifies
producer, consumer, Rust revision plus working-tree changes, and `complete`,
`partial`, or `blocked` state. Specialist-required report sections belong inside
its evidence file. One writer owns each artifact. Attach large machine outputs
by relative path with provenance; do not overwrite failed comparison evidence.
Keep fixtures synthetic or authorized, and do not commit private data, credentials,
large native artifacts, or scratch reports by default. Durable accepted decisions
belong in project docs/ADRs; reusable regression fixtures belong with tests when
introduced. `_workspace/` is a convention, not an existing tool or ignored directory.

## Guidance regression scenarios

When changing this harness, check paths/frontmatter and walk through at least a
normal task and a blocked task. Expected routes and guardrails:

- **Parser slice with a pinned oracle:** migration → recover grammar/errors →
  typed syntax-only implementation → unit/differential evidence; no DuckDB needed.
- **`generate` mutation on a lazy relation:** data (+ migration for contract gaps)
  → state table → eager/lazy and failure-atomicity tests; do not sort away ordering
  differences or force a scan for metadata.
- **Clustered `regress` mismatch:** statistics → compare sample and covariance
  corrections → retain three-way outputs; do not widen tolerances to get green.
- **ReadStat callback spike:** native + data → ownership/panic-boundary review and
  label/missing-value fixtures; local success does not establish Linux support.
- **Missing Python revision/reference:** record exactly what is absent; a prototype
  may be partial, but parity/reference validation cannot be complete.
- **Unavailable worker or shared-file conflict:** serialize under one owner or
  report blocked; preserve partial evidence and do not invent delegated coverage.

These are instruction walkthroughs, not executable product tests. Cargo passing
on the scaffold does not satisfy any of the product scenarios above.
