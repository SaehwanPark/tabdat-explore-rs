# ADR 0003: Isolated DuckDB Rust feasibility prototype

- Status: Accepted for continued evaluation; production adoption deferred
- Scope: Local CSV/Parquet/Arrow feasibility only

## Context

DuckDB is the planned canonical tabular engine, but the Rust repository is still a
scaffold. Choosing a crate, native linkage mode, relation lifecycle, and Arrow
conversion without evidence would make a proposed backend look implemented and could
leak foreign ownership into domain APIs.

## Decision

Maintain an isolated, unpublished prototype under
[`spikes/duckdb-prototype`](../../spikes/duckdb-prototype) using `duckdb-rs`
`1.10505.0` with bundled DuckDB and Parquet features. It owns an in-memory
connection and an `active` view, exercises local CSV and Parquet reads, repeated
row counts, and Arrow result batches, and records a release-build orientation
measurement. Its lockfile and tests are separate from the root package; no DuckDB
runtime dependency is added to the root binary.

The prototype's safe Rust facade forbids unsafe code and does not expose raw DuckDB
or Arrow handles beyond the short-lived query operation. Production adoption requires
a separate domain-owned adapter, explicit state/error/cancellation/thread contracts,
representative semantics, and platform/license evidence. HTTP/S3, remote Parquet,
labels, second engines, and production session integration are deferred.

## Evidence and risks

The macOS Apple Silicon run passed two tests and a release build; Linux x86_64 is
awaiting a passing path-scoped spike workflow. `cargo audit` passed the 168-package
lockfile locally. `cargo deny` passed advisories, licenses, bans, and sources but warned on
several duplicate versions. `cargo geiger` found no unsafe code in the spike and
reported unsafe usage in the DuckDB dependency subtree; its nonzero exit is an
expected inventory signal, not a safety approval. The bundled stack includes
`CDLA-Permissive-2.0`, explicitly allowed for this evaluated dependency and subject
to redistribution review. Measurements are local orientation data, not performance
gates.

## Alternatives

- Add DuckDB to the root package immediately: rejected because it would conflate
  feasibility with product support and domain/backend ownership.
- Use a system DuckDB library: deferred until macOS/Linux linkage, versioning, and
  redistribution evidence exists.
- Use another data engine: rejected for this spike because the proposal names DuckDB
  as canonical and no benchmark evidence supports a second path.

## Revisit conditions

Accept, defer, or reject the candidate after Linux build evidence, upstream ownership
and thread-safety review, realistic semantic fixtures, native packaging/license
review, and measured costs are available. Supersede this ADR if the selected crate,
linkage mode, or canonical engine changes.
