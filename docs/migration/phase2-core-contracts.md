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

## Major Python type/signature review

| Python contract at the pin | Rust signature / decision | Behavioral policy and evidence |
| --- | --- | --- |
| `models.py:Command` dataclass union and parsed expressions | `Command`, command-specific structs, finite option/operator enums and recursive owned AST | Preserve existing parser forms/diagnostics; language contract suites; no runtime backend import |
| `ColumnInfo`, `DatasetInfo` | eager compatibility `DatasetInfo`; general `DatasetMetadata` with `Option<u64>`, `DataSource`, ordered schema | Preserve existing eager runtime; lazy/URI metadata compiles but execution is later work |
| `PanelMetadata`, `PanelStructureSummary` | validated distinct-key `PanelMetadata`, typed summary with `is_balanced()` | Retain key names and balance definition; constructors/decoders enforce distinctness |
| `ValueLabelSet`, `LabelMetadata` | owned ordered mapping/attachment records, `LabelValue` enum | Preserve existing label execution and normalized order; no dynamic scalar objects |
| Existing execution result union / `TransformResult` | owned `ExecutionResult` and named transform results | Preserve bounded Rust payloads rather than invent messages/overflow counts; further family-specific output records follow their execution slices |
| `CoefficientEstimate`, estimation/inference records | existing pure `tabdat-stats` contracts | Reused unchanged; not a new statistical validation claim |
| `SessionState` | owned `ExecutionState` with optional dataset/model/panel and deterministic registry snapshot | Explicit optional fields; backend connections excluded; snapshot does not initialize capabilities |
| Private family caches `_RegressionState` through `_SpatialRegressionState` | `Option<EstimationState>`; finite variants + normalized owned `ModelState`; current linear cache is complete | Intentional ADR 0022 replacement of independently nullable/native slots; adapter scoring caches excluded, no claim of unsupported-family execution |
| `TabDatError`/parse/execution subclasses and `ScriptError` | `TabDatError` hierarchy, typed existing `RuntimeError`, located `ScriptError`, wire `Diagnostic` | Failure is `Result`; deterministic messages/source chains; no foreign error/handle escapes |
| Effect Literal categories and discovery records | `CommandEffect` and exhaustive command metadata | All 81 conservative discovery entries match pinned Python JSON; no implied permission grant |
| Backend/library checks | finite `Capability` requirements incl. nested command requirements | New Rust metadata, not Python module replication; installation/variant checks remain execution-owned |
| JSON result formatting | existing CLI v1 plus internal `DomainEnvelope<T>` prototype | CLI unchanged; internal tagged prototype explicitly versioned and documented separately |

No `Any`, dynamic/native model escape hatch, or duplicate backend engine is used.
Finite choices are enums, optional state uses Option, and fallible APIs use Result.
`Box` is used only for recursive commands/expressions, the large linear-state
variant and typed errors, never type erasure. ADR 0022 records design deviations.

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

Slice 2 implemented (PR #186): explicit panel/execution/family state, exhaustive
command names/effects, typed capability requirements and top-level errors, and
[internal JSON v1 prototype](domain-json-v1.md). Runtime uses the domain Linear
family cache and exposes an owned snapshot. CLI JSON is unchanged. New tests first
failed for absent state/wire APIs. Two representative fixture inputs were corrected
to existing parser syntax (quoted label text, explicit recode target), without
changing the parser. Clippy caught the large Linear variant; boxing fixed it without
changing wire shape. No tolerance or statistical implementation changed.

Pinned oracle command (Python 3.13.3, existing unsynchronized environment):
`PYTHONDONTWRITEBYTECODE=1 uv run --no-sync tabdat --json --list-command-effects`.
All 81 entries exactly match the existing Rust discovery table used to recover
exhaustive domain effects. Output is committed at
`../../tests/fixtures/python-command-effects-v1.json`; checkout remained clean.

Slice 2 validation: 716 passed, one pre-existing ignored (92 suites); format,
locked all-target check/test/Clippy, diff/link/schema-JSON checks and full domain
geiger inventory passed. No dependency versions changed; slice 1 deny/audit evidence
still applies. Three owner review passes covered wire/constructor invariants,
family cache/source/error compatibility, and architecture/effect/fixture evidence.
No actionable issues remained; independent model review was not used.

Slice 3 implemented (PR #187): existing DuckDB operations moved into
`tabdat-data`, filesystem/script-path effects into `tabdat-io`, coordination into
responsibility modules in `tabdat-execution`, catalog/help/JSON into
`tabdat-reporting` and CLI into `tabdat-shell`. Domain owns discovery/envelope
records, with typed effects preserving JSON. `tabdat-runtime` re-exports remain
source-compatible. `tabdat-mcp` is a typed transport-independent tool boundary,
not a server; it shares parser/coordinator/owned results and CLI JSON reporting.

## Extraction state/effect review

| Boundary | Success | Failure / unchanged scope |
| --- | --- | --- |
| Read-only inspection | Existing owned result, no metadata/label/model publication | Existing typed errors; no new backend on a missing dataset |
| Local eager use / named activation | Existing staged schema/count/relation publication and label clearing | Prior staging-error behavior retained; remote/lazy formats still unsupported |
| Transforms | Existing relation staging, metadata/named-table sync and label retention/rename policies | Existing failure tests retained; no new general atomicity claim beyond those contracts |
| Save/export | Existing target/replacement/parent checks and backend output | Active state preserved; output-file atomicity/other formats not newly promised |
| Fit attempt | Clear previous model on attempt, publish owned Linear cache on success | Preserves existing reset-before-validation behavior; no other family execution |
| Streaming sample read | One owned row at a time; complete-case/weight/group decisions unchanged | Query/visitor errors remain typed; unaccessed unsupported cells do not fail skipped rows |
| Script reader/path resolution | Same bytes/UTF8/path/file-line diagnostics through I/O | Same recursion/parse/command error contracts |
| Discovery/explain / typed MCP metadata | Same typed catalogs/results, no capability initialization | Execute is trusted, not a transport/security sandbox; MCP protocol/server remains later work |

Source-identity review verified all 53 unchanged coordinator methods and all 44
DuckDB methods modulo visibility, formatting and typed unit-error normalization.
The four intentional coordinator-body adaptations are save/export's shared I/O
checks and regression/t-test's streaming owned-row boundary. Tests retain numeric
results, samples, cell-conversion and command diagnostics; no tolerance changed.

The first full extraction test run caught a new adapter metadata-timing panic:
DuckDB column metadata was queried after prepare but before execution. Fixed by
reading metadata from executed rows, with three dedicated regressions for empty
queries, unsupported skipped cells, early visitor errors and query recovery. The
[contributor lesson](../../CONTRIBUTING.md#duckdb-metadata-timing) preserves cause,
resolution and prevention. New workspace/MCP tests first failed for missing crates
before extraction. All later full runs passed.

Full policy evidence: cargo-deny 0.20.2 (`advisories/bans/licenses/sources ok`),
cargo-audit 0.22.2 (`-D warnings`, no vulnerability), cargo-geiger 0.13.0 full
all-dependency/all-target inventory for all 11 workspace packages, each with
exactly one matching first-party record, forbid-unsafe and zero used unsafe.
Raw local reports: `/tmp/phase2-geiger-reports/`; all scan statuses were zero.
The first 600-second inventory loop timed out after five complete reports; the
remaining six were rerun to completion, not treated as passing partial output.
These are Rust workspace controls, not proof of transitive native memory safety
or license redistribution approval.

Three owner review passes (not independent model reviews): contract/source
identity and success/failure state policy; native ownership/query/serialization/
interface compatibility; dependency/safety/docs/fixture/roadmap evidence. The
metadata panic was fixed and rechecked; no actionable issue remains. No model
subagents were spawned. AI usage was checked before each slice and after heavy
review; final observation was 66% five-hour / 74% weekly, below wait/stop limits.

Final verification (PR #187): **724 tests passed, one pre-existing manual live-clock
reshape probe ignored, 100 suites**. Format, locked all-target workspace check/test/
Clippy, diff checks, cargo deny/audit and all 11 full geiger inventories passed;
the root inventory was rerun after the final interface tests. Changed/relocated
Markdown links resolve; repo skill frontmatter fields remain intact. Guidance
scenarios reviewed: normal syntax-only parser work follows domain contracts without
a backend; unsupported lazy/native execution still reports its bounded limitation
rather than interpreting compiling metadata/spikes as support. No required check
was skipped or statistical tolerance changed.

## Phase 2 exit disposition

All requested core crate/type/review and four exit gates are demonstrated. Approved
Phase 1 low-level bindings remain isolated because no production FFI integration
is needed by Phase 2. General lazy/remote/panel/family execution, REPL/MCP transport,
native integration, statistical breadth and shipping remain later roadmap work,
not unresolved Phase 2 blockers. Cargo checks establish type/build/regression
acceptance, not whole-command or statistical parity.
