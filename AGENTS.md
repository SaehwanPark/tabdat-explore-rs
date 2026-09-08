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

## Roadmap development loop

- One loop is one bounded target slice, one meaningful development unit, and one
  PR. Recover branch/PR state before selecting the next unmet roadmap gate.
- Use two spaces for indentation throughout (no hard tabs), except formats that
  require otherwise. Configure editors and formatters rather than relying on memory.
- Use functional-first design: pure domain transformations, explicit state and
  typed failures, effects at adapter boundaries; avoid speculative abstractions.
- Use specification-driven development: record scope and observable acceptance
  before implementation; keep specs, architecture, and history aligned with evidence.
- Use TDD for behavior changes: add a focused test, observe the intended failure,
  implement the smallest correct change, then rerun checks. For documentation and
  configuration, validate their actual contracts rather than inventing product tests.
- Delegate bounded research, implementation, or independent review when useful;
  retain one integration owner and serialize overlapping writes. Unavailable agents
  are a reason to work serially, not to invent review results.
- While executing the user's active autonomous roadmap request, commits, pushes,
  PR creation, and merging into `main` are authorized for its bounded slices only.
  This is not standing permission for unrelated tasks or later sessions without
  that request; a newer user restriction (such as local-only work) takes precedence.
  Before merge, inspect the diff, resolve blocking review
  findings, and require applicable local checks and hosted CI to pass on the latest
  revision. Absence of CI is not evidence of a passing CI run: record the bootstrap
  exception only for the initial guidance PR (#1), and establish CI in the next
  setup slice. Do not reuse that exception for subsequent documentation PRs.
- After merge, update the clean local `main`, choose the next slice, and repeat.
  Do not force-push, bypass protections, silently waive failed checks, or mark the
  whole roadmap complete after a single slice. Stop with evidence and needed input
  when access, authority, missing contracts, or unexplained failures block progress.

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
