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
- [ ] Create `readstat-sys`-style low-level bindings.
- [ ] Create safe Rust facade prototype.
- [ ] Read representative Stata `.dta` files.
- [ ] Preserve variable labels.
- [ ] Preserve value labels.
- [ ] Validate missing-value behavior.
- [ ] Convert data without pandas.
- [ ] Confirm all unsafe code is confined to the low-level adapter.

### 3.3 libgretl spike

- [ ] Build/link libgretl on macOS Apple Silicon.
- [ ] Build/link libgretl on Linux x86_64.
- [ ] Create low-level binding crate.
- [ ] Create safe wrapper crate.
- [ ] Define Rust-owned `EstimationProblem`.
- [ ] Define Rust-owned `EstimationResult`.
- [ ] Prevent `MODEL`, `DATASET`, or raw handles from escaping the adapter.
- [ ] Implement OLS fixture.
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
- [ ] Compare results against Python TabDat.
- [ ] Compare results against trusted reference outputs.
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

- [ ] Create `tabdat-language`.
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
- [ ] Port SQL command boundary.
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

- [ ] `use`
- [ ] `describe`
- [ ] `summarize`
- [ ] `codebook`
- [ ] `missing`
- [ ] `duplicates`
- [ ] `isid`
- [ ] `datasignature`
- [ ] `assert`
- [ ] `count`
- [ ] `head`
- [ ] `tail`

### 6.3 Transform commands

- [ ] `keep`
- [ ] `drop`
- [ ] `select`
- [ ] `generate`
- [ ] `replace`
- [ ] `rename`
- [ ] `sort`
- [ ] `gsort`
- [ ] `recode`
- [ ] `encode`
- [ ] `decode`
- [ ] `label`

### 6.4 Combine and summarize

- [ ] `join`
- [ ] `append`
- [ ] `reshape`
- [ ] `tabulate`
- [ ] `collapse`
- [ ] `by`

### 6.5 Persistence and SQL

- [ ] `sql`
- [ ] `save`
- [ ] `export`
- [ ] Parquet output
- [ ] CSV output
- [ ] Feather/Arrow output
- [ ] DTA input through ReadStat

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

- [ ] `panel`
- [ ] `xtdata`
- [ ] `ivregress 2sls`
- [ ] `ivregress gmm`
- [ ] `estat firststage`
- [ ] `estat overid`
- [ ] `estat endogenous`
- [ ] `xtreg, fe`
- [ ] `xtreg, re`
- [ ] `estat hausman`
- [ ] `xtabond`
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
