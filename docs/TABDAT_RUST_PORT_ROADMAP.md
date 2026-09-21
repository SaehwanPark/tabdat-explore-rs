# TabDat Explore Rust Port Roadmap

**Status:** Proposed  
**Date:** 2026-09-07  
**Scope:** Migration from the mature Python implementation to a Rust-native TabDat runtime  
**Planning principle:** Preserve behavior and product knowledge while redesigning implementation boundaries for idiomatic, safe, low-latency Rust.

---

## Execution cadence and evidence

This roadmap is a backlog, not a requirement to implement a whole phase in one PR.
One development loop selects one bounded target slice, records its contract and
acceptance evidence, implements/tests it, reviews it, and merges one PR only after
applicable checks pass. Then recover `main` and choose the next unmet dependency.
Use the workflow in [AGENTS.md](../AGENTS.md), including two-space indentation,
functional-first design, specification-driven development, and test-first behavior
changes. Revise future tasks when evidence improves the plan; record major changes
in an ADR rather than silently changing migration semantics.

Initial sequencing:

1. Land repo-local guidance and development-loop rules (documentation-only bootstrap).
2. Establish reproducible Rust formatting, toolchain, scaffold checks, and hosted CI.
3. Recover and pin the Python oracle; define authority and deviation tracking before
   claiming behavioral parity. Repository/build setup does not depend on that recovery.
4. Add security/license tooling and architecture decisions, then undertake bounded
   feasibility spikes with explicit acceptance evidence before product integration.

Check off only the individual tasks demonstrated by source, tests, decisions, or CI.
Passing scaffold checks is not statistical validation, and a merged PR alone does
not satisfy a phase exit gate. Blocked slices retain their evidence and missing
inputs; select independent work only when it does not bypass the blocked contract.

## 1. Always-On Engineering Invariants

### 1.1 Safety

- [ ] Use safe Rust by default across all application/domain crates.
- [ ] Add `#![forbid(unsafe_code)]` to crates that do not require low-level interoperability.
- [ ] Restrict unsafe Rust to explicitly approved low-level FFI or exceptional systems crates.
- [ ] Add `#![deny(unsafe_op_in_unsafe_fn)]` to unsafe-enabled crates.
- [ ] Require an adjacent `// SAFETY:` explanation for every unsafe block.
- [ ] Prevent raw foreign pointers from appearing in public TabDat APIs.
- [ ] Prevent FFI-native model/data types from escaping adapter crates.
- [ ] Wrap owned native handles with RAII.
- [ ] Ensure Rust panics never unwind across FFI boundaries.
- [ ] Do not mark foreign handles `Send` or `Sync` without documented upstream guarantees.
- [x] Track project and dependency unsafe usage with `cargo geiger` ([PR #4 CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/34187594609)).
- [x] Run `cargo audit` and `cargo deny` in CI ([PR #4 CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/34187594609)).
- [ ] Prohibit speculative unsafe micro-optimizations without profiling and dedicated benchmarks.

### 1.2 Architecture

- [ ] Preserve language → execution → backend dependency direction.
- [ ] Keep command semantics independent from installed statistical backends.
- [ ] Keep human-readable and machine-readable output derived from the same typed results.
- [ ] Keep specialized capability initialization lazy.
- [ ] Keep DuckDB as the canonical initial data engine.
- [ ] Do not reintroduce Polars until benchmark evidence justifies a second execution path.
- [ ] Do not recreate monolithic Python-style `executor.rs` or `backend.rs`.
- [ ] Keep crate/module dependencies acyclic.
- [ ] Require ADRs for major runtime, backend, FFI, packaging, and dependency decisions.
- [ ] Keep all public error behavior deterministic and typed.

### 1.3 Statistical trust

- [ ] Treat the Python implementation as an oracle, not an unquestioned golden truth.
- [ ] Preserve the existing reference-validation philosophy.
- [ ] Validate statistical changes against trusted external references.
- [ ] Record intentional differences rather than weakening tolerances silently.
- [ ] Keep estimation sample construction explicit and inspectable.
- [ ] Keep covariance mode and correction semantics explicit.
- [ ] Keep post-estimation family/state compatibility explicit.

### 1.4 Performance

- [ ] Measure cold startup on every release candidate.
- [ ] Measure warm startup on every release candidate.
- [ ] Measure representative REPL command latency.
- [ ] Measure TabDat overhead separately from backend computation.
- [ ] Measure peak RSS for representative workflows.
- [ ] Avoid initializing DuckDB for metadata-only CLI modes when unnecessary.
- [ ] Avoid initializing libgretl for non-statistical commands.
- [ ] Avoid initializing plotting, spatial, Bayesian, or MCP subsystems unless required.
- [ ] Keep command cancellation responsive.
- [ ] Establish CI performance-regression thresholds after baselines stabilize.

---

# 2. Phase 0 - Repository and Migration Governance

## Goal

Create a clean Rust-port workspace with explicit links to the Python implementation as the behavioral source of truth.

### 2.1 Repository setup

- [x] Decide whether the Rust port lives in a new repository or a clearly isolated workspace during migration (ADR 0001).
- [x] Create [`README.md`](../README.md).
- [x] Create the [project proposal](TABDAT_RUST_PORT_PROJECT_PROPOSAL.md) (existing canonical filename).
- [x] Create this roadmap (existing canonical filename).
- [x] Create [`ARCHITECTURE.md`](../ARCHITECTURE.md) as current-versus-proposed architecture authority.
- [x] Create [`CONTRIBUTING.md`](../CONTRIBUTING.md).
- [x] Create [`AGENTS.md`](../AGENTS.md) and [repo-local domain skills](harness/tabdat/team-spec.md).
- [x] Add ADR directory and ADR template (`docs/adr/`).
- [x] Configure 2-space formatting conventions where applicable (`.editorconfig`, `rustfmt.toml`).
- [x] Configure Rust toolchain version policy (`rust-toolchain.toml`, `CONTRIBUTING.md`).
- [x] Configure `rustfmt`.
- [x] Configure Clippy (warnings-as-errors in CI).
- [ ] Configure `cargo-nextest` if adopted.
- [x] Configure `cargo deny` (`deny.toml`, pinned CI install).
- [x] Configure `cargo audit` (pinned CI install and `-D warnings`).
- [x] Configure `cargo geiger` (pinned CI install and all-target/dependency scan).
- [x] Configure basic GitHub Actions CI ([PR #2 build evidence](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/34186271522)).

### 2.2 Migration authority

- [x] Record the current Python repository revision used as the migration baseline (`docs/migration/python-baseline.toml`).
- [x] Define which Python documents are authoritative for user-facing behavior (`docs/migration/README.md`).
- [x] Define precedence between Python docs and Rust docs during migration (`docs/migration/README.md`).
- [x] Define a process for recording intentional Rust deviations (`docs/migration/README.md` and `decisions.md`).
- [x] Add a migration decision log (`docs/migration/decisions.md`).
- [ ] Freeze major new Python estimator breadth unless required for production maintenance.
- [ ] Continue Python correctness fixes when they affect canonical behavior.

### Exit gate

- [x] Rust workspace builds in CI (scaffold only; see PR #2 evidence above).
- [x] Safety and lint tooling run in CI (policy job in the linked PR #4 run).
- [x] Migration baseline revision is recorded (initial pin and verification in `docs/migration/README.md`).
- [x] Architecture and documentation ownership rules are explicit ([`ARCHITECTURE.md`](../ARCHITECTURE.md), ADR 0001, and migration authority policy).

---

# 3. Phase 1 - Technical Feasibility Spikes

## Goal

Resolve the largest backend and FFI uncertainties before implementing the product.

### 3.1 DuckDB spike

- [x] Create a minimal `duckdb-rs` prototype (`spikes/duckdb-prototype/`; [report](feasibility/duckdb.md)).
- [x] Load Parquet (local fixture; [report](feasibility/duckdb.md)).
- [x] Load CSV (local fixture; [report](feasibility/duckdb.md)).
- [x] Query Arrow-compatible results (local `RecordBatch` rows; [report](feasibility/duckdb.md)).
- [ ] Test remote HTTP Parquet (deferred; no authorized endpoint in this slice).
- [ ] Test S3 support if intended for initial parity (deferred pending scope decision).
- [x] Implement active-relation lifecycle prototype (owned in-memory connection/view).
- [x] Measure startup cost (release orientation measurement; not a release gate).
- [x] Measure first-query cost (release orientation measurement; not a release gate).
- [x] Measure repeated-query overhead (release orientation measurement; not a release gate).
- [x] Document ownership/copy behavior ([report](feasibility/duckdb.md)); production adapter remains deferred.

### 3.2 ReadStat spike

- [x] Build ReadStat on macOS Apple Silicon ([report](feasibility/readstat.md)).
- [x] Build ReadStat on Linux x86_64 ([PR #7 workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/34197952522); [report](feasibility/readstat.md)).
- [x] Create `readstat-sys`-style low-level bindings ([spike](../spikes/readstat-prototype/readstat-sys/src/lib.rs); [report](feasibility/readstat.md)).
- [x] Create safe Rust facade prototype (`#![forbid(unsafe_code)]` crate root; [spike](../spikes/readstat-prototype/src/lib.rs)).
- [x] Read representative Stata `.dta` files ([tests](../spikes/readstat-prototype/tests/dta_ingestion.rs)).
- [x] Preserve variable labels ([tests](../spikes/readstat-prototype/tests/dta_ingestion.rs)).
- [x] Preserve value labels (numeric sets fully; string value-label keys are not preserved by ReadStat — [report](feasibility/readstat.md)).
- [x] Validate missing-value behavior (system/tagged preserved; missing strings arrive as empty strings — [report](feasibility/readstat.md)).
- [x] Convert data without pandas ([tests](../spikes/readstat-prototype/tests/dta_ingestion.rs)).
- [x] Confirm all unsafe code is confined to the low-level adapter (`forbid(unsafe_code)` facade; [report](feasibility/readstat.md)).

### 3.3 libgretl spike

- [x] Build/link libgretl on macOS Apple Silicon ([report](feasibility/libgretl.md)).
- [x] Build/link libgretl on Linux x86_64 ([PR #8 workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/34201655560); [report](feasibility/libgretl.md)).
- [x] Create low-level binding crate ([`gretl-sys`](../spikes/gretl-prototype/gretl-sys/src/lib.rs) + C ownership shim; [ADR 0006](adr/0006-libgretl-ffi-adapter-and-ols-fixture.md)).
- [x] Create safe wrapper crate ([`tabdat-gretl-spike`](../spikes/gretl-prototype/src/lib.rs), `forbid(unsafe_code)`; [ADR 0006](adr/0006-libgretl-ffi-adapter-and-ols-fixture.md)).
- [x] Define Rust-owned `EstimationProblem` ([spike](../spikes/gretl-prototype/src/lib.rs)).
- [x] Define Rust-owned `EstimationResult` ([spike](../spikes/gretl-prototype/src/lib.rs)).
- [x] Prevent `MODEL`, `DATASET`, or raw handles from escaping the adapter (RAII wrappers; `forbid(unsafe_code)` facade; [ADR 0006](adr/0006-libgretl-ffi-adapter-and-ols-fixture.md)).
- [x] Implement OLS fixture ([test](../spikes/gretl-prototype/tests/ols_fixture.rs); [report](feasibility/libgretl.md)).
- [ ] Implement robust OLS fixture.
- [ ] Implement clustered OLS fixture.
- [ ] Implement logit fixture.
- [ ] Implement probit fixture.
- [ ] Implement quantile-regression fixture.
- [ ] Implement Tobit fixture.
- [ ] Implement Poisson fixture.
- [ ] Implement negative-binomial fixture.
- [ ] Implement IV/2SLS fixture.
- [ ] Implement panel FE fixture.
- [ ] Implement panel RE fixture.
- [ ] Implement dynamic-panel fixture.
- [ ] Implement Heckman fixture.
- [ ] Implement survival fixture.
- [x] Compare results against Python TabDat (NIST Longley; [test](../spikes/gretl-prototype/tests/ols_fixture.rs); [report](feasibility/libgretl.md)).
- [x] Compare results against trusted reference outputs (NIST/ITL certified; [test](../spikes/gretl-prototype/tests/ols_fixture.rs); [report](feasibility/libgretl.md)).
- [ ] Document semantic mismatches.
- [ ] Benchmark first-use initialization.
- [ ] Benchmark repeated-model overhead.

### 3.4 Visualization spike

- [ ] Generate a Vega-Lite specification in Rust.
- [ ] Render SVG with `vl-convert-rs`.
- [ ] Render PNG with `vl-convert-rs`.
- [ ] Compare visual/output behavior with current TabDat artifacts.
- [ ] Measure initialization and render overhead.

### 3.5 Feasibility decision

- [ ] Produce backend feasibility report.
- [ ] Decide whether libgretl is accepted as the primary compiled classical/econometric backend.
- [ ] Decide whether ReadStat is accepted for DTA support.
- [ ] Decide the initial linear-algebra substrate.
- [ ] Decide whether any FFI dependency is too costly to justify.
- [ ] Record accepted decisions in ADRs.

### Exit gate

- [ ] DuckDB spike passes.
- [ ] DTA ingestion spike passes.
- [ ] At least the core libgretl estimator set passes numerical validation.
- [ ] Plot rendering works without Python.
- [ ] Unsafe boundaries have been reviewed.
- [ ] No unresolved P0 feasibility blocker remains.

---

# 4. Phase 2 - Rust Workspace and Core Type System

## Goal

Build the domain model before substantial runtime implementation.

### 4.1 Workspace topology

- [x] Create `tabdat-language` (bounded syntax-only parser foundations; PRs #11–#20).
- [ ] Create `tabdat-domain`.
- [ ] Create `tabdat-execution`.
- [ ] Create `tabdat-data`.
- [ ] Create `tabdat-io`.
- [ ] Create `tabdat-stats`.
- [ ] Create `tabdat-reporting`.
- [ ] Create `tabdat-shell`.
- [ ] Create `tabdat-mcp`.
- [ ] Create approved low-level FFI crates as needed.
- [ ] Add `#![forbid(unsafe_code)]` to all safe crates.

### 4.2 Core domain types

- [ ] Port command enums.
- [ ] Port result enums.
- [ ] Port dataset metadata types.
- [ ] Port labels metadata.
- [ ] Port panel metadata.
- [ ] Port execution-state types.
- [ ] Design explicit estimation-family state enum.
- [ ] Design typed error hierarchy.
- [ ] Define stable JSON serialization contracts.
- [ ] Define command effect categories.
- [ ] Define capability requirement metadata.

### 4.3 Rust contract review

For each major Python type/signature:

- [ ] Record existing Python contract.
- [ ] Determine whether behavior should remain unchanged.
- [ ] Design idiomatic Rust signature.
- [ ] Eliminate Python-specific dynamic representation.
- [ ] Encode finite choices as enums.
- [ ] Encode optional state as `Option`.
- [ ] Encode failure as `Result`.
- [ ] Avoid `Box<dyn Any>`-style escape hatches unless explicitly justified.

### Exit gate

- [ ] Core domain crate has no runtime backend dependency.
- [ ] Core domain crate forbids unsafe.
- [ ] Main command/result/state types compile.
- [ ] JSON schema prototype is stable enough for migration tests.

---

# 5. Phase 3 - Parser, Language, and Script Migration

## Goal

Port TabDat's language before implementing most execution.

### 5.1 Parser

- [x] Port bounded syntax-only control/status/inspection/diagnostic/configuration forms
  (`help`, `status`, `exit`, `describe`, `doctor`, `set`, `datasignature`, `count`,
  `head`, `tail`, and direct `use` syntax; PRs #11–#20).
- [x] Add the bounded direct `codebook [varlist]` syntax slice (PR #18; execution
  and full varlist semantics remain deferred).
- [x] Add the bounded direct `missing [varlist]` syntax slice (PR #19; execution
  and full varlist semantics remain deferred).
- [x] Add the bounded direct `duplicates [report] [varlist]` syntax slice (PR #20;
  execution and full varlist semantics remain deferred).
- [x] Add the bounded direct `summarize [varlist]` syntax slice (PR #21;
  structured conditions/options, execution, and full varlist semantics remain
  deferred).
- [x] Add the bounded direct `isid [varlist] [, missok]` syntax slice (PR #23;
  active-dataset key semantics, execution, and full tokenizer/varlist parity
  remain deferred).
- [x] Add the bounded direct `run <script-path>` syntax slice (PR #24, merged
  as `77f4754`;
  script loading, line-oriented execution, nested/recursive scripts, and
  file/line diagnostics remain deferred).
- [x] Add the bounded direct `rename <old> <new>` syntax slice (PR #25, merged
  as `89f6c14`; schema lookup, collision checks, relation mutation, and
  execution remain deferred until a later data-runtime slice).
- [x] Add the bounded direct `select <varlist>` syntax slice (PR #26, merged
  as `5735b43`;
  active-schema lookup, wildcard/range expansion, relation mutation, and
  execution remain deferred until a later data-runtime slice).
- [x] Add the bounded direct `sort <varlist>` syntax slice (PR #27, merged as
  `7cf21ae`; active-schema lookup, row sorting, and execution remain deferred
  until a later data-runtime slice).
- [x] Add the bounded direct `gsort [+|-]varlist` syntax slice (PR #28, merged
  as `fd94133`; direction metadata only, with active-schema lookup, row ordering,
  and execution deferred until a later data-runtime slice).
- [x] Add the bounded direct `save <path> [, replace]` and `export <path> [, replace]`
  syntax slice (PR #29, merged as `8b16223`; filesystem validation, output formats, persistence, and
  execution remain deferred until a later data-runtime slice).
- [x] Add the bounded direct `generate <target> = <expression>` syntax slice
  (PR #45, merged as `63e65ec`; expression/function-call nodes are retained
  syntax-only, with evaluation, type/schema validation, mutation, and runtime
  surfaces deferred to a later data-runtime slice).
- [x] Add the bounded direct `replace <target> = <expression> [if <condition>]`
  syntax slice (PR #47, merged as `87ec017`; target, replacement expression,
  optional condition, nested boundaries, and bounded diagnostics are retained
  syntax-only, while relation mutation, predicate/type semantics, and runtime
  surfaces remain deferred).
- [x] Add the bounded direct `join <table> on <keylist> [, how=inner|left suffix(_right)]`
  syntax slice (PR #59, squash merge `585c53f`; typed table/key/mode/suffix
  parsing and bounded diagnostics are covered; named-table state, SQL, and
  execution remain deferred).
- [x] Add the bounded direct `append <table>` syntax slice (PR #60, squash
  merge `8ab016f`; typed table parsing and bounded diagnostics are covered;
  named-table state, SQL, and execution remain deferred).
- [x] Add the bounded direct `reshape long|wide <varlist>, i(<id_vars>) j(<name>)`
  syntax slice (PR #61, squash merge `e6cc4f1`; typed direction, ordered names,
  duplicate/distinctness validation, and bounded diagnostics are covered;
  relation/session reshape execution remains deferred).
- [x] Add the bounded direct `panel [<id_var> <time_var>|clear]` syntax slice
  (PR #62, squash merge `92e5d5e`; typed report/clear/set actions,
  string/backtick `clear` boundary, distinct entity/time validation, and
  bounded diagnostics are covered; panel metadata and runtime execution
  remain deferred).
- [x] Add the bounded direct `xtdata <varlist>, within|between` syntax slice
  (PR #63, squash merge `74eea07`; typed within/between actions, flag-only
  transform options, quoted variables, exact-one validation, and bounded
  diagnostics are covered; panel metadata and runtime execution remain
  deferred).
- [x] Add the bounded direct `ivregress 2sls|gmm` syntax slice
  (PR #64, squash merge `d355551`; typed 2SLS/GMM estimators, ordered
  exogenous/instrument lists, endog/iv/robust/cluster/noconstant options,
  bounded diagnostics, and parser-only execution deferral are covered;
  estimation and model state remain deferred).
- [x] Add the bounded direct `xtreg <y> <xvars>, fe|re` syntax slice
  (PR #65, squash merge `c88d002`; typed fixed/random-effects estimators,
  ordered predictors, robust/cluster options, bounded diagnostics, and
  parser-only execution deferral are covered; panel metadata and runtime
  execution remain deferred).
- [x] Add the bounded direct `estat firststage|overid|endogenous|hausman`
  syntax slice (PR #66, squash merge `484148e`; typed no-option diagnostic
  subcommands, case/quote normalization, bounded diagnostics, and parser-only
  execution deferral are covered; post-estimation state and calculations remain
  deferred).
- [x] Add the bounded direct `xtabond <y> [xvars]` syntax slice (PR #67,
  squash merge `820376d`; typed ordered predictors, robust flag, bounded
  integer lag options, lag-order validation, exact diagnostics, and parser-only
  execution deferral are covered; panel metadata and dynamic-panel estimation
  remain deferred).
- [x] Add the bounded shared tokenizer API (PR #68, squash merge `45da1ac`;
  owned token kinds/text, Unicode-scalar offsets, recovered identifiers,
  strings, numbers, symbols, exact lexical diagnostics, and delegation from
  existing option/expression consumers are covered; command-specific
  simple-body parsing and broader tokenizer integration remain deferred).
- [x] Add the bounded direct `ttest` syntax slice (PR #69, squash merge
  `4dcd892`; typed value, paired-variable, and `by()` forms, Welch/unequal
  flags, recovered diagnostics, quoted names, and parser-only execution
  deferral are covered; numeric conversion and statistical inference remain
  deferred).
- [x] Add the bounded direct `sql` syntax slice (PR #77, squash merge
  `feeeaf4`; owned `SqlCommand { query, into }`, direct and triple-quoted
  queries, trailing `into <table>` with identifier/reserved-name validation,
  and parser-only execution deferral are covered; multiline script grouping,
  SQL execution, and named-table lifecycle remain deferred).
- [x] Add the bounded direct `regress` syntax slice (PR #79, squash merge
  `e6b81d3`; typed `RegressEstimator` (`ols`, `wls`, `gls`), `RegressCommand`,
  options `robust`, `cluster(<var>)`, `noconstant`, `wls(<var>)`, `gls(<var>)`,
  exact diagnostic parity, and parser-only execution deferral are covered;
  statistical estimation and model state remain deferred).
- [x] Add the bounded direct `logit` and `probit` syntax slice (PR #81, squash
  merge `a14d552`; typed `LogitCommand` and `ProbitCommand`, options `robust`,
  `cluster(<var>)`, `noconstant`, exact diagnostic parity, and parser-only execution
  deferral are covered; statistical estimation and model state remain deferred).
- [ ] Port tokenizer behavior.
- [ ] Port command parsing.
- [ ] Port varlist parsing.
- [ ] Port option parsing.
- [ ] Port `if` clauses.
- [ ] Port expression AST.
- [ ] Port expression precedence.
- [ ] Port function-call syntax.
- [ ] Port quoted/unquoted identifier behavior.
- [ ] Port missing literal semantics.
- [x] Port SQL command boundary.
- [ ] Port prefixed command syntax such as `bayes:`.

### 5.2 Script engine

- [ ] Port `.td` line-oriented execution syntax.
- [ ] Port comments.
- [ ] Port multiline SQL.
- [ ] Port `seed`.
- [ ] Port `let`.
- [ ] Port macro expansion.
- [ ] Port `if` / `else` / `end`.
- [ ] Port nested `run`.
- [ ] Port recursion rejection.
- [ ] Port file/line diagnostics.

### 5.3 Test migration

- [ ] Port parser unit tests.
- [ ] Port parser error tests.
- [ ] Port script tests.
- [ ] Port expression tests.
- [ ] Add property-based parser tests.
- [ ] Add parser fuzz target.
- [ ] Add Python/Rust parse-result differential tests where practical.

### Exit gate

- [ ] All supported command forms parse into typed Rust commands.
- [ ] Script behavior matches Python for canonical fixtures.
- [ ] Parser and script crates contain no unsafe code.
- [ ] No backend is required for syntax-only operations.

---

# 6. Phase 4 - Core Data Runtime

## Goal

Reach end-to-end parity for ordinary data exploration and transformation without statistics.

### 6.1 Session and backend

- [x] Evaluate bounded eager local-Parquet session/adapter slice (PR #22;
  broad session, relation, and load gates below remain unchecked).
- [x] Evaluate bounded eager local-CSV session/adapter slice (PR #74,
  squash merge `7a5b8d4`; broader session, relation, and load gates remain
  unchecked).
- [ ] Implement persistent DuckDB session.
- [ ] Implement active relation.
- [ ] Implement named-table registry.
- [ ] Implement eager load.
- [ ] Implement lazy Parquet scan.
- [ ] Implement remote Parquet.
- [ ] Implement materialization tracking.
- [ ] Implement cached schema metadata with explicit invalidation.
- [ ] Implement row-count known/unknown state.
- [ ] Preserve insertion/order contracts where required.

### 6.2 Load and inspect commands

- [x] `use` — bounded eager local Parquet/CSV source execution (PR #22 and
  PR #74; broader loaders, lazy/materialized behavior, and interface surfaces
  remain deferred).
- [x] `describe` — bounded eager local-Parquet active-metadata execution
  (`DescribeResult`; PR #31, squash merge `6fccd5d`). Broader inspection,
  lazy/materialized behavior, labels, formatting, CLI, and MCP surfaces remain
  deferred.
- [x] `summarize` — bounded eager local-Parquet numeric summary execution
  (`SummarizeResult`/`SummaryRow`; PR #35, squash merge `f2b7ed7`). Broader
  grouped/lazy/materialized behavior, unsupported numeric/container values,
  formatting, CLI, and MCP surfaces remain deferred.
- [x] `codebook` — bounded eager local-Parquet column profiles
  (`CodebookResult`/`CodebookRow`; PR #36, squash merge `bbadf12`). Variable
  labels, lazy/materialized behavior, unsupported logical/container coercion,
  formatting, CLI, and MCP surfaces remain deferred.
- [x] `missing` — bounded eager local-Parquet SQL-NULL missingness execution
  (`MissingResult`/`MissingRow`; PR #37, squash merge `fac2d34`). Lazy/materialized
  behavior, last-operation state, labels, wildcard/range expansion, formatting,
  CLI, JSON, and MCP surfaces remain deferred.
- [x] `duplicates` — bounded eager local-Parquet duplicate-key aggregate
  (`DuplicatesResult`; PR #38, squash merge `6a10039`). NULL-equal groups,
  requested/default key order, checked aggregate metrics, and read-only state
  are covered. Lazy/materialized execution, last-operation state, labels,
  wildcard/range expansion, formatting, CLI, JSON, and MCP remain deferred.
- [x] `isid` — bounded eager local-Parquet key-uniqueness assertion
  (`IsidResult`; PR #39, squash merge `e04def0`). Ordered/repeated key requests,
  SQL-NULL-equal groups, `missok` gating, duplicate-group failures, checked
  aggregate metrics, exact diagnostics, and read-only state are covered.
  Lazy/materialized execution, last-operation state, labels, wildcard/range
  expansion, formatting, CLI, JSON, and MCP remain deferred.
- [x] `datasignature` — bounded eager local-Parquet reproducibility fingerprint
  (`DatasignatureResult`; PR #40, squash merge `9a141da`). The exact SHA-256
  schema/row/value protocol, deterministic order, empty-schema behavior,
  nested temporal hints, interval encoding, and escaped struct names are covered.
  Lazy/materialized execution, `last_operation`, labels/panel metadata, CLI,
  JSON/MCP, direct union values, and broader relation APIs remain deferred.
- [x] `assert` — bounded eager local-Parquet row assertion
  (`AssertResult`; PR #41, squash merge `019ceb1`). The typed expression subset,
  null/failure semantics, checked numeric arithmetic, unsigned guards,
  first-unknown validation, and read-only active-state behavior are covered.
  Lazy/materialized execution, function calls and `e(sample)`,
  `last_operation`, row-level diagnostics, formatting, CLI, JSON/MCP, and
  broader tokenizer/expression parity remain deferred.
- [x] `count` — bounded eager local-Parquet active-dataset row count
  (`CountResult`; PR #32, squash merge `2287fff`). Broader lazy/materialized,
  status, transforms, formatting, CLI, and MCP surfaces remain deferred.
- [x] `head` — bounded eager local-Parquet owned preview execution
  (`PreviewResult`/`CellValue`; PR #33, squash merge `b107251`). Broader
  lazy/materialized behavior, unsupported logical/container values, formatting,
  CLI, and MCP surfaces remain deferred.
- [x] `tail` — bounded eager local-Parquet owned suffix preview execution
  (`PreviewResult`/`CellValue`; PR #34, squash merge `1c5affa`). Broader
  lazy/materialized behavior, unsupported logical/container values, formatting,
  CLI, and MCP surfaces remain deferred.

### 6.3 Transform commands

- [x] `keep` — bounded eager local-Parquet explicit-varlist projection
  (`KeepResult`; PR #42, squash merge `d43c923`). Requested/duplicate column
  order, row order, quoted identifiers, exact bounded diagnostics, staged
  publication, and failure-atomic metadata are covered. Predicate form
  (`keep if`), lazy/materialized execution, wildcard/range expansion,
  overflow/filter semantics, labels, `last_operation`, formatting, CLI, JSON,
  and MCP remain deferred.
- [x] `drop` — bounded eager local-Parquet explicit-varlist complement projection
  (`DropResult`; PR #43, squash merge `50cf80c`). Source-order complement,
  quoted identifiers, duplicate-request behavior, all-column guard, staged
  publication, and failure-atomic metadata/relation are covered. Predicate
  form (`drop if`), lazy/materialized execution, wildcard/range expansion,
  labels, `last_operation`, formatting, CLI, JSON, and MCP remain deferred.
- [x] `select` — bounded eager local-Parquet explicit-varlist projection
  (`SelectResult`; PR #44, squash merge `228fa50`). Requested order,
  duplicate projection naming, quoted identifiers, empty-relation behavior,
  staged publication, and failure-atomic metadata/relation are covered.
  Predicate form (`select if`), lazy/materialized execution, wildcard/range
  expansion, labels/panel metadata, `last_operation`, formatting, CLI, JSON,
  and MCP remain deferred.
- [x] `generate` — bounded eager local-Parquet numeric-column generation
  (`GenerateResult`; PR #46, squash merge `98979bc`). Numeric identifiers and
  literals, unary minus, `+`, `-`, `*`, `/`, quoted identifiers, staged
  publication, typed validation/errors, row/order/schema preservation, empty
  relations, and failure-atomic metadata/relation behavior are covered.
  String/boolean/NULL/comparison expressions, function calls, exact
  overflow-count reporting, lazy/materialized execution, labels/panel metadata,
  `last_operation`, formatting, CLI, JSON, MCP, and broad transform parity
  remain deferred.
- [x] `replace` — bounded eager local-Parquet value replacement
  (`ReplaceResult`; PR #48, squash merge `df2cad9`). Numeric/string domain
  assignments, explicit NULL replacement, typed boolean/missing predicates,
  quoted identifiers, schema-position/row-order preservation, staged
  publication, typed validation/errors, and failure-atomic metadata/relation
  behavior are covered. Function calls, unsupported boolean/other target
  domains, exact overflow-count diagnostics, lazy/materialized execution,
  labels/panel metadata, `last_operation`, formatting, CLI, JSON, MCP, and
  broad transform parity remain deferred.
- [x] `rename` — bounded eager local-Parquet schema rename
  (`RenameResult`; PR #49, squash merge `0bd547f`). Exact source/target
  validation, target-collision and same-name errors, quoted identifiers,
  source-position/type/row-order/NULL preservation, staged publication, and
  failure-atomic metadata/relation behavior are covered. Panel/label metadata,
  lazy/materialized execution, wildcard or multi-column forms, `last_operation`,
  formatting, CLI, JSON, MCP, and broad transform sequencing remain deferred.
- [x] `sort` — bounded eager local-Parquet stable ascending sort
  (`SortResult`; PR #50, squash merge `f33987a`). Native scalar keys sort
  ascending with SQL NULLs last, complete ties preserve prior row order,
  quoted identifiers and private-ordinal collisions are covered, and staged
  publication is failure-atomic. Panel/label metadata, lazy/materialized
  execution, descending keys, `gsort`, expression keys, `last_operation`,
  formatting, CLI, JSON, MCP, and broad transform sequencing remain deferred.
- [x] `gsort` — bounded eager local-Parquet stable directed sort
  (`GsortResult`; PR #51, squash merge `c06ed5a`). Native scalar keys honor
  per-key ascending/descending directions with SQL NULLs last, complete ties
  preserve prior row order, quoted identifiers and private-ordinal collisions
  are covered, and staged publication is failure-atomic. Panel/label metadata,
  lazy/materialized execution, `last_operation`, formatting, CLI, JSON, MCP,
  and broad transform sequencing remain deferred.
- [x] `recode` — bounded eager local-Parquet recode (RecodeResult; PR #52,
  squash merge `2e25cda`). Scalar values, inclusive numeric ranges,
  missing/nonmissing and else rules, ordered first-match behavior,
  generate/replace placement, quoted identifiers, staged publication, and
  failure-atomic relation/metadata behavior are covered. Lazy/materialized
  execution, panel/label metadata, last_operation, formatting, CLI, JSON,
  MCP, and broad transform sequencing remain deferred.
- [x] `encode` — bounded eager local-Parquet integer coding (`EncodeResult`; PR #53,
  squash merge `af3e3b2`). Sorted unique nonmissing string values receive
  one-based codes, NULLs are preserved, quoted/empty relations and
  failure-atomic staged publication are covered, and ordinary encode retains
  session-owned label metadata for the bounded decode and label slices. The
  optional label name selects the generated value-label set. Persistence,
  rendering, lazy/materialized execution, panel metadata, last_operation,
  formatting, CLI, JSON, MCP, and broad transform sequencing remain deferred.
  PR-head workflows `35483588143`/`35483588144` and merge-head workflows
  `35484575373`/`35484575369` passed; documentation-closeout evidence is in
  `_workspace/runtime-label/`.
- [x] `decode` — bounded eager local-Parquet same-session decode
  (`DecodeResult`; PR #54, squash merge `0845e6c`). Encode-produced integer
  mappings decode known codes to strings, preserve NULL/unmapped values as
  NULL, support quoted/empty relations, reconcile provenance across rename and
  projections, and publish atomically. Arbitrary imported value-label
  metadata, DTA labels, persistence, lazy/materialized execution, panel
  metadata, last_operation, formatting, CLI, JSON, MCP, and broad transform
  sequencing remain deferred.
- [x] `label` — bounded eager session-local variable/value labels
  (`LabelResult`; PR #55, squash merge `70b9745`). Variable labels, named
  integer/numeric/text value-label sets, attachments, list filtering, drop,
  atomic validation, encode/decode integration, and reconciliation across
  rename, projections, value-changing replace, recode, and `use` are covered.
  `label save/use` persistence, DTA-imported labels, inspection/reporting
  rendering, lazy/materialized execution, panel metadata, last_operation,
  formatting, CLI, JSON, MCP, and broad transform sequencing remain deferred.

### 6.4 Combine and summarize

- [ ] `join` — runtime remains deferred; the parser boundary is accepted in
  the Phase 5.1 direct-language slice (PR #59, squash merge `585c53f`).
- [ ] `append` — runtime remains deferred; the parser boundary is accepted in
  the Phase 5.1 direct-language slice (PR #60, squash merge `8ab016f`).
- [ ] `reshape` — runtime remains deferred; the parser boundary is accepted in
  the Phase 5.1 direct-language slice (PR #61, squash merge `e6cc4f1`).
- [x] `tabulate` — bounded eager local-Parquet one- and two-way frequency
  tables (TabulateResult; PR #56, squash merge 24405a6). Direct row/column
  forms, count/percent output, row/column percentages, missing categories,
  native deterministic ordering, session-local value-label display, nolabel,
  owned results, and read-only state are covered. values/stat aggregation,
  if predicates, by-prefixes, multi-dimensional forms, named tables,
  lazy/materialized execution, persistence, formatting, CLI, JSON, MCP, and
  broad Python parity remain deferred. PR-head workflows
  35486898316/35486898348 and merge-head workflows
  35487873526/35487873479 passed; detailed evidence is in
  _workspace/runtime-tabulate/.
- [x] `collapse` — bounded eager local-Parquet grouped aggregates
  (CollapseResult; PR #57, squash merge 94391af). Direct `by(...)` forms for
  count/mean/sum/min/max, SQL NULL grouping, deterministic NULL-last ordering,
  non-NULL counts, numeric validation, atomic active-relation replacement,
  owned metadata, and surviving-group label retention are covered. Conditions,
  weights, named tables, lazy/materialized execution, panel propagation,
  persistence, formatting, CLI, JSON, MCP, and broad Python parity remain
  deferred. PR-head workflows 35491989391/35491989384 and merge-head
  workflows 35492943464/35492943539 passed; detailed evidence is in
  _workspace/runtime-collapse/.
- [x] `by` — bounded eager local-Parquet grouped summarize/count
  (ByResult; PR #58, squash merge cc818e5). Typed by-prefix parsing preserves
  grouping and child-command boundaries, supports quoted identifiers, and
  rejects missing delimiters, empty groups, nested by, help, status, and doctor
  children. Grouped summarize defaults to numeric non-group columns, grouped
  count uses COUNT(*), SQL NULL groups are explicit and NULL-last ordered, and
  both read-only forms return owned cells while preserving active state.
  Grouped tabulate, conditions, weights, named tables, lazy/materialized
  execution, panel propagation, persistence, formatting, CLI, JSON, MCP, and
  broad Python by parity remain deferred. PR-head workflows
  35496489763/35496489761 and merge-head workflows
  35497533142/35497533149 passed; detailed evidence is in
  _workspace/runtime-by/.

### 6.5 Persistence and SQL

- [ ] `sql`
- [x] `save` — bounded eager local-Parquet output for an active relation
  (PR #70, squash merge `9bbf804`; detailed contract/evidence in
  `_workspace/runtime-save/`)
- [x] `export` — bounded eager CSV output for an active local-Parquet relation
  (PR #72, squash merge `5cb5b34`; detailed contract/evidence in
  `_workspace/runtime-export-csv/`)
- [x] Parquet output — bounded eager `save` path only (PR #70); broader output
  adapters remain deferred
- [x] CSV output — bounded eager `export` path only (PR #72); broader output
  adapters remain deferred
- [ ] Feather/Arrow output
- [ ] DTA input through ReadStat

The accepted `save` and CSV-only `export` slices validate their output targets,
create missing parent directories, gate replacement, preserve the active session
relation, and cover schema/order/count/NULL values. They are library-only eager
local-Parquet boundaries; Parquet aliasing through `export`, path normalization,
lazy/materialized persistence, metadata/label/panel persistence, Feather/Arrow,
and interface surfaces remain outside this roadmap gate.

### 6.6 Semantic parity

- [ ] Port missingness tests.
- [ ] Port integer overflow tests.
- [ ] Port ordering tests.
- [ ] Port atomic-mutation failure tests.
- [ ] Port label tests.
- [ ] Port panel-metadata preservation/clearing tests.
- [ ] Port eager/lazy consistency tests.
- [ ] Add Python/Rust JSON differential suite for core data commands.

### Exit gate

- [ ] Canonical EDA workflows run fully in Rust.
- [ ] Core workflows require no Python or R.
- [ ] Structured outputs match Python contracts.
- [ ] No known P0/P1 semantic mismatch remains in core data operations.

---

# 7. Phase 5 - CLI, REPL, Reporting, and Machine Interfaces

## Goal

Make the Rust implementation pleasant enough to use interactively and stable enough for automation.

### 7.1 CLI

- [ ] Implement CLI argument parsing.
- [ ] Implement repeated `-c`.
- [ ] Implement `-f`.
- [ ] Implement positional script execution.
- [ ] Implement `--version`.
- [ ] Implement `--json`.
- [ ] Implement `--list-commands`.
- [ ] Implement `--help-topic`.
- [ ] Implement `--explain`.
- [ ] Implement command-effect discovery.

### 7.2 REPL

- [ ] Implement interactive shell.
- [ ] Add persistent history.
- [ ] Add inline history suggestions.
- [ ] Add syntax highlighting.
- [ ] Add context-aware completion.
- [ ] Add multiline support where required.
- [ ] Add responsive Ctrl-C handling.
- [ ] Ensure completion does not materialize active data.
- [ ] Ensure completion does not initialize statistical capabilities.

### 7.3 Reporting

- [ ] Port terminal table rendering.
- [ ] Port deterministic numeric formatting.
- [ ] Port JSON serialization.
- [ ] Port HTML reporting primitives.
- [ ] Ensure one typed result model feeds all renderers.

### 7.4 Visualization

- [ ] Port `histogram`.
- [ ] Port `scatter`.
- [ ] Port `bar`.
- [ ] Generate Vega-Lite specs in Rust.
- [ ] Render SVG.
- [ ] Render PNG.
- [ ] Preserve plot auto-open behavior only where appropriate.
- [ ] Preserve script reproducibility semantics.

### 7.5 MCP

- [ ] Implement MCP server with Rust SDK.
- [ ] Port tools.
- [ ] Port resources.
- [ ] Port prompt templates.
- [ ] Port session status resource.
- [ ] Port schema resource.
- [ ] Port command catalog resource.

### Exit gate

- [ ] Interactive Rust TabDat is usable for daily EDA.
- [ ] JSON workflows match current contracts.
- [ ] MCP baseline works.
- [ ] Plotting has no Python dependency.
- [ ] CLI metadata modes do not initialize heavy runtimes unnecessarily.

---

# 8. Phase 6 - Performance Baseline and Snappiness Gate

## Goal

Make low latency a measurable architectural property before statistical complexity grows.

### 8.1 Benchmark harness

- [ ] Create cold-start benchmark.
- [ ] Create warm-start benchmark.
- [ ] Create parser-only benchmark.
- [ ] Create REPL latency benchmark.
- [ ] Create rendering benchmark.
- [ ] Create DuckDB overhead benchmark.
- [ ] Create memory benchmark.
- [ ] Record Apple Silicon baseline.
- [ ] Record Linux x86_64 baseline.

### 8.2 Initial targets

- [ ] Target `tabdat --version` below 20 ms where platform/tooling permits.
- [ ] Target interactive shell startup below 50-100 ms.
- [ ] Target parser-only commands below 1 ms.
- [ ] Target metadata commands such as `status` and `help` below 5 ms warm.
- [ ] Target trivial TabDat dispatch overhead below 1-2 ms excluding backend work.
- [ ] Validate immediate cancellation behavior.

### 8.3 Architectural optimization

- [ ] Remove unnecessary startup initialization.
- [ ] Ensure DuckDB is not loaded for syntax-only paths where avoidable.
- [ ] Ensure libgretl is not initialized before the first relevant estimator.
- [ ] Cache command metadata statically.
- [ ] Cache schema metadata with correct invalidation.
- [ ] Avoid intermediate generic maps/dictionaries in result rendering.
- [ ] Measure copies across DuckDB/Arrow/statistical boundaries.
- [ ] Eliminate avoidable dataframe materialization.
- [ ] Avoid unsafe optimization unless profiling demonstrates necessity.

### Exit gate

- [ ] Rust version is materially snappier than the Python baseline for common interactions.
- [ ] Performance numbers are recorded in CI artifacts or release reports.
- [ ] No known high-impact startup regression remains.

---

# 9. Phase 7 - Classical Statistics Foundation

## Goal

Port foundational statistical inference and establish the stable estimator backend interface.

### 9.1 Statistical contracts

- [ ] Define `EstimationProblem`.
- [ ] Define `EstimationSample`.
- [ ] Define `CoefficientEstimate`.
- [ ] Define covariance representation.
- [ ] Define model metadata.
- [ ] Define convergence metadata.
- [ ] Define prediction contract.
- [ ] Define post-estimation state contract.
- [ ] Define backend capability trait.
- [ ] Define backend error normalization.

### 9.2 Linear regression

- [ ] Port OLS `regress`.
- [ ] Port WLS.
- [ ] Port current GLS semantics.
- [ ] Port robust covariance.
- [ ] Port clustered covariance.
- [ ] Port `predict, xb`.
- [ ] Port `predict, residuals`.
- [ ] Port VIF.
- [ ] Port residual diagnostics.
- [ ] Port `test`.
- [ ] Port `lincom`.
- [ ] Port `ttest`.
- [ ] Port HTML regression report.

### 9.3 Validation

- [ ] Compare against Python TabDat.
- [ ] Compare against trusted external references.
- [ ] Validate coefficients.
- [ ] Validate standard errors.
- [ ] Validate covariance corrections.
- [ ] Validate predictions.
- [ ] Validate sample exclusion.
- [ ] Validate failure behavior.

### Exit gate

- [ ] `regress` family is production-quality in Rust.
- [ ] Common post-estimation commands are TabDat-owned rather than backend-output wrappers.
- [ ] Statistical adapter initialization is lazy.
- [ ] No FFI types escape statistical adapters.

---

# 10. Phase 8 - Generalized, Limited, Count, and Survival Models

## Goal

Replace the bulk of statsmodels/R-backed conventional estimators.

### 10.1 Commands

- [ ] `qreg`
- [ ] `logit`
- [ ] `probit`
- [ ] `estat margins`
- [ ] binary `predict, xb`
- [ ] binary `predict, pr`
- [ ] `tobit`
- [ ] `heckman`
- [ ] `nl`
- [ ] `poisson`
- [ ] `nbreg`
- [ ] `streg`

### 10.2 ZIP/ZINB

- [ ] Decide whether to use a native backend or implement in Rust.
- [ ] Define zero-inflated likelihood contract.
- [ ] Implement ZIP.
- [ ] Validate ZIP convergence behavior.
- [ ] Validate ZIP covariance.
- [ ] Implement ZINB.
- [ ] Validate NB parameterization.
- [ ] Validate ZINB convergence and covariance.
- [ ] Port `estat gof`.
- [ ] Port relevant prediction modes.

### 10.3 Validation

- [ ] Add three-way tests for every model family.
- [ ] Record known backend differences.
- [ ] Confirm deterministic error behavior.
- [ ] Confirm estimation sample parity.

### Exit gate

- [ ] Routine nonlinear/count/survival workflows no longer require Python or R.
- [ ] ZIP/ZINB parity status is explicitly documented.

---

# 11. Phase 9 - IV, Panel, and Causal Inference

## Goal

Port the econometric command families that currently rely on linearmodels, statsmodels, or R.

### 11.1 IV and panel

- [ ] `panel` — runtime remains deferred; the parser boundary is accepted
  in the Phase 5.1 direct-language slice (PR #62, squash merge `92e5d5e`).
- [ ] `xtdata` — runtime remains deferred; the parser boundary is accepted
  in the Phase 5.1 direct-language slice (PR #63, squash merge `74eea07`).
- [ ] `ivregress 2sls` — runtime remains deferred; the parser boundary is
  accepted in the Phase 5.1 direct-language slice (PR #64, squash merge
  `d355551`).
- [ ] `ivregress gmm` — runtime remains deferred; the parser boundary is
  accepted in the Phase 5.1 direct-language slice (PR #64, squash merge
  `d355551`).
- [ ] `estat firststage` — runtime remains deferred; the parser boundary is
  accepted in the Phase 5.1 direct-language slice (PR #66, squash merge
  `484148e`).
- [ ] `estat overid` — runtime remains deferred; the parser boundary is
  accepted in the Phase 5.1 direct-language slice (PR #66, squash merge
  `484148e`).
- [ ] `estat endogenous` — runtime remains deferred; the parser boundary is
  accepted in the Phase 5.1 direct-language slice (PR #66, squash merge
  `484148e`).
- [ ] `xtreg, fe` — runtime remains deferred; the parser boundary is accepted
  in the Phase 5.1 direct-language slice (PR #65, squash merge `c88d002`).
- [ ] `xtreg, re` — runtime remains deferred; the parser boundary is accepted
  in the Phase 5.1 direct-language slice (PR #65, squash merge `c88d002`).
- [ ] `estat hausman` — runtime remains deferred; the parser boundary is
  accepted in the Phase 5.1 direct-language slice (PR #66, squash merge
  `484148e`).
- [ ] `xtabond` — runtime remains deferred; the parser boundary is accepted in
  the Phase 5.1 direct-language slice (PR #67, squash merge `820376d`).
- [ ] dynamic-panel prediction
- [ ] dynamic-panel overidentification diagnostics

### 11.2 Control function

- [ ] Port `cfregress`.
- [ ] Keep two-stage orchestration in Rust.
- [ ] Port first-stage diagnostics.
- [ ] Port endogenous diagnostics.
- [ ] Port prediction.

### 11.3 Fixed-effects logit

- [ ] Evaluate compiled/native implementation options.
- [ ] If no satisfactory backend exists, implement conditional FE logit in Rust.
- [ ] Define conditional likelihood.
- [ ] Implement optimization.
- [ ] Validate sufficient-statistic conditioning.
- [ ] Validate clustered/robust behavior if supported.
- [ ] Port prediction semantics only where statistically meaningful.

### 11.4 DID and DR-DID

- [ ] Port `did`.
- [ ] Port DID diagnostics.
- [ ] Port DID prediction.
- [ ] Define DR-DID estimating equations.
- [ ] Port `drdid`.
- [ ] Validate influence-function/inference semantics.
- [ ] Compare against trusted causal-inference references.

### Exit gate

- [ ] Standard IV and panel workflows require no Python/R.
- [ ] Dynamic panel is reference validated.
- [ ] DID/DR-DID semantics are explicitly validated.
- [ ] Any remaining FE-logit gap is documented and bounded.

---

# 12. Phase 10 - Regularization and DML

## Goal

Replace sklearn dependencies with lightweight Rust-native learning primitives where practical.

### 12.1 Linear learners

- [ ] Implement ridge.
- [ ] Implement Lasso.
- [ ] Implement elastic net.
- [ ] Decide coordinate-descent vs LARS implementation strategy.
- [ ] Add warm-start support if beneficial.
- [ ] Add deterministic seeding behavior where needed.

### 12.2 Cross-validation

- [ ] Implement fold generation.
- [ ] Implement `cvlasso`.
- [ ] Implement `cvridge`.
- [ ] Implement `cvelasticnet`.
- [ ] Validate scoring semantics.
- [ ] Validate deterministic fold behavior.

### 12.3 Post-Lasso

- [ ] Implement `postlasso`.
- [ ] Reuse canonical OLS inference.
- [ ] Validate selected-variable ordering.
- [ ] Validate empty-selection behavior.

### 12.4 DML

- [ ] Define `NuisanceLearner` trait.
- [ ] Keep sample splitting in TabDat.
- [ ] Keep cross-fitting in TabDat.
- [ ] Keep orthogonal score construction in TabDat.
- [ ] Keep ATE inference in TabDat.
- [ ] Implement `dml linear`.
- [ ] Port `estat dml`.
- [ ] Validate against Python and trusted references.
- [ ] Evaluate optional broader compiled ML backends only after native linear learners are mature.

### Exit gate

- [ ] Current regularization commands require no sklearn.
- [ ] DML uses pluggable learners behind a stable Rust contract.
- [ ] No large C++ ML dependency is added without measured benefit.

---

# 13. Phase 11 - Spatial Capability

## Goal

Replace PySAL/spreg while preserving spatial semantics and keeping the dependency optional if necessary.

### 13.1 Spatial substrate evaluation

- [ ] Prototype libgeoda/GeoDa integration.
- [ ] Design a C ABI shim if required.
- [ ] Isolate all C++ interaction behind the shim.
- [ ] Build on macOS Apple Silicon.
- [ ] Build on Linux x86_64.
- [ ] Verify thread-safety assumptions.
- [ ] Verify ownership and error conversion.

### 13.2 Commands

- [ ] Port spatial weights from coordinates.
- [ ] Port KNN weights.
- [ ] Port spatial diagnostics.
- [ ] Port lag model.
- [ ] Port error model.
- [ ] Port SARAR/GMM if supported.
- [ ] Port same-sample spatial-lag prediction.
- [ ] Port out-of-sample weight alignment/reconstruction.

### 13.3 Validation

- [ ] Differential-test against current PySAL outputs.
- [ ] Validate coefficients.
- [ ] Validate covariance.
- [ ] Validate diagnostics.
- [ ] Validate weights ordering.
- [ ] Validate prediction.

### Exit gate

- [ ] Spatial capability no longer requires Python, or
- [ ] a bounded optional compatibility backend is explicitly retained with rationale.

---

# 14. Phase 12 - Bayesian Capability

## Goal

Determine whether the Bayesian surface can become native without compromising architecture or delaying core stabilization.

### 14.1 Simple Bayesian linear model

- [ ] Evaluate a native conjugate implementation for `bayes linear`.
- [ ] Port posterior summary structures.
- [ ] Port prediction.
- [ ] Port deterministic seeded behavior.

### 14.2 General MCMC

- [ ] Prototype BridgeStan.
- [ ] Evaluate model compilation UX.
- [ ] Evaluate binary-size/toolchain burden.
- [ ] Evaluate runtime initialization.
- [ ] Evaluate cross-platform packaging.
- [ ] Define TabDat posterior-sample representation.
- [ ] Implement R-hat/ESS or select a suitable native implementation.
- [ ] Port posterior prediction.
- [ ] Port trace plot.
- [ ] Port density plot.
- [ ] Port autocorrelation plot.

### 14.3 Product decision

- [ ] Decide whether `bayes:` ships in the standard distribution.
- [ ] Decide whether it becomes an optional feature/package.
- [ ] Decide whether a temporary Python compatibility backend remains justified.
- [ ] Record the decision in an ADR.

### Exit gate

- [ ] Bayesian capability has a clear long-term backend decision.
- [ ] Core TabDat remains independent from Bayesian runtime/toolchain requirements.

---

# 15. Phase 13 - Comprehensive Differential and Reference Validation

## Goal

Establish Rust as statistically and behaviorally trustworthy enough to supersede Python.

### 15.1 Core behavior matrix

- [ ] Run canonical `.td` workflows through Python.
- [ ] Run the same workflows through Rust.
- [ ] Compare JSON outputs.
- [ ] Compare terminal golden outputs where appropriate.
- [ ] Compare state transitions.
- [ ] Compare error behavior.
- [ ] Compare ordering.
- [ ] Compare missingness.
- [ ] Compare output artifacts.

### 15.2 Statistical matrix

For every public estimator:

- [ ] Record backend.
- [ ] Record reference implementation.
- [ ] Record reference version.
- [ ] Record fixture dataset.
- [ ] Record coefficient tolerance.
- [ ] Record SE tolerance.
- [ ] Record diagnostic tolerance.
- [ ] Record prediction tolerance.
- [ ] Record known differences.
- [ ] Record validation date.
- [ ] Mark implementation status separately from validation status.

### 15.3 Cross-platform determinism

- [ ] Validate macOS Apple Silicon.
- [ ] Validate Linux x86_64.
- [ ] Investigate numerically meaningful platform differences.
- [ ] Define deterministic-output tolerances.
- [ ] Ensure deterministic formatting even when floating computation differs slightly.

### Exit gate

- [ ] All Tier-1/core estimators are reference validated.
- [ ] Every public estimator has explicit validation status.
- [ ] Core workflows have no unexplained Python/Rust divergence.
- [ ] Known intentional differences are documented.

---

# 16. Phase 14 - Packaging and Native Distribution

## Goal

Ship TabDat as a normal native CLI without requiring Python or R for standard use.

### 16.1 Build and packaging

- [ ] Define supported targets.
- [ ] Produce macOS Apple Silicon build.
- [ ] Produce Linux x86_64 build.
- [ ] Evaluate Linux aarch64.
- [ ] Evaluate Windows feasibility.
- [ ] Decide static vs dynamic linkage per dependency.
- [ ] Audit native library redistribution requirements.
- [ ] Audit AGPL/GPL compatibility and source obligations.
- [ ] Bundle required license notices.
- [ ] Test installation into a clean environment.
- [ ] Test execution outside the source checkout.

### 16.2 User installation

- [ ] Design one-command installer.
- [ ] Evaluate Homebrew formula.
- [ ] Evaluate cargo installation only if native dependency UX is acceptable.
- [ ] Document upgrade.
- [ ] Document uninstall.
- [ ] Document optional capabilities.

### 16.3 Distribution metrics

- [ ] Record binary size.
- [ ] Record installed footprint.
- [ ] Record dependency count.
- [ ] Record cold startup.
- [ ] Record warm startup.
- [ ] Record first-statistics-command initialization.
- [ ] Record first-plot initialization.

### Exit gate

- [ ] Standard TabDat installation requires no Python or R.
- [ ] Clean install completes canonical EDA/statistical workflow.
- [ ] Installation is materially simpler than the Python dependency stack.

---

# 17. Phase 15 - Python-to-Rust Cutover

## Goal

Make Rust the authoritative production implementation.

### 17.1 Cutover readiness

- [ ] Define required parity subset.
- [ ] Define allowed deferred capability gaps.
- [ ] Confirm migration documentation is complete.
- [ ] Confirm statistical validation gates pass.
- [ ] Confirm performance gates pass.
- [ ] Confirm packaging gates pass.
- [ ] Confirm no P0/P1 correctness issue remains.
- [ ] Confirm unsafe audit passes.

### 17.2 Transition

- [ ] Announce Rust implementation as the primary runtime.
- [ ] Freeze new Python feature development.
- [ ] Keep Python available temporarily as a reference oracle.
- [ ] Redirect normal installation/documentation to Rust.
- [ ] Archive Python-only implementation notes where appropriate.
- [ ] Preserve historical tests/fixtures needed for regression validation.

### 17.3 Retirement criteria

- [ ] Rust has covered all required core workflows.
- [ ] Rust has passed multiple stable releases.
- [ ] Remaining Python-only features are either ported, intentionally removed, or optionalized.
- [ ] No critical differential-test dependence on executing Python remains.
- [ ] Python implementation is formally moved to maintenance/archive status.

### Exit gate

- [ ] Rust is the canonical TabDat implementation.

---

# 18. Phase 16 - Ecosystem Extraction and Upstream Contribution

## Goal

Contribute reusable components discovered during the Rust migration back to the broader ecosystem.

### Candidate extractions

- [ ] Evaluate extracting conditional FE logit as a reusable crate.
- [ ] Evaluate extracting ZIP/ZINB primitives as a reusable crate.
- [ ] Evaluate extracting covariance/inference utilities.
- [ ] Evaluate extracting statistical reference-validation tooling.
- [ ] Evaluate upstreaming bug fixes to DuckDB Rust bindings.
- [ ] Evaluate upstreaming ReadStat wrapper improvements.
- [ ] Evaluate upstreaming libgretl binding improvements.
- [ ] Evaluate upstreaming GeoDa/libgeoda wrapper improvements.
- [ ] Document generic FFI lessons where useful.
- [ ] Avoid premature extraction before APIs stabilize in TabDat.

### Exit gate

- [ ] Reusable components are extracted only when their interfaces have proven stable.
- [ ] Upstream contributions reduce long-term TabDat maintenance burden.

---

# 19. Release Quality Gates

For every release candidate:

- [ ] `cargo fmt --check` passes.
- [ ] Clippy passes with project-defined deny rules.
- [ ] Unit tests pass.
- [ ] Integration tests pass.
- [ ] Differential tests pass.
- [ ] Reference statistical tests pass for required tiers.
- [ ] `cargo audit` passes or reviewed exceptions are documented.
- [ ] `cargo deny` passes.
- [ ] Unsafe usage report is reviewed.
- [ ] No unexpected unsafe appears outside approved crates.
- [ ] Cold-start benchmark is recorded.
- [ ] Warm-start benchmark is recorded.
- [ ] REPL benchmark is recorded.
- [ ] Peak memory benchmark is recorded.
- [ ] Clean-install smoke test passes.
- [ ] Help/JSON/MCP contracts remain aligned.
- [ ] Changelog is updated.
- [ ] Relevant ADRs are updated.

---

# 20. Definition of Done

The Rust migration is complete when:

- [ ] Rust is the canonical production implementation.
- [ ] Core TabDat has zero Python dependency.
- [ ] Core TabDat has zero R dependency.
- [ ] Common statistical/econometric workflows have zero Python/R dependency.
- [ ] Specialized optional capabilities have explicit backend decisions.
- [ ] Standard installation is native and reproducible.
- [ ] Interactive startup and REPL latency are materially improved.
- [ ] Unsafe code is confined to approved low-level boundaries.
- [ ] Raw FFI types do not escape into application/domain code.
- [ ] Statistical results are backed by tracked reference validation.
- [ ] Python has transitioned from production runtime to historical/reference implementation.
- [ ] The resulting architecture is simpler, safer, and more maintainable than the Python implementation it replaces.
