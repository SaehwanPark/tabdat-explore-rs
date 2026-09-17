# ADR 0007: Eager local-Parquet runtime boundary

- Status: Proposed; bounded evaluation in PR #22
- Scope: `use <existing-local-parquet>` in eager mode only

## Context

The language layer now parses `use`, but the repository has no session or data
runtime. The isolated DuckDB feasibility prototype proves local Parquet reads,
ordered Arrow results, and owned connection teardown, but it explicitly defers a
domain-owned adapter, failure-atomic session state, thread/lifetime review,
packaging, and redistribution decisions.

## Decision under evaluation

Evaluate a workspace `tabdat-runtime` library that owns a session and keeps a
DuckDB connection private to a safe adapter. The first supported operation is
only an existing local `.parquet` loaded eagerly from a typed
`tabdat-language::Command::Use`. The adapter stages and inspects the relation,
then replaces the active table transactionally; the session publishes owned
metadata only after success. DuckDB is initialized lazily when this operation is
requested, not when a session is constructed.

The dependency candidate is `duckdb-rs` `1.10505.0` with `bundled` and `parquet`
features, matching the isolated prototype. No raw handle, Arrow value, or
foreign lifetime may cross the runtime public API, and ordinary Rust crates
continue to use `#![forbid(unsafe_code)]`.

## Explicitly out of scope

CSV/DTA/Feather/Arrow, URI/network access, lazy execution, named tables,
transformations, inspect/count execution, labels, CLI/JSON/MCP surfaces, and a
general relation API remain deferred. This slice must not mark the broad `use`
or Phase 4 data-runtime roadmap items complete.

## Required evidence before acceptance

- success and failure-atomicity tests against a synthetic Parquet fixture;
- schema/order and row-count metadata checks;
- local and hosted locked Rust, dependency, advisory, and unsafe inventory checks;
- DuckDB ownership, transaction/cleanup, threading, and panic-boundary review;
- macOS Apple Silicon and Linux x86_64 build evidence;
- license/notice and bundled-native redistribution review against the project's
  AGPL intent; and
- an accepted disposition in this ADR and the `_workspace/use-eager-parquet/`
  contract/evidence/review artifacts.

If any required ownership, semantic, platform, or licensing evidence remains
unresolved, retain the implementation as a bounded partial evaluation or defer
production integration rather than weakening the policy or claiming full
backend adoption.
