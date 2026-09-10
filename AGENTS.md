# Working on TabDat Explore Rust

## Purpose and current state

TabDat is a terminal-first statistical/EDA environment with a Stata-inspired
language. This repository is its Rust-native successor. Today it contains a
single Rust 2024 binary scaffold (`Cargo.toml`, `src/main.rs`), not a working
TabDat CLI. Build tooling and the scaffold smoke test are described in
[SPEC.md](SPEC.md) and [CONTRIBUTING.md](CONTRIBUTING.md). Do not describe proposed
commands, crates, or backends as present.

Read the relevant sections of:
- [Project proposal](docs/TABDAT_RUST_PORT_PROJECT_PROPOSAL.md): product intent,
  architecture, migration method, and constraints.
- [Architecture](ARCHITECTURE.md): current-versus-proposed Rust boundaries,
  ownership, state/effect rules, and explicit deferrals.
- [Roadmap](docs/TABDAT_RUST_PORT_ROADMAP.md): phased work and acceptance gates.

Both documents are proposed plans. Source/tests establish implemented behavior;
unchecked tasks are not evidence of support. The initial Python oracle revision and
authority policy are recorded in [docs/migration/](docs/migration/); recover the
pinned checkout and slice-specific evidence before claiming migration parity.
Record conflicts or intentional deviations rather than silently choosing a behavior.

## Engineering boundaries

- Preserve public behavior, not Python module layouts. Use typed commands,
  results, errors, and explicit session/model state.
- Keep language → execution → backend dependencies directional; domain types
  must not depend on runtime backends. Human/JSON/MCP surfaces share typed results.
- DuckDB is the planned canonical data engine. A second engine needs benchmark
  evidence and a recorded decision. Initialize specialized capabilities lazily.
- Core distribution must need neither Python nor R; external validation tools
  are separate from runtime dependencies.
- Use safe Rust by default and `#![forbid(unsafe_code)]` in ordinary new crates.
  Confine approved FFI unsafety to low-level adapters with safe facades, owned
  results, RAII, and adjacent `// SAFETY:` reasoning. No raw foreign ownership
  or native model types in domain/application APIs.
- Statistical parity needs trusted references as well as the Python oracle.
  Never hide disagreements by silently widening tolerances.
- Record major backend, FFI, packaging, or dependency decisions in ADRs (create
  `docs/adr/` when the first decision is needed). Review native dependency license
  compatibility and redistribution obligations against the proposal's AGPL intent.

## Checks and change discipline

From the repository root, the baseline Rust checks are:

```sh
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
```

These do not establish command or statistical parity. Differential/reference
fixtures and performance harnesses remain roadmap work; do not claim they ran merely
because Cargo passed. The pinned dependency/advisory/unsafe-code commands in
`CONTRIBUTING.md` are separate policy checks and cover only the Rust workspace.
For documentation changes, validate links, skill frontmatter, scenario coverage,
and `git diff --check`.
Update only roadmap items supported by evidence, not entire phases by inference.
Preserve unrelated work. Commit/push only when authorized; do not force-push.

## Repo-local skills

Use [the routing and handoff guide](docs/harness/tabdat/team-spec.md) to select
only the relevant skills:

- [tabdat-migration](.agents/skills/tabdat-migration/SKILL.md): recover a behavior
  contract and port one bounded language/runtime/interface slice.
- [tabdat-data-semantics](.agents/skills/tabdat-data-semantics/SKILL.md): DuckDB
  relations, transformations, missingness, labels, and session invariants.
- [tabdat-statistical-validation](.agents/skills/tabdat-statistical-validation/SKILL.md):
  estimator semantics, inference, post-estimation, and reference evidence.
- [tabdat-native-backends](.agents/skills/tabdat-native-backends/SKILL.md): feasibility
  spikes, FFI safety, native dependency decisions, and capability costs.

Skills are portable markdown; no particular agent runtime or delegation is required.

## Subagents

Use subagents proactively to reduce main-context growth.

* Delegate bounded, self-contained investigation or implementation tasks when the parent mainly needs the result, not the working process.
* Prefer subagents for work that requires reading many files, logs, tests, documentation, or other large intermediate context.
* Give subagents only the context and scope needed for their task; avoid copying the full parent conversation unless necessary.
* Ask subagents to return concise findings, evidence/references, risks, and recommended actions rather than raw working context.
* Keep architectural decisions, cross-component integration, and final verification with the parent agent.
* Avoid redundant subagents inspecting the same scope unless independent review is intentional.
* If a subagent's scope expands substantially, it should escalate back to the parent rather than absorbing unrelated work.
* Use the main context for decisions; use subagent contexts for discovery.

See `docs/subagents_policy.md` for detailed delegation patterns and guidance.

## Asynchronous GitHub Communication

Use GitHub proactively as the durable communication channel when human collaborators are unavailable or work may continue across sessions.

* Prefer remote branches, commits, PRs, and GitHub discussions/comments over keeping important state only in local context.
* Push meaningful work to a remote branch regularly when it is safe and useful to preserve progress.
* Open a draft PR early for non-trivial work when it provides a useful place for status, design notes, review, and human steering.
* Keep PR descriptions and comments updated with current status, key decisions, unresolved questions, risks, and next steps.
* Use commits and PRs to leave a durable trail that another human or agent can resume without reconstructing the full conversation.
* When blocked on a human decision, record the question and relevant context in the PR or issue rather than leaving it only in transient agent context.
* Prefer small, reviewable commits and branches with clear scope.
* Do not merge, close, force-push shared work, or perform other irreversible repository actions unless explicitly authorized or clearly permitted by project policy.
* Never commit secrets, credentials, private data, or machine-specific sensitive artifacts.

Use local context for active reasoning; use GitHub for durable project state and asynchronous human communication.

## Agentic Loop

Use agentic loops for long-running tasks or when pursuing goals.

One loop is defined by

1. Select target slice (what to implement/examine/do)
2. Design a plan
3. Execute the plan
4. Test and verify
5. Update documents if necessary
6. PR handoff and merge autonomously
7. Move on to the next task or slice
