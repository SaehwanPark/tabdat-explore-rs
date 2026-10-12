# ADR 0022: owned domain state and directional workspace boundaries

- Status: Accepted
- Date: 2026-10-12
- Scope: Phase 2 type system, not later-phase product parity

## Context

The repository already has a large bounded runtime, syntax-only language contracts,
pure statistics and CLI discovery/JSON contracts. Creating duplicate types or empty
crates would make the Phase 2 topology misleading. Python's pinned `models.py` uses
unions/Literal fields and `executor.py:SessionState` has independently nullable
backend-native family caches. Native cache layouts are not Rust domain APIs.

## Decision

1. Domain owns command/expression/result/error/metadata and owned session/family
   contracts. It depends only on serde and the pure stats crate. Language/runtime
   re-export old names while consumers migrate. Existing eager `DatasetInfo` stays
   source-compatible; `DatasetMetadata` additionally models unknown lazy counts,
   URI sources, panel keys and labels without implementing lazy execution.
2. One `Option<EstimationState>` replaces nullable family slots. Each family is a
   finite variant, and normalized owned coefficients/covariance/sample state may
   be shared. Backend-specific scoring caches and native handles remain adapters'
   responsibility. Runtime currently publishes only `Linear` and keeps its existing
   behavior: a new regression attempt clears the previous fit before validation.
   Other variants compile but do not assert estimation/prediction support.
3. Effects reproduce conservative Python discovery categories (for example SQL
   is read/write and `by`/`run` may have all effects). Capability metadata is a
   typed requirement, not initialization: nested prefixes include child
   requirements; syntax-only/help/state inspection never initialize a backend.
4. Introduce internal domain exchange schema v1 separately from the existing CLI
   v1 envelope. Tagged snake_case unions encode finite choices; limits are
   validated decimal strings, wide scalar integers/decimals are decimal strings,
   scalar floats are text (including nonfinite values and negative zero), binary
   is a byte array. Unsupported envelope versions are rejected. This internal
   format is not a claim of Python JSON parity, and existing CLI JSON is unchanged.
   Stats records retain their existing serde contracts; nonfinite statistical
   result fields are not guaranteed to roundtrip in internal JSON.
5. Extract actual effects into nonempty data/I/O and execution crates; retain
   runtime as a compatibility facade. Extract existing reporting/shell code.
   MCP gets a tested transport-independent typed tool boundary, not a server.
   No production FFI crate is needed: approved Phase 1 adapters remain isolated
   until a runtime integration slice supplies its ownership/release evidence.

## Consequences and validation

No Python/R dependency, new native library, engine or numerical algorithm is added.
The selected boundaries remain acyclic. Backend-independent contract tests, full
workspace regression checks, CLI JSON fixtures, metadata dependency checks and
unsafe inventory establish this phase. They do not establish all-command parity,
MCP transport, REPL behavior or native distribution safety. Future family-specific
cache/result additions require the bounded execution/reference contract, not a
`Box<dyn Any>` escape hatch or a schema-version bump without migration evidence.
