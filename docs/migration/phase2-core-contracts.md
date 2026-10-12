# Phase 2 core contracts

## Python contract

Source: clean `../tabdat-explore` checkout at
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`; `uv.lock` SHA-256
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
Verified locally before implementation. Contract sources are `src/tabdat/models.py`
(command union, metadata, result records), `executor.py` (SessionState and family
caches), `errors.py`, `script.py` (located errors), and the existing Rust contract
suites. This phase ports type boundaries, not previously unsupported execution.

## Rust contract

Slice 1 moves existing owned command/expression, result, metadata and error
contracts into a backend-independent `tabdat-domain`. Language/runtime re-export
existing names to preserve callers. Domain may use serde and the already pure
`tabdat-stats`; it must never depend on DuckDB, language, runtime or native handles.
Numeric syntax spellings remain strings to preserve precision. Row limits retain
their validated arbitrary-length decimal invariant across construction and decoding.

Slice 2 adds explicit panel/session/model-family state, typed effect and capability
metadata and a versioned serialization prototype. Python's independently nullable
family caches are intentionally replaced by one finite enum; backend-specific
scoring caches remain adapter-owned. Existing fitted linear behavior is preserved.

Slice 3 establishes nonempty execution/data/I/O/reporting/shell/MCP boundaries by
moving existing implementations or introducing tested interface contracts; it
must not claim a new REPL, MCP server or production native adapter merely because
a crate exists. Approved Phase 1 FFI remains isolated until needed by runtime work.

## Test contract

Characterize existing parser/runtime/CLI behavior with the full workspace suites.
Add backend-free construction, row-limit rejection/roundtrip, state/effect routing,
JSON golden and dependency-direction tests. Run format, locked check/test/Clippy,
`git diff --check`, dependency policies and unsafe inventory for changed topology.
No new numerical parity claim is made and no tolerances change.

## Implementation mapping

Owned contracts and deterministic routing/serialization: safe Rust. Existing
DuckDB evaluation: data adapter. Filesystem/script paths: I/O boundary. Existing
numerical utilities: tabdat-stats. libgretl/ReadStat native integration, REPL and
MCP transport: later phases, not prerequisites for this type-system phase.

## Status

Slice 1 complete (PR #185): command/result/metadata/script and execution error
ownership moved, with compatibility re-exports and validating RowLimit constructor.
New tests first failed because the domain package was absent; after implementation
709 workspace tests passed, one pre-existing test ignored (90 suites). Format,
locked all-target check/test/Clippy, diff/link checks, cargo deny and audit passed.
Full dependency geiger inventory for domain confirms forbid-unsafe and zero used
first-party unsafe. Review compared moved result/error contracts modulo formatting
and the documented label visibility/helper relocation: no behavior differences.

Three owner review passes (not independent model reviews): contract/source identity,
API/row-limit invariants and compatibility, and dependency/safety/docs/test evidence.
No actionable issues remained. The initial 120-second test invocation timed out
while running suites; the complete rerun passed. Initial geiger relative manifest
was rejected; the absolute-manifest invocation passed. No failures were hidden.

Slice 2 WIP: explicit panel and owned execution/family state, exhaustive command
names/effects/capability requirements, typed top-level errors and a lossless tagged
JSON prototype. Existing CLI v1 JSON remains unchanged and separate from the new
internal domain exchange format. Runtime linear caches will use the domain state.
Slice 3 will extract nonempty workspace boundaries. Both have early draft PRs,
progress commits, verification and owner review before autonomous merge.
