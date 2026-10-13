# TabDat Explore Rust architecture

**Status:** Phase 2 core workspace implemented; bounded execution, not full product parity

This document separates source-established behavior from the future product in the
[proposal](docs/TABDAT_RUST_PORT_PROJECT_PROPOSAL.md) and
[roadmap](docs/TABDAT_RUST_PORT_ROADMAP.md). The
[migration authority](docs/migration/README.md) defines how pinned Python behavior
becomes evidence. A compiling type or crate is not proof of command support.

## Current implementation

The Rust 2024 workspace contains the root binary and ten library crates. All eleven
first-party packages forbid unsafe code. Phase 2 ownership and intentional design
differences are recorded in [ADR 0022](docs/adr/0022-domain-state-and-workspace-boundaries.md)
and [the recovered contract / review](docs/migration/phase2-core-contracts.md).

| Crate | Implemented responsibility | Runtime/backend dependencies |
| --- | --- | --- |
| `tabdat-domain` | Owned commands/AST, results, dataset/label/panel metadata, errors, execution snapshot, one optional finite estimation-family state, typed effects/capabilities, internal JSON v1 prototype, discovery/envelope records | serde + pure stats only |
| `tabdat-language` | Existing command/tokenizer/expression/script syntax, macros/control directives and deterministic parse errors | domain; lightweight I/O for the existing explicit `read_script` compatibility helper |
| `tabdat-stats` | Pure statistical problems/samples, coefficients/covariance/inference, Householder QR OLS/WLS/GLS, HC1/cluster covariance, hypothesis/VIF and t-test utilities | serde/serde_json; no DuckDB or FFI |
| `tabdat-io` | Local input format/path checks, output-format/replacement/parent checks, script bytes/UTF8 diagnostics and path resolution | domain only |
| `tabdat-data` | Canonical bundled DuckDB adapter, generated SQL, eager CSV/Parquet loading/persistence, relation transformations/inspection and native-to-owned cell conversion | domain + I/O + DuckDB/sha2 |
| `tabdat-execution` | Session coordination, metadata/label/model publication, named tables, data/script/statistical dispatch and owned snapshots | domain, language, I/O, data, stats |
| `tabdat-reporting` | Existing deterministic catalog/schemas/effects/help and bounded Python-compatible CLI JSON | domain/language + serde_json/base64; no runtime backend |
| `tabdat-shell` | Existing batch/script CLI argument parsing, exit policy and terminal/JSON effects | language/execution/reporting; no native types |
| `tabdat-mcp` | Typed transport-independent discovery/explain/execute tool boundary sharing parser, coordinator and result reporting | domain/language/execution/reporting; **not an MCP server** |
| `tabdat-runtime` | Source-compatible domain/Session re-exports and existing integration contract tests | domain/execution; native test fixtures are dev-only |

`src/main.rs` delegates to shell. Existing `--help`, `--version`, discovery,
repeated `-c`, `-f`, positional scripts and bounded JSON behavior are preserved.
No-argument execution still prints the scaffold greeting; no interactive REPL is
implemented. JSON success reporting remains limited to the existing use/count/
head/tail surface. The typed MCP boundary does not wire `--mcp` to a server.

The language accepts more forms than execution supports. The supported eager
runtime contains local CSV/Parquet, inspection/transforms/labels, named-table SQL,
join/append/reshape, scripts, linear regress/predict/VIF/lincom/test and t-tests.
Source tests and individual roadmap entries define each bounded variant; parsing
an estimator, panel or plotting command does not establish its execution.
Session publishes only `EstimationState::Linear` today. Other family variants and
unknown lazy row-count metadata are contracts for later integration, not installed
capabilities. Existing regression-attempt failure behavior is retained: beginning
a new fit clears the prior fit before validation.

Execution and data are split into responsibility modules, not new monolithic
executor/backend files. DuckDB connections, statements, rows and native values
remain private to data. Statistical sample extraction streams one owned row at a
time; unsupported cell conversions are retained until accessed so a row skipped
for another reason does not spuriously fail. No native row/connection lifetime
crosses into execution. Data operation failures use an owned `DataError`; existing
command-specific `RuntimeError` messages remain at coordination boundaries.

`tests/workspace_boundaries.rs` checks dependency direction and safe crate roots;
existing parser/runtime/statistical/CLI contracts characterize the extraction.
Build/toolchain/advisory/license/unsafe inventory are local development controls,
not product parity or FFI-safety proof. GitHub Actions are disabled with no tracked
workflows; see [ADR 0009](docs/adr/0009-disable-github-actions.md).

## Accepted backend decisions

Phase 1 is complete as feasibility/architecture acceptance under
[ADR 0021](docs/adr/0021-phase1-backend-and-linear-algebra-decisions.md) and
[the final report](docs/feasibility/phase1-final-report.md): canonical DuckDB,
DTA-only patched ReadStat, primary compiled libgretl alongside existing safe Rust
utilities, BLAS/LAPACK reused within gretl, and native SVG/resvg. No duplicate
engine, standalone numerical FFI or JavaScript/Python plotting runtime is added.

ReadStat/libgretl/plotting adapters remain isolated spikes, not production workspace
integrations. Phase 2 needs no additional low-level FFI crate. Accepted feasibility
does not approve arbitrary hostile input, relocatable shipping artifacts, native
redistribution, or full platform/sanitizer certification. Those later-phase
integration/release prerequisites are not silently removed by workspace completion.

## Dependency direction

```text
shell / typed MCP tools
  ├── reporting ── language ── domain
  └── execution
        ├── language ── domain
        ├── data ── domain + I/O
        ├── I/O ── domain
        └── pure stats

domain ── pure stats + serde
runtime ── domain + execution (compatibility only)
```

Dependencies remain acyclic. Language's explicit legacy script reader delegates
filesystem effects to lightweight I/O; parsing itself performs no I/O or backend
initialization. Data does not depend on language or execution. Domain types must
never import runtime backends. Backend adapters return owned domain results,
not raw foreign pointers, native model/data types or backend-specific lifetimes.
Human/JSON/MCP surfaces share typed contracts, not separate command semantics.

Further splits/merges require current evidence and an ADR for major dependency,
runtime, backend, FFI, packaging or licensing changes. Avoid speculative empty
crates, Python module-layout replication and `Box<dyn Any>` escape hatches.

## Functional state and effects

Prefer the explicit shape:

```text
session state + typed input -> Result<(new state, typed result), typed error>
```

Pure functions own validation, domain semantics, sample/result shaping and
rendering inputs. Filesystem/network, DuckDB, native calls, terminal output, time
and randomness belong to adapters. Inject/record effect inputs when deterministic
comparison requires them.

State transitions make preconditions and failure policy visible:

1. Parse/validate into an owned command before mutation.
2. Check data/capability preconditions without accidental partial publication.
3. Execute through a narrow adapter and normalize errors into typed failures.
4. Publish state after success; preserve prior state on recoverable failure unless
   the recovered contract explicitly specifies otherwise (as for new fit attempts).
5. Return a typed result used by all supported presentation surfaces.

Relations, named tables, labels, samples, covariance and post-estimation families
are explicit domain state, not unrelated nullable native objects. A snapshot owns
its data; reading it and initialized-capability metadata does not initialize data.

## Data and capability boundaries

DuckDB is the canonical data engine; no second engine is accepted. Lazy-engine
syntax/types remain compatibility metadata, not support for a Polars runtime.
Initialize data only when execution needs it. Help/discovery/syntax/state paths
must not initialize statistical, plotting, spatial, Bayesian, MCP or FFI systems.
Command effects are conservative discovery metadata; requirements describe
capability families, not permission grants or eager initialization requests.
Post-estimation requirements also depend on the fitted family at dispatch.

Future lazy relations remain lazy until the command requires materialization;
unknown counts are not zero. Cache invalidation, cancellation/resource cleanup,
ordering/missingness/labels, and failure atomicity require their own bounded
contracts. Existing save/export validation is not an atomic output-file
publication or general persistence guarantee. Low-level adapter internal-SQL
methods and MCP Execute are trusted operations, not execution sandboxes; a
transport must enforce its own authorization/resource policy.

## Unsafe and FFI boundary

Ordinary crates use `#![forbid(unsafe_code)]`. Unsafe-enabled crates require
explicit approval for low-level interoperability/exceptional systems work,
`#![deny(unsafe_op_in_unsafe_fn)]`, narrow unsafe blocks and adjacent `// SAFETY:`
reasoning. Approved adapters must wrap ownership in RAII, contain panics at FFI
boundaries and document thread-safety assumptions. No foreign handle becomes
`Send`/`Sync` without upstream proof and no raw foreign ownership escapes.

The full `cargo-geiger` inventory is a signal, not an unsafe approval or proof of
transitive native memory safety. Review, sanitizers, fuzzing, upstream ownership
contracts and native license/distribution decisions remain necessary.

## Migration, serialization and validation

Each migrated behavior records four contracts:

1. **Python:** verified pin, syntax/defaults/state/output/errors, sources/tests,
   conflicts and required statistical references.
2. **Rust:** typed inputs/results/errors, ownership/state transitions, capability
   requirements and intentional deviations.
3. **Tests:** fixtures, exact comparisons/normalizations, environment/seed/tool
   details, diagnostics and trusted numerical references where applicable.
4. **Mapping:** safe Rust, DuckDB, approved FFI, optional capability or deferred work.

Recover semantics before backend breadth. Python is an oracle, not unquestioned
truth; statistical claims also need trusted external references, unchanged
justified tolerances and explicit sample/covariance/inference contracts.

[Internal domain JSON v1](docs/migration/domain-json-v1.md) is a tagged migration
prototype, separate from the unchanged Python-compatible CLI v1 envelope. It
supports typed/golden migration tests, not a full public protocol/schema promise.
Backend/model validity and execution authority are not established by decoding.

## Documentation ownership and remaining work

- This file owns implemented/target structure, direction, state/effects and deferrals.
- `SPEC.md` owns active implementation and evidence; historical slices document
  acceptance at merge, not later composition or whole-command parity.
- The roadmap owns phased gates; checkboxes need source/test/decision evidence.
- `docs/migration/` owns oracle authority, conflicts, contracts and comparisons.
- `docs/adr/` owns accepted major architecture/backend/FFI/dependency decisions.
- `AGENTS.md`, `CONTRIBUTING.md` and repo-local skills own workflow/check discipline.

Phase 2 establishes core type/workspace boundaries. Broad parser/data/estimator
parity, general lazy/remote/DTA runtime, REPL, MCP transport, native integration,
benchmarks, cancellation, hostile-input hardening and release packaging remain
later-phase tasks. Core distribution requires neither Python nor R; external
validation tools are not runtime dependencies. Do not describe those future
capabilities as present merely because their contracts or spikes compile.
