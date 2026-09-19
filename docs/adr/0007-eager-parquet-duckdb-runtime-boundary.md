# ADR 0007: Eager local-Parquet runtime boundary

- Status: Accepted for bounded evaluation; broad production integration deferred
- Scope: `use <existing-local-parquet>` in eager mode only

## Context

The language layer now parses `use`; before this evaluation, the repository had
no session or data runtime. The isolated DuckDB feasibility prototype proves local Parquet reads,
ordered Arrow results, and owned connection teardown, but it explicitly defers a
domain-owned adapter, failure-atomic session state, thread/lifetime review,
packaging, and redistribution decisions.

## Decision

Accept a workspace `tabdat-runtime` library as a bounded evaluation. It owns a
session and keeps a DuckDB connection private to a safe adapter. The first
supported operation is only an existing local `.parquet` loaded eagerly from a
typed `tabdat-language::Command::Use`. The adapter stages and inspects the
relation, then replaces the active table transactionally; the session publishes
owned metadata only after success. DuckDB is initialized lazily when this
operation is requested, not when a session is constructed. Validation checks the
suffix before filesystem existence/type, matching the pinned Python resolver.
Home-directory `~` expansion is intentionally deferred to callers in this
bounded Rust API, as recorded in MIG-0003.

The dependency candidate is `duckdb-rs` `1.10505.0` with `bundled` and `parquet`
features, matching the isolated prototype. No raw handle, Arrow value, or
foreign lifetime may cross the runtime public API, and ordinary Rust crates
continue to use `#![forbid(unsafe_code)]`.

## Explicitly out of scope

CSV/DTA/Feather/Arrow, URI/network access, `~` expansion, lazy execution, named tables,
transformations, broad inspect execution beyond bounded `describe`/`count`/`head`/`tail`/`summarize`/`codebook`/`missing`/`duplicates`/`isid`/`datasignature`/`assert`,
labels, CLI/JSON/MCP surfaces, and a general relation API remain deferred. This slice must not mark the broad `use`
or Phase 4 data-runtime roadmap items complete.

## Acceptance evidence and disposition

- Success and failure-atomicity tests against a synthetic Parquet fixture,
  schema/order and row-count metadata checks, and parser-to-session coverage are
  present in `crates/tabdat-runtime/tests/use_contract.rs`.
- Local macOS Apple Silicon checks passed on Darwin arm64 with Rust 1.97.1;
  hosted Linux x86_64 locked check/test/Clippy evidence passed in the
  [runtime workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35232676906).
- The hosted [Rust baseline and dependency-policy run](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35232676933)
  passed. Its geiger summaries report `forbid(unsafe_code)` and zero first-party
  unsafe functions/expressions for the root, language, and runtime crates; the
  runtime transitive inventory is retained as an explicit warning signal.
- The adapter owns its `duckdb::Connection`, exposes no native handle, and does
  not add `Send`/`Sync` or cross-thread guarantees. `&mut self` provides the
  bounded exclusive session contract. Staging cleanup runs on every recoverable
  error, publication is transactional, and normal Rust drop handles connection
  teardown; no first-party unsafe or panic-catching boundary is introduced.
- `cargo deny` and `cargo audit -D warnings` pass. `deny.toml` explicitly records
  `CDLA-Permissive-2.0` for the bundled dependency subtree. The repository remains
  unpublished and adds no native redistribution/NOTICE package, so AGPL-oriented
  license and notice work remains a production-packaging prerequisite rather than
  an implicit approval.
- The accepted contract, evidence, and review artifacts are
  [`_workspace/use-eager-parquet/`](../../_workspace/use-eager-parquet/); this
  ADR accepts the bounded evaluation while broad runtime integration remains
  deferred.

At the first PR head, the generic hosted unsafe-code job failed because plain
`cargo geiger` treated 33 dependency asset warnings as a nonzero status, despite
zero first-party unsafe usage. The security workflow now records JSON reports,
asserts `forbid(unsafe_code)` and zero first-party unsafe counts for each workspace
package, and surfaces transitive inventory in the job summary. A path-scoped Linux
runtime workflow supplies hosted native-build evidence for this crate; local
macOS Apple Silicon evidence is recorded above.

If any required ownership, semantic, platform, or licensing evidence remains
unresolved, retain the implementation as a bounded partial evaluation or defer
production integration rather than weakening the policy or claiming full
backend adoption.
