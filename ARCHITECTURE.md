# TabDat Explore Rust architecture

**Status:** current scaffold plus proposed ownership boundaries

This document separates repository truth from the target architecture. The Rust
repository currently contains one binary crate and no TabDat command implementation.
The [proposal](docs/TABDAT_RUST_PORT_PROJECT_PROPOSAL.md) and
[roadmap](docs/TABDAT_RUST_PORT_ROADMAP.md) are design plans; their proposed crate
names and product commands are not installed support. The [migration authority](docs/migration/README.md)
defines how Python behavior becomes evidence rather than letting this document
silently invent parity.

## Current implementation

- `Cargo.toml` describes one unpublished Rust 2024 binary package.
- `src/main.rs` contains the scaffold entry point, forbids unsafe code, and prints
  `Hello, world!`.
- `tests/scaffold.rs` characterizes that placeholder output, successful exit, and
  empty stderr.
- `rust-toolchain.toml`, rustfmt, Clippy, baseline CI, dependency policy, advisory
  checks, and unsafe inventory are development controls, not runtime architecture.
- There is no parser, session model, data engine, statistical backend, REPL, MCP
  server, or public TabDat command contract in this repository yet.

No component should be described as implemented until source/tests and the relevant
roadmap gate provide that evidence.

## Target dependency direction

The intended flow is:

```text
language / interface
        ↓ typed commands
execution / session
        ↓ domain-owned plans and capability requests
backend adapters
        ↓ owned domain results
execution / reporting / interface
```

The dependency arrow points toward effects and installed capabilities. Domain types
must not import runtime backends. Backends may depend on domain contracts, but a
DuckDB, FFI, plotting, or MCP type must not leak into public language/domain APIs.
Human-readable, JSON, and MCP surfaces should render the same typed result rather
than implement separate command semantics.

The proposal's candidate boundaries are guidance, not a crate tree to create all at
once:

- **Language:** syntax, AST, command schemas, scripts, and diagnostics.
- **Domain:** commands, typed results, errors, metadata, session/model state, and
  invariants that do not require I/O or an installed backend.
- **Execution:** dispatch, state transitions, capability routing, cancellation,
  and post-estimation coordination.
- **Data/IO:** DuckDB relations, file formats, labels, lazy plans, and translation
  into domain-owned values.
- **Statistics:** estimator contracts, inference, diagnostics, covariance, and
  post-estimation state; optional native adapters remain behind these contracts.
- **Surfaces:** shell, reporting, JSON, and MCP adapters that consume typed results.

A future implementation may combine or split these boundaries. A split requires
current evidence and, when it is a major dependency/runtime decision, an ADR.
Avoid monolithic Python-style dispatcher/backend modules and avoid speculative empty
crates.

## Functional state and effects

For a command boundary, prefer the explicit shape:

```text
session state + typed input -> Result<(new state, typed result), typed error>
```

Pure transformations should own parsing-independent validation, expression/domain
semantics, result shaping, and deterministic rendering inputs. Effects belong at
adapters: filesystem/network access, DuckDB execution, native calls, terminal I/O,
time, and randomness. Inject or record effect inputs when deterministic comparison
requires them.

State transitions must make preconditions and failure atomicity visible:

1. Parse and validate into an owned command before mutating session state.
2. Check capability and data preconditions without partially applying a mutation.
3. Execute through the narrow adapter and translate failures into deterministic,
  typed domain/application errors.
4. Commit the new session state only after the command succeeds; preserve the prior
  state on recoverable failure unless the recovered contract explicitly says otherwise.
5. Return a typed result from which all presentation surfaces derive output.

Relations, named tables, labels, estimation samples, covariance modes, and
post-estimation families are domain state with explicit invariants, not unrelated
nullable fields or backend-owned handles.

## Data and capability boundaries

DuckDB is the planned canonical initial tabular engine. Initialize it only when a
command needs data execution; metadata-only or syntax-only paths must not eagerly
load unrelated statistical, plotting, spatial, Bayesian, MCP, or FFI capabilities.
A lazy relation remains lazy unless the command's contract requires materialization.
Do not add a second execution engine until benchmark and semantic evidence justify it.

Backend adapters own connection/handle lifetimes and conversion details. They return
owned domain/application values or typed adapter errors. They do not expose native
model/data types, raw pointers, or backend-specific lifetimes to language/domain
callers. Cancellation and resource cleanup are part of each adapter contract.

## Unsafe and FFI boundary

Ordinary application/domain crates should use:

```rust
#![forbid(unsafe_code)]
```

An unsafe-enabled crate must be explicitly approved for low-level interoperability
or exceptional systems work. Every unsafe block is narrow and adjacent to a
`// SAFETY:` explanation of the invariant; unsafe functions deny unsafe operations
unless each operation is justified. Approved adapters must provide safe facades,
owned RAII handles, panic containment at FFI boundaries, and documented thread-safety
assumptions. No raw foreign pointer or native model type appears in a public
TabDat/domain API, and no foreign handle is marked `Send`/`Sync` without upstream
proof.

The CI `cargo-geiger` report is an inventory signal. It does not approve a proposed
FFI adapter, establish memory safety, or replace code review, sanitizers, fuzzing,
and native dependency/license decisions. Such a capability needs its own feasibility
slice and ADR.

## Migration and validation

Each ported behavior gets four explicit contracts:

1. **Python contract:** pinned revision, syntax/options/defaults, state effects,
  outputs/errors, source/docs/tests, and unresolved conflicts.
2. **Rust contract:** typed command/result/error, ownership, state transition,
  capability requirement, and intentional deviations.
3. **Test contract:** test-first cases, exact semantic comparisons, fixtures,
  diagnostics, and trusted statistical references where applicable.
4. **Implementation mapping:** native Rust, DuckDB, approved FFI, optional capability,
  or deferred work.

Recover language and execution behavior before designing backend breadth. Syntax-only
parsing must remain independent of backend initialization. Differential fixtures may
compare Python and Rust only after both contracts and environment/revision details
are recorded; statistical claims additionally require a trusted external reference.
Passing the scaffold or policy checks does not establish command or statistical parity.

## Ownership and decision records

- This file owns Rust structure, dependency direction, state/effect boundaries, and
  current-versus-proposed architecture.
- `SPEC.md` owns the small active slice and evidence state.
- The roadmap owns phased backlog and acceptance gates; checkboxes require evidence.
- `docs/migration/` owns Python baseline, authority, conflict, and parity evidence.
- `docs/adr/` owns accepted major backend, FFI, packaging, licensing, and dependency
  decisions. `docs/adr/template.md` is the minimal format.
- `AGENTS.md`, `CONTRIBUTING.md`, and repo-local skills own repeatable workflow and
  checks; they do not assert product support beyond source evidence.

When these documents disagree, record the conflict and avoid choosing a behavior by
convenience. Update the narrowest owning document after implementation or evidence;
do not mark a proposed boundary as present merely because it is described here.

## Explicit deferrals

This document does not select a DuckDB crate/version, native statistical library,
ReadStat/libgretl ABI, plotting stack, parser framework, workspace crate layout,
minimum supported Rust version, packaging model, or Rust/Python parity surface. Those
are bounded decisions for later spikes with feasibility, license, performance,
ownership, and validation evidence. Python and R remain external validation tools,
not core Rust runtime dependencies.
