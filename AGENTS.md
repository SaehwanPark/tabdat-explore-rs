# Working on TabDat Explore Rust

## Purpose and current state

TabDat is a terminal-first statistical/EDA environment with a Stata-inspired
language. This repository is its Rust-native successor. Today it contains a
single Rust 2024 binary scaffold (`Cargo.toml`, `src/main.rs`), not a working
TabDat CLI. Do not describe proposed commands, crates, backends, or CI as present.

Read the relevant sections of:
- [Project proposal](docs/TABDAT_RUST_PORT_PROJECT_PROPOSAL.md): product intent,
  architecture, migration method, and constraints.
- [Roadmap](docs/TABDAT_RUST_PORT_ROADMAP.md): phased work and acceptance gates.

Both documents are proposed plans. Source/tests establish implemented behavior;
unchecked tasks are not evidence of support. No Python oracle location/revision
is recorded yet. Recover and pin it before claiming migration parity; record
conflicts or intentional deviations rather than silently choosing a behavior.

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
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

These do not establish command or statistical parity. Differential/reference
fixtures, CI, security tooling configuration, and performance harnesses are
roadmap work; do not claim they ran merely because Cargo passed. For documentation
changes, validate links, skill frontmatter, scenario coverage, and `git diff --check`.
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
