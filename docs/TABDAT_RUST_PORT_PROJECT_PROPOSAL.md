# TabDat Explore Rust Port: Project Proposal

**Status:** Proposed  
**Date:** 2026-09-07  
**Project:** TabDat Explore Rust Port  
**Relationship to existing project:** Rust-native successor to the current Python implementation of TabDat Explore

---

## 1. Executive Summary

TabDat Explore is a terminal-native exploratory data analysis and statistical computing environment with a Stata-inspired command language, modern tabular data support, scriptable workflows, structured machine interfaces, statistical estimation, visualization, and MCP integration.

The current Python implementation has successfully validated the product concept and accumulated substantial value in:

- command semantics;
- language behavior;
- session-state design;
- statistical functionality;
- reference-validation fixtures;
- scripts and automation behavior;
- interactive UX;
- documentation;
- error handling;
- installation and release experience;
- real-world product lessons.

The proposed project is a deliberate Rust-native successor rather than a mechanical source translation.

The Rust port will preserve the proven TabDat language and user-facing semantics while redesigning the runtime around:

- safe and idiomatic Rust;
- strong domain types and explicit state;
- a lightweight, low-latency CLI and REPL;
- DuckDB as the canonical tabular execution engine;
- narrowly isolated native FFI adapters for mature compiled statistical libraries;
- zero Python or R dependency in the core distribution;
- explicit capability boundaries for specialized subsystems;
- differential validation against the existing Python implementation and trusted external references;
- measurable startup, interaction-latency, memory, and throughput targets;
- minimal and tightly governed `unsafe` usage.

The Python repository will serve as a behavioral specification, migration oracle, reference implementation, test source, and product-design record during the transition.

The long-term goal is not simply "TabDat rewritten in Rust." The goal is a modern statistical systems application whose core qualities are:

> Fast to start, fast to interact with, safe by default, deterministic where promised, easy to distribute, easy to automate, and statistically trustworthy.

---

## 2. Motivation

### 2.1 Why port now

The Python implementation has reached a level of maturity where the dominant challenges are increasingly architectural rather than exploratory.

The project has already demonstrated that:

- the command language is viable;
- terminal-native statistical EDA is useful;
- the current feature families are coherent enough to stabilize;
- the scripting and machine interfaces have clear product value;
- DuckDB is a strong execution substrate;
- a broad statistical command surface is achievable;
- reference validation can be treated as a first-class project discipline.

At this point, the remaining concerns increasingly involve:

- runtime and startup overhead;
- dependency breadth;
- installation complexity;
- capability isolation;
- maintainability of large dispatcher/backend modules;
- strong enforcement of state invariants;
- native distribution;
- predictable memory behavior;
- low-latency interactivity;
- long-term ecosystem contribution.

Rust is well aligned with those goals.

### 2.2 Why not simply optimize the Python version

The Python implementation already delegates much heavy computation to native engines such as DuckDB, Polars, NumPy/SciPy-backed libraries, and compiled statistical packages. Therefore, the Rust port is not justified by a simplistic claim that "Rust loops are faster."

The stronger arguments are:

- lower application and orchestration overhead;
- much faster cold startup potential;
- lower interactive latency;
- simpler deployment as a compiled application;
- tighter control over capability initialization;
- fewer Python-to-native and dataframe representation transitions;
- explicit ownership and lifetime management;
- stronger type-level enforcement of command and state invariants;
- safer concurrency;
- cleaner FFI boundaries;
- greater potential for standalone binaries and predictable runtime environments.

### 2.3 Strategic OSS contribution

Rust has strong infrastructure for:

- systems programming;
- data formats;
- Arrow;
- DuckDB;
- high-performance linear algebra;
- parsers;
- CLIs;
- serialization;
- asynchronous systems;
- MCP integration.

Its statistical and econometric ecosystem is comparatively less mature.

TabDat can therefore contribute not only as an application but potentially as a source of reusable Rust components for:

- estimation contracts;
- covariance and inference routines;
- conditional fixed-effects logit;
- ZIP/ZINB estimation;
- statistical diagnostics;
- reference-validation infrastructure;
- safe wrappers over mature compiled statistical engines.

The project should reuse high-quality existing numerical libraries whenever possible and reinvent only the semantic or statistical layers that are genuinely missing.

---

## 3. Product Vision

### 3.1 User experience

A normal interactive workflow should look like:

```text
$ tabdat
tabdat> use data/analytic.parquet, lazy
tabdat> describe
tabdat> summarize age bmi
tabdat> tabulate treatment
tabdat> regress outcome treatment age bmi, robust
tabdat> predict yhat
tabdat> export results/analytic.parquet
```

The intended experience is:

- near-instant startup;
- immediate command parsing and dispatch;
- highly responsive autocomplete and help;
- low overhead around DuckDB operations;
- no loading of unrelated statistical runtimes;
- clear feedback when computation is actually expensive;
- immediate and reliable cancellation behavior;
- consistent human and machine interfaces.

### 3.2 Automation experience

The same language and semantics should be available to automated callers:

```bash
tabdat --json \
  -c "use data.parquet" \
  -c "summarize age bmi"
```

Human-readable output, JSON output, scripts, and MCP tools must derive from the same underlying typed command and result semantics.

### 3.3 Product positioning

The Rust-native TabDat should distinguish itself through the combination of:

- terminal-first statistical workflows;
- a compact Stata-inspired command language;
- modern Parquet/Arrow/DuckDB data handling;
- native compiled runtime;
- lightweight distribution;
- strong statistical validation;
- structured machine interfaces;
- explicit capability boundaries;
- fast interactive iteration.

It should not aim to become a complete Stata, SAS, SPSS, R, or Python replacement.

---

## 4. Core Design Principles

### 4.1 Behavioral continuity over source continuity

The Python repository is a specification source, not a file-layout template.

Migration should preserve:

- public command semantics;
- documented behavior;
- error behavior;
- output contracts;
- script semantics;
- state transitions;
- statistical interpretation.

Migration should not preserve:

- accidental Python-specific module boundaries;
- dynamic patterns that are unnecessary in Rust;
- monolithic executor/backend organization;
- object representations that exist only because of Python libraries.

### 4.2 Rust-first domain modeling

Rust types should express the domain directly.

Prefer:

```rust
enum EstimationState {
  None,
  Linear(LinearModelState),
  Binary(BinaryModelState),
  Iv(IvModelState),
  Panel(PanelModelState),
}
```

over multiple independently nullable model fields.

Prefer typed command/result structures over stringly typed dictionaries.

Prefer enums for finite semantic choices:

```rust
enum LinearEstimatorKind {
  Ols,
  Wls,
  Gls,
}
```

### 4.3 Safe by default

All ordinary application and domain code should forbid unsafe Rust.

Unsafe code is permitted only when a boundary genuinely requires it, primarily:

- C FFI;
- C++ C-ABI shims;
- Fortran-access wrappers through C interfaces;
- native callback plumbing;
- rare, separately justified low-level optimizations.

The preferred topology is:

```text
safe TabDat application
        ↓
safe capability facade
        ↓
small unsafe wrapper
        ↓
C ABI
        ↓
native compiled library
```

### 4.4 Narrow FFI boundaries

Raw pointers, native handles, and foreign ownership must not escape low-level adapter crates.

Foreign results should be converted promptly into owned TabDat Rust types.

For small statistical result objects, bounded copying is preferable to complicated borrowed foreign lifetimes.

Zero-copy should be pursued primarily where it has material value and a mature ownership protocol, such as Arrow-compatible large buffers.

### 4.5 Statistical semantics belong to TabDat

A command such as:

```text
regress y x1 x2, robust
```

means "execute the TabDat `regress` contract," not "print what a particular backend prints."

Backends may compute:

- coefficients;
- covariance matrices;
- likelihood quantities;
- residuals;
- model-specific diagnostics;
- fitted values.

TabDat should own common semantics such as:

- estimation sample construction;
- missing-row exclusion;
- covariance labeling;
- model state;
- `predict`;
- `test`;
- `lincom`;
- margins;
- common reporting;
- structured output;
- error conventions;
- post-estimation compatibility.

### 4.6 Performance is a product invariant

The Rust port should explicitly optimize for:

- cold startup;
- warm command latency;
- REPL responsiveness;
- memory efficiency;
- low application overhead;
- rapid cancellation;
- lazy capability loading.

The project should distinguish:

```text
total command latency
=
TabDat overhead
+
backend computation
+
rendering
```

and benchmark those components separately.

---

## 5. Proposed High-Level Architecture

```text
tabdat
│
├── tabdat-language
│     parser
│     AST
│     command syntax
│     scripts
│     macro expansion
│     command schemas
│
├── tabdat-domain
│     domain types
│     command types
│     result types
│     errors
│     model state
│     metadata
│
├── tabdat-execution
│     dispatcher
│     session context
│     command handlers
│     capability routing
│     post-estimation state
│
├── tabdat-data
│     DuckDB backend
│     active relation
│     named tables
│     lazy execution
│     relational transformations
│
├── tabdat-io
│     Parquet
│     CSV
│     Arrow/Feather
│     Stata DTA via ReadStat
│
├── tabdat-stats
│     statistical contracts
│     inference
│     diagnostics
│     covariance
│     native algorithms
│
├── tabdat-gretl
│     safe Rust facade
│
├── tabdat-gretl-sys
│     low-level FFI
│     unsafe allowed
│
├── tabdat-ml
│     regularization
│     cross-validation
│     DML nuisance learners
│
├── tabdat-spatial
│     spatial weights
│     spatial diagnostics
│     optional native adapter
│
├── tabdat-bayes
│     optional Bayesian capability
│
├── tabdat-viz
│     Vega-Lite specifications
│     SVG/PNG conversion
│
├── tabdat-reporting
│     terminal rendering
│     JSON
│     HTML
│
├── tabdat-shell
│     REPL
│     history
│     completion
│     highlighting
│
└── tabdat-mcp
      MCP server
```

Exact crate boundaries should be validated through implementation experience and ADRs.

The key architectural requirement is directional dependency flow and isolation of specialized runtimes.

---

## 6. Data and Execution Backend

### 6.1 Canonical engine

DuckDB should be the canonical tabular execution engine for the initial Rust implementation.

Responsibilities include:

- active relation management;
- Parquet/CSV/Arrow access;
- remote Parquet;
- filtering;
- projection;
- grouping;
- aggregation;
- joins;
- append;
- reshape support where appropriate;
- SQL;
- export;
- lazy scanning.

### 6.2 Polars

Polars should not be automatically replicated as a second execution path during the initial port.

It may be reintroduced later if:

- benchmarks show a meaningful advantage;
- semantics can remain consistent;
- the maintenance cost is justified.

### 6.3 Stata DTA

ReadStat is the preferred candidate for native Stata `.dta` ingestion and label handling.

The adapter should translate ReadStat structures directly into TabDat/DuckDB-compatible representations without pandas.

---

## 7. Statistical Backend Strategy

### 7.1 General strategy

Use a layered approach:

```text
TabDat statistical semantics
        ↓
EstimatorBackend trait
        ↓
 ┌─────────────┬──────────────┐
 │             │              │
native Rust   libgretl      specialized backend
```

### 7.2 libgretl

`libgretl` is the leading candidate for the broad classical/econometric backend because its compiled C API overlaps strongly with current TabDat functionality.

Candidate coverage includes:

- OLS;
- WLS;
- robust covariance;
- clustered covariance;
- quantile regression;
- logit;
- probit;
- Tobit;
- Heckman selection;
- Poisson;
- negative binomial;
- duration models;
- IV/2SLS;
- GMM;
- panel FE/RE;
- dynamic panel GMM;
- nonlinear models.

### 7.3 Native Rust statistical ownership

Some features are better implemented directly in Rust because they are bounded and integral to TabDat semantics:

- common covariance/inference utilities;
- `test`;
- `lincom`;
- t-tests;
- prediction routing;
- margins;
- diagnostics;
- regularized linear models;
- cross-validation;
- DID;
- DML orchestration;
- control-function regression orchestration;
- ZIP/ZINB if no satisfactory native backend exists;
- conditional FE logit if required;
- post-estimation state and reporting.

### 7.4 Numerical libraries

Use mature numerical substrates rather than rebuilding kernels.

Candidates include:

- `faer` or BLAS/LAPACK for linear algebra;
- CBLAS/LAPACKE for portable compiled linear algebra;
- NLopt for general optimization;
- Ipopt for specialized constrained optimization where justified.

Avoid adding overlapping native dependencies without clear value.

### 7.5 Spatial

Evaluate GeoDa/libgeoda behind a narrow C ABI shim.

Spatial migration should be treated as a separate capability with dedicated parity tests against the current PySAL behavior.

### 7.6 Bayesian

Bayesian MCMC should be deferred until the core and classical statistical system are mature.

BridgeStan is a plausible compiled substrate, but it does not directly replace the complete Bambi/PyMC/ArviZ experience.

The Bayesian capability should therefore remain optional and must not block the Rust core.

---

## 8. Unsafe Rust Policy

### 8.1 Project-wide invariant

All ordinary TabDat crates should contain:

```rust
#![forbid(unsafe_code)]
```

where practical.

Unsafe-enabled crates should be explicitly listed and limited to FFI or exceptional low-level work.

### 8.2 Unsafe requirements

Every unsafe block must:

- be narrowly scoped;
- establish one clear safety invariant;
- include an adjacent `// SAFETY:` explanation;
- expose a safe API upward;
- prevent raw foreign ownership from leaking upward;
- avoid unwinding across FFI boundaries;
- document thread-safety assumptions;
- use RAII wrappers for owned native resources.

### 8.3 Unsafe anti-patterns

Unsafe must not be used merely to:

- bypass inconvenient borrow checking;
- avoid ordinary bounds checks without evidence;
- implement manual memory management for normal Rust data;
- use unchecked indexing for speculative micro-optimizations;
- force `Send`/`Sync` without upstream guarantees;
- expose raw pointers in public TabDat APIs.

### 8.4 Measurable safety targets

Target invariants:

```text
unsafe blocks outside approved low-level crates: 0
unsafe impl outside approved low-level crates: 0
raw foreign pointers in public TabDat APIs: 0
```

Use tooling such as:

- Clippy;
- `cargo audit`;
- `cargo deny`;
- `cargo geiger`;
- sanitizers where practical;
- fuzzing for parsers and FFI conversion boundaries.

---

## 9. Migration Methodology

The migration should proceed as contract recovery and re-expression rather than code translation.

### 9.1 Migration pipeline

```text
existing Python behavior
        ↓
extract semantic contract
        ↓
design Rust domain API
        ↓
migrate/reuse tests
        ↓
implement backend
        ↓
differential validation
        ↓
cut over
```

### 9.2 Component migration artifact set

Each migrated component should have:

1. **Python contract**
   - current behavior;
   - current signatures;
   - state dependencies;
   - edge cases.

2. **Rust contract**
   - typed API;
   - ownership model;
   - errors;
   - capability requirements.

3. **Test contract**
   - reusable tests;
   - golden outputs;
   - property tests;
   - differential tests.

4. **Implementation mapping**
   - native Rust;
   - DuckDB;
   - libgretl;
   - FFI;
   - optional capability;
   - new implementation required.

### 9.3 Python as oracle

During migration:

- Python TabDat remains runnable;
- canonical fixtures are shared;
- identical `.td` workflows are executed through Python and Rust;
- structured JSON output is compared;
- deviations require explanation;
- statistical output is additionally checked against a trusted reference.

Python should not be assumed automatically correct.

---

## 10. Test and Validation Strategy

### 10.1 Unit tests

Port and adapt unit tests for:

- parser behavior;
- AST construction;
- expression semantics;
- identifiers;
- missingness;
- coercion;
- ordering;
- command validation;
- state transitions;
- output serialization;
- error behavior.

### 10.2 Golden tests

Use stable golden fixtures for:

- terminal output;
- JSON;
- help topics;
- command discovery;
- scripts;
- reports.

### 10.3 Differential Python/Rust tests

For compatible commands:

```text
fixture
  ↓
Python TabDat → canonical result
  ↓
Rust TabDat   → canonical result
  ↓
semantic comparison
```

### 10.4 Three-way statistical validation

For statistical commands:

```text
Python TabDat
Rust TabDat
trusted reference
```

Compare:

- coefficients;
- covariance;
- standard errors;
- confidence intervals;
- predictions;
- estimation sample size;
- diagnostic statistics;
- convergence status;
- failure behavior.

### 10.5 Property-based and fuzz testing

High-value targets include:

- parser;
- expression compiler;
- identifier handling;
- serialization;
- script parser;
- FFI conversions;
- shape and ownership checks;
- edge-case numeric inputs.

---

## 11. Performance and UX Goals

### 11.1 Performance philosophy

The Rust version should target a noticeably snappier interactive experience than conventional statistical packages and the existing Python implementation.

The primary focus is:

1. interaction latency;
2. startup latency;
3. application overhead;
4. memory behavior;
5. computational throughput.

### 11.2 Initial aspirational targets

These are engineering targets, not public guarantees:

| Operation | Initial warm target |
|---|---:|
| `tabdat --version` | <20 ms |
| interactive shell startup | <50-100 ms |
| parser-only command | <1 ms |
| `help` / `status` | <5 ms |
| trivial dispatch overhead | <1-2 ms excluding backend |
| autocomplete response | imperceptible |
| Ctrl-C reaction | immediate |

Targets should be revised using cross-platform benchmark data.

### 11.3 Lazy capability initialization

Core commands must not initialize unrelated capabilities.

Examples:

```text
help
  → no DuckDB if unnecessary
  → no statistics
  → no plotting
  → no MCP

summarize
  → DuckDB
  → no libgretl
  → no spatial
  → no Bayes

regress
  → initialize statistics capability once
```

### 11.4 REPL latency benchmark

Maintain a representative benchmark flow:

```text
startup
use data.parquet
describe
head
summarize x
summarize x y
tabulate group
generate z = x / y
summarize z
regress y x z
predict yhat
estat vif
status
exit
```

Track:

- cold startup;
- warm startup;
- per-command wall time;
- TabDat overhead;
- backend time;
- render time;
- peak RSS;
- allocation behavior;
- first-use capability penalty.

Performance regressions should eventually become CI-gated.

---

## 12. Packaging and Distribution

### 12.1 Core distribution

Target:

- no Python runtime;
- no R runtime;
- no virtual environment;
- no user-managed Python dependency graph;
- straightforward compiled installation.

### 12.2 Capability packaging

Prefer a minimal standard installation.

Potential conceptual packaging:

```text
tabdat
tabdat-spatial
tabdat-bayes
```

or feature-gated equivalents.

Do not fragment the user experience prematurely. Packaging decisions should follow:

- binary-size measurements;
- dynamic-library burden;
- startup impact;
- platform coverage;
- licensing;
- installation reliability.

### 12.3 Platforms

Initial supported platforms should prioritize:

- macOS Apple Silicon;
- Linux x86_64;
- Linux aarch64 where feasible.

Windows should be evaluated once the native dependency story is sufficiently stable.

---

## 13. Licensing

TabDat is AGPL-licensed.

All linked native dependencies must receive an explicit license compatibility review.

Particular attention should be paid to:

- libgretl;
- ReadStat;
- DuckDB;
- BLAS/LAPACK distribution;
- spatial backends;
- optimization libraries;
- Bayesian components.

Binary distribution must include required notices and comply with all corresponding-source obligations.

---

## 14. Risks

### 14.1 Statistical semantic mismatch

A native backend may implement an estimator with subtly different:

- defaults;
- covariance corrections;
- missingness;
- optimization;
- convergence rules;
- factor handling;
- parameterization.

Mitigation:

- TabDat-owned contracts;
- differential tests;
- trusted reference validation;
- explicit backend normalization.

### 14.2 FFI complexity

Native libraries may introduce:

- ownership hazards;
- thread-safety limitations;
- build-system complexity;
- ABI differences;
- callback complexity.

Mitigation:

- C-shaped boundaries;
- safe facade crates;
- tiny unsafe regions;
- RAII;
- platform CI;
- adapter-specific tests.

### 14.3 Dependency bloat

Replacing Python with too many native dependencies can create a worse system.

Mitigation:

> Every native dependency must eliminate substantially more complexity than it introduces.

### 14.4 Premature parity pressure

Demanding immediate 100% feature parity could derail the core rewrite.

Mitigation:

- prioritize common workflows;
- cut over by capability family;
- defer Bayes/spatial when necessary;
- maintain Python as oracle during transition.

### 14.5 Architecture cargo culting

Replicating Python filenames/modules could reproduce existing concentration problems.

Mitigation:

- preserve conceptual boundaries;
- redesign physical boundaries for Rust;
- require ADRs for major crate/dependency decisions.

---

## 15. Success Criteria

The Rust-port initiative is successful when:

- core TabDat runs without Python or R;
- common EDA workflows are behaviorally compatible;
- the Rust shell is materially faster to start and interact with;
- DuckDB remains the canonical tabular execution substrate;
- command semantics and machine interfaces are stable;
- common statistical estimators pass reference validation;
- unsafe Rust is isolated to approved low-level interoperability boundaries;
- no raw FFI types escape into domain/application code;
- installation is simpler than the Python version;
- memory and latency regressions are continuously tracked;
- Python can eventually be retired as the production runtime.

The ideal end state is:

> A safe, compact, native statistical runtime whose behavior is informed by the mature Python implementation but whose architecture fully embraces Rust.

---

## 16. Non-Goals

The initial Rust port will not:

- reproduce every Python dependency internally;
- pursue broad Stata compatibility;
- add major new estimator families during migration;
- require immediate Bayes parity;
- require immediate spatial parity;
- preserve Python module organization;
- optimize every kernel before profiling;
- use unsafe Rust for speculative performance;
- create multiple execution engines without evidence they improve the product;
- turn TabDat into a general-purpose Rust dataframe library.

---

## 17. Recommended First Proof

Before committing to the complete port, build four focused technical spikes:

```text
spike-duckdb
spike-readstat
spike-gretl
spike-vl-convert
```

The decisive statistical spike should reproduce representative existing TabDat fixtures for:

```text
regress
regress, robust
regress, cluster()
logit
probit
qreg
tobit
poisson
nbreg
ivregress 2sls
xtreg, fe
xtreg, re
xtabond
heckman
streg
```

Requirements:

- Rust-owned result structures;
- no foreign model structures above the adapter;
- narrow unsafe surface;
- documented memory ownership;
- successful differential comparison;
- benchmarked startup and first-use cost.

If those spikes succeed, the principal technical uncertainty behind the Rust port will be substantially reduced.
