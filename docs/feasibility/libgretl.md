# libgretl feasibility reconnaissance

Hosted workflow results cited in this report are historical. GitHub Actions are
disabled and workflow definitions are removed under
[ADR 0009](../adr/0009-disable-github-actions.md).

- Status: **partial**; the pinned native library builds on macOS Apple Silicon and
  Linux x86_64. The later sections record an isolated Rust OLS binding and reference
  fixture; no product estimator backend is accepted.
- Candidate: gretl `2026b`, libgretl ABI version `55.0.0`.
- Reproducible source artifact:
  `gretl-2026b.tar.xz`, SHA-256
  `fb57f4922da546067c8be542aafc5a26be77faf2668b40e651ee8c90b702563d`.
- Upstream: [gretl-project/gretl](https://github.com/gretl-project/gretl) and its
  [SourceForge release archive](https://sourceforge.net/projects/gretl/files/gretl/2026b/).
- Decision record: [ADR 0005](../adr/0005-libgretl-feasibility.md).

## Scope and observed contract

This slice only answers whether a pinned libgretl source release can be built and
linked against a small native smoke program. It does not add a Cargo dependency,
expose `MODEL`/`DATASET` handles, implement OLS or other estimators in Rust, or
claim statistical parity.

The upstream headers expose process-level `libgretl_init` and
`libgretl_cleanup`, broad C structs such as `MODEL` and `DATASET`, matrix/data
helpers, and estimation-facing APIs. Initialization and cleanup are effects with
process/global implications; a future adapter must make them explicit, lazy, and
failure-safe rather than allowing native handles to leak into domain/application
state.

## Build and test evidence

| Platform | Result | Exact scope |
| --- | --- | --- |
| macOS Apple Silicon | Pass, local | gretl `2026b` static `libgretl-1.0` build with Homebrew dependencies and explicit Apple Clang/libomp flags; manually linked `tests/nistcheck.c`; 11 NIST datasets reported zero unexpected errors and zero poor/unacceptable libgretl results |
| Linux x86_64 | Pass ([PR #8 workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/34201655560)) | Pinned archive, static library, `make -C tests check`, and 11 NIST datasets; the workflow passed with zero unexpected errors and zero poor/unacceptable libgretl results |

The macOS library build used the following bounded configuration choices:
`--disable-gui --disable-json --disable-nls --disable-build-addons
--disable-xdg-utils --disable-sse2 --disable-avx`, Homebrew glib/libxml2/FFTW/GMP/MPFR,
BLAS/LAPACK, curl, and libomp, and `CFLAGS="-Xclang -fopenmp ..."` with
`OMP_LIB="-L.../libomp/lib -lomp"`. The release configure script reported
OpenMP disabled even though `gretl_matrix.c` still calls `omp_set_num_threads`;
plain disabled-OpenMP compilation therefore fails, while the explicit Apple
Clang/libomp compatibility flags build the library.

A manually linked NIST smoke binary called `libgretl_init`/`libgretl_cleanup` and
reported, across Norris, Pontius, NoInt1/2, Filip, Longley, and Wampler1-5:

```text
number of tests carried out: 11
reference data files missing or corrupted: 0
unexpected errors in estimation of models: 0
poor or unacceptable results with libgretl: 0
```

The generated macOS test link also exposed an upstream portability issue: the
release configure output adds `-lrt` to the libtool link line even on Darwin,
where no `librt` exists. The manual smoke link omitted that flag and linked the
static archive directly. The full optional CLI build additionally encountered
Apple system-readline API drift and a plugin OpenMP-header path. Both macOS and
Linux NIST runs reported that the optional `mp_ols` plugin was unavailable; the
standard libgretl result still passed all 11 reference cases. Plugins and optional
CLI behavior remain outside this slice. These are native build findings, not Rust
safety evidence.

## Ownership, safety, and licensing ledger

| Boundary | Observed evidence | Decision/risk |
| --- | --- | --- |
| Initialization | `libgretl_init`/`libgretl_cleanup` are process-level functions; headers expose global/session helpers. | A future adapter must serialize or otherwise define initialization/cleanup and never let panic/cancellation strand native global state. |
| Native objects | `MODEL`, `DATASET`, matrices, strings, and plugin-facing pointers are C-owned and mutable. | No raw handle, borrowed string, or foreign struct may enter domain/application APIs; copy results into Rust-owned typed values. |
| Numerical substrate | Build links BLAS/LAPACK, FFTW3, GMP/MPFR, GLib, libxml2, zlib, curl, and OpenMP/libomp. | ABI, allocator, thread-count, plugin-loading, and packaging costs require a separate adapter decision and benchmark. |
| License | The pinned source `COPYING` identifies GNU GPL v3. | Confirm GPLv3/AGPLv3 combination, dynamic/static-link obligations, notices, and redistribution boundaries before any runtime adoption. |

## Decision and remaining risks

Continue libgretl as an isolated feasibility candidate. Do not create a
`tabdat-gretl-sys` crate or safe facade until the now-green cross-platform
build/link evidence is followed by an independent FFI ownership review and a
Rust-owned estimator/result contract.
The candidate is not product statistical support, an accepted primary backend, or
Python/trusted-reference parity.

Remaining risks include the large native dependency surface, process-global
initialization, OpenMP and BLAS thread interactions, GPLv3 redistribution terms,
plugin loading, Darwin `-lrt`/readline portability, the age and configuration
assumptions of the release build system, and the absence of a Rust fixture-backed
numerical comparison in this slice.

## FFI adapter and OLS fixture (2026-09-10)

Status: **partial**. A Rust-owned OLS boundary over libgretl is implemented and
numerically validated against the NIST certified reference and the pinned Python
TabDat oracle. This is still a feasibility spike, not product statistical
support or an accepted-backend decision.

### What was built

`spikes/gretl-prototype/` contains a two-crate spike mirroring the ReadStat
spike's shape:

- `gretl-sys/` — the only crate with `unsafe`. It declares the `extern "C"`
  surface of a C ownership shim (`shim.c`) and wraps the opaque native handles
  in RAII types (`GretlDataset`, `GretlModel`). No raw pointer or foreign struct
  is part of its public API.
- `tabdat-gretl-spike` (`src/lib.rs`) — `#![forbid(unsafe_code)]`. Defines
  Rust-owned `EstimationProblem` / `EstimationResult` / `Coefficient` types and
  orchestrates an OLS fit by driving the safe `gretl-sys` wrappers.

### Why a C ownership shim

libgretl's estimation entry point `lsq` returns a large `MODEL` struct **by
value** whose fields point at C-allocated memory, and `DATASET`/`MODEL` are
mutable C structures. Re-declaring those layouts in Rust would couple the
binding to the exact struct layout of a pinned C release. Instead `shim.c`
(compiled from the same pinned source tree, so its view of the structs is always
correct) owns all native objects and exposes only opaque pointers plus
scalar/array accessors. All `unsafe` and all libgretl layout knowledge stays in
`gretl-sys` + `shim.c`; the facade is compiler-enforced `forbid(unsafe_code)`.

### Build recipe (macOS Apple Silicon)

The pinned gretl `2026b` static library builds with:

```sh
./configure --disable-gui --disable-json --disable-nls --disable-build-addons \
  --disable-xdg-utils --disable-sse2 --disable-avx \
  --disable-shared --enable-static \
  OMP_LIB="-L/opt/homebrew/opt/libomp/lib -lomp"
make buildstamp
make -C lib -j8 CFLAGS="-g -O2 -I/opt/homebrew/include -I/opt/homebrew/opt/libomp/include -Xclang -fopenmp"
```

Two platform findings required explicit handling:

1. **OpenMP.** The release configure reports OpenMP disabled, but `gretl_matrix.c`
   calls `omp_set_num_threads` unguarded, so a plain disabled-OpenMP compile
   fails. Compiling with `-Xclang -fopenmp` (Apple Clang's OpenMP flag) defines
   `_OPENMP` so `omp.h` is included and the calls resolve; linking uses
   Homebrew `libomp`. `--disable-shared` is required because the shared-library
   (dynamiclib) link passes a bare `-fopenmp` that clang rejects at link time;
   only the static archive is needed.
2. **`monte_carlo.c` latent bug.** gretl 2026b's `monte_carlo.c` references
   `prog_cmd_started` (only a `#define` in `prog_loop.c`) as if it were a
   function. This is masked in the shared-library build (undefined-symbol
   suppression + the NIST test never calls that path) but breaks a clean
   static-library link. `build.rs` applies a minimal, documented patch that adds
   the missing `#define` after the existing `loop_line_*` macros in
   `monte_carlo.c`. This is a required compatibility fix for the pinned release,
   not a behavior change to the OLS path.

`build.rs` performs the full from-source build (download + SHA-256 verify +
patch + configure + make) for CI, and supports a `GRETL_PREBUILT_DIR` fast path
for local iteration.

### OLS numerical validation (NIST Longley)

`tests/ols_fixture.rs` runs OLS on the NIST Longley dataset (16 obs, 6
predictors + intercept) through the Rust facade and compares against:

1. **NIST/ITL certified values** (trusted external reference, from
   `gretl 2026b tests/Longley.dat`), and
2. **the pinned Python TabDat oracle** (`tabdat --json -f longley.td` at
   `tabdat-explore @ 16b45d9`, v0.25.0).

All 7 coefficients, 7 standard errors, R², adjusted R², residual standard
deviation, and F-statistic agree with both references to ~1e-11..1e-13 relative.
The test tolerance is a relative 1e-9 (three orders of magnitude looser than the
observed agreement) so it cannot mask a real regression. Representative values:

| Quantity | libgretl (Rust) | NIST certified | Python oracle |
| --- | --- | --- | --- |
| intercept | -3482258.63459777 | -3482258.63459582 | -3482258.6345977746 |
| x1 | 15.0618722715431 | 15.0618722713733 | 15.06187227159171 |
| x6 | 1829.15146461456 | 1829.15146461355 | 1829.1514646145479 |
| R² | 0.995479004577305 | 0.995479004577296 | 0.9954790045772965 |
| residual sd | 304.854073561657 | 304.854073561965 | 304.85407356193167 |
| F | 330.285339235259 | 330.285339234588 | — |

### Ownership and safety ledger

| Boundary | Evidence | Decision |
| --- | --- | --- |
| Initialization | `libgretl_init`/`cleanup` are process-level; the facade calls init once via `std::sync::Once`. | Init is idempotent and lazy; no per-call init cost. |
| Native objects | `GretlDataset`/`GretlModel` RAII wrappers free native memory in `Drop`; the C shim is the only code touching `DATASET`/`MODEL`. | No raw handle or foreign struct escapes `gretl-sys`. |
| Thread safety | libgretl does not document thread-safety; `GretlDataset`/`GretlModel` are not `Send`/`Sync`. | Handles are thread-confined (roadmap invariant 1.1). |
| Panic across FFI | The facade is `forbid(unsafe_code)`; the `gretl-sys` wrappers are thin and do not hold native state across a `catch_unwind` boundary in this slice. | No panic-FFI path exercised yet; revisit when adding long-running estimation. |

### Remaining risks and deferred work

- Only OLS is implemented; the roadmap's robust/clustered OLS, logit, probit,
  quantile, Tobit, Poisson, NB, IV/2SLS, panel, and other fixtures remain.
- The `monte_carlo.c` patch is pinned to gretl 2026b; a newer release may not
  need it (or may need a different fix).
- Linux x86_64 from-source build is implemented in `build.rs` but not yet
  exercised in hosted CI for this slice.
- GPLv3 redistribution terms are not yet resolved for runtime adoption.
- No benchmark of first-use initialization or repeated-model overhead yet.

## Boundary hardening (2026-10-11)

[ADR 0010](../adr/0010-gretl-spike-native-lifecycle.md) supersedes the original
facade-only initialization and thread-safety ledger for the current spike.
Every shim call now takes one adapter-global mutex, initializes once lazily,
and retains process state until exit; public cleanup is removed. Dataset and
model allocations still use serialized RAII teardown. Rust checks positive,
C-representable dimensions, all dataset/list indices and count fields, and
successful-model status and coefficient bounds before native buffer access.
Variable zero stays reserved even for no-intercept fits. All unsafe operations
remain in `gretl-sys`, with adjacent safety explanations and
`deny(unsafe_op_in_unsafe_fn)`.

### Checks and review

Local macOS 27.0.1 arm64, pinned toolchain:

- Root format/check/test/Clippy: pass (locked workspace, all targets).
- Spike format/check/test/Clippy: pass; eight tests, including Longley and
  five boundary regressions. Parallel fixture performs 4 × 20 fits/teardowns.
- Root `cargo audit -D warnings` and `cargo deny check`: pass (duplicate-version
  warnings only); spike lockfile audit: pass. Native dependencies are outside
  Cargo's advisory inventory.
- Adapter geiger JSON scan: completed; inventory only, not safety certification.
- Three separate owner review passes: (1) bounds/C ABI, (2) global lifecycle,
  ownership and teardown, (3) fixture integrity, API callers and scope. No
  unresolved actionable issue in this bounded change. No independent reviewer
  runtime was available; these are explicit author passes, not independent reviews.
- `git diff --check` and changed-document relative links: pass.

The no-intercept characterization passed even before hardening; reserving slot
zero is a native-layout invariant correction, not a demonstrated numerical
failure on that fixture. Potentially invalid native-memory accesses were not
executed on the unsafe pre-change wrapper. Regression cases now verify that
rejection happens in Rust. Native startup emits `otool exited with status 256`
on this machine; tests still pass, but the diagnostic remains a packaging risk.

No sanitizer/leak checker or new Linux execution ran. This review covers the
libgretl boundary only, not ReadStat/DuckDB or the whole Phase 1 unsafe exit gate.
Dropped-regressor naming, broader missingness/sample contracts, estimator
normalization, process-global cleanup cost, and full native packaging remain
separate validation/decision work. No additional estimator roadmap task is
checked by this safety prerequisite.

## HC1 and clustered OLS (2026-10-11)

Status: **pass for finite complete full-rank OLS fixtures**, not broad statistical
parity or primary-backend acceptance. `run_ols_hc1` and `run_ols_clustered` add
explicit covariance requests; results copy all matrix entries and coefficient
SEs into Rust-owned storage, include covariance mode, exact zero-based sample
rows and explicit inference df. No additional Cargo dependency is introduced.

### Contract and references

Synthetic 40-row `fixtures/covariance.csv`, predictors x1/x2 plus intercept,
10 groups of 4 observations in input order. SHA-256:
`d3be3915f7269cea0a5642d4b40e40de168187b66e4779877b47ec81b8f59822`.
Reference CSV SHA-256:
`1bda2699f939a7cffb227e97373989e4c105cd2872a9fd93d385a367137df246`.
The generator uses deterministic arithmetic, not a random seed. All rows are
finite and retained exactly; coefficient order is intercept, x1, x2.

- Pinned clean Python TabDat `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`,
  tree `601b236788872323af9277d2276a236154a0f129`, lock SHA-256
  `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`;
  Python 3.13.3 existing environment. `covariance_python.py` invokes the actual
  executor, retrieves fitted model covariance, and records public coefficients.
- Independent trusted reference: R 4.3.2 `lm` and sandwich 3.1-1
  `vcovHC(type="HC1")` / `vcovCL(type="HC1", cadjust=TRUE)`.
  Installed externally into `/tmp/tabdat-r-reference-lib`, not runtime code.
  `covariance_reference.R` verifies ordered sample rows 1..40 before extraction.
- Formula: HC1 n/(n-k); one-way cluster G/(G-1)*(n-1)/(n-k).
- Predeclared comparator: finite values and absolute difference no greater than
  `1e-10 + 1e-9*abs(reference)`, suitable for this well-conditioned fixture.
  Names, sample rows/counts, dimensions, mode and inference df compare exactly.
  No tolerance change followed a failed comparison.

All three coefficients, three SEs and nine covariance entries agree with both
references for classical/HC1/clustered modes (six reference rows in the committed
CSV). The existing Longley certified/oracle test also remains green.

### Native semantics and ownership

HC1 version is set/restored inside one serialized C call. The cluster option is
configured by the shim using an internal unique native column name and cleared
after fitting, including configuration/estimation failure paths. Original Rust
names remain in owned results, so C truncation cannot change cluster lookup.
Classical `lsq` leaves packed VCV storage lazy; the first failing fixture exposed
NaN matrix access. The shim now calls `makevcv` before copying, without changing
references or relaxing comparisons. Failed models never expose buffers.

Gretl reports `MODEL.dfd = 9` for the clustered fixture, while Python's fitted
`df_resid` remains 37 and its `df_resid_inference` is 9. Both use G-1 for cluster
inference; do not reinterpret Gretl's dfd as n-k across all covariance modes.
The result's explicit `inference_df` is 9 (37 for classical/HC1). No p-values,
intervals, clustered F-test or fit-statistic normalization is claimed here.

This spike now rejects non-finite/incomplete data, rank-dropped models and
nonpositive residual sample df rather than silently reporting a changed sample
or mislabeling dropped coefficients. This is a bounded input limitation, not
Python missing-row/collinearity parity. Invalid group length and fewer than two
groups return typed errors; labels are exact i32 values converted losslessly.
No domain/native handle leakage or new Rust unsafe block is introduced beyond
the reviewed adapter. New covariance/cluster arguments are checked before C.

### Verification and review

Eleven spike tests pass: existing 8 plus full matrix/reference comparison,
invalid-group/nonfinite/rank-failure recovery and 30 mixed-covariance fits on
three independent threads. The test-first attempt failed to compile on missing
APIs; subsequent matrix failure was diagnosed as lazy native VCV, then fixed.

Root and spike locked format/check/test/Clippy pass; root audit/deny pass with
duplicate-version warnings only, spike lock audit passes. Geiger adapter scan
is an unsafe inventory, not native certification. Changed-document links and
whitespace pass. Three explicit owner passes cover (1) source settings/correction
and VCV materialization, (2) checked inputs, RAII/serialization/error recovery,
(3) independent fixture/sample/DF provenance; a follow-up after VCV correction
confirms finite whole-matrix comparison. No unresolved blocker in this bounded
slice; no independent reviewer runtime was available. No new Linux, sanitizer,
or native-transitive advisory/license review is claimed.

Reproduce references separately from Rust tests (no Python/R runtime need):

```sh
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=/path/to/oracle/src /path/to/oracle/.venv/bin/python spikes/gretl-prototype/fixtures/covariance_python.py
R_LIBS=/tmp/tabdat-r-reference-lib Rscript spikes/gretl-prototype/fixtures/covariance_reference.R spikes/gretl-prototype/fixtures/covariance.csv
CARGO_TARGET_DIR=target/gretl-review cargo test --manifest-path spikes/gretl-prototype/Cargo.toml --locked --workspace --all-targets
```

Core nonlinear/count/IV/panel/selection/survival fixtures, comprehensive semantic
mismatch inventory, performance and primary-backend/licensing decisions remain
open. Only the two demonstrated OLS covariance fixture tasks are checked by
that slice.

## Classical logit and probit (2026-10-11)

Status: **pass for the bounded complete-data MLE fixtures**. Safe `BinaryLink`,
`BinaryResult` and `run_binary` expose Rust-owned coefficients, SEs, full
covariance, sample rows and likelihood, not native model state. The C shim owns
the by-value results from `binary_logit` / `binary_probit`; both use the existing
serialized initialization/RAII/check-buffer boundary. The facade accepts finite
numeric predictors and 0/1 outcomes and rejects changed samples/parameter counts
or non-finite outputs. Binary low-level lists reuse the checked OLS list helper.

### Reference contract

`fixtures/binary.csv` contains 200 synthetic observations, intercept and two
predictors, NumPy RNG seed 4276; the saved CSV is the shared fixture, not a promise
of seeded generator parity across environments. Data SHA-256
`8111c31527b23575c392b79b89679022556ed07da492c493557025630f820c03`;
reference SHA-256
`1b7921b276064522a5abb5f43cbace7acdfdd03632c0d91db2a2b1e154aa4dcb`.
Python checkout/tree/lock and Python 3.13.3 provenance are the same verified pin
as the preceding covariance slice. `binary_python.py` runs actual `logit` and
`probit` executor commands and confirms both fitted models converged.

Independent reference is R 4.3.2 base `glm(..., family=binomial(link=...))`,
`epsilon=1e-12`, maximum 100 iterations, confirming convergence and exact sample
rows 1..200. Its script computes the **observed information** from analytic
binary-likelihood derivatives at the R fit and verifies it against independent
R `optimHess` finite differences (maximum matrix difference <1e-4).

This distinction matters: ordinary R `vcov(glm)` uses expected/Fisher information
and may differ for the noncanonical probit link. Gretl and Python use observed
Hessian covariance. The reference script explicitly documents that convention;
it does not silently compare Fisher and observed SEs or widen tolerances.
Logit is canonical, so the observed/Fisher distinction vanishes there.

Gretl's `discrete.c::binary_model` uses Newton-Raphson defaults: maximum 100
iterations, objective tolerance 1e-8, gradient tolerance 1e-7. The spike does not
change those settings. Native nonzero errors, including no-convergence/perfect
prediction, return `GretlError::EstimationFailed`; a successful finite result is
not a generalized optimizer/cancellation guarantee.

Predeclared comparator: finite values and absolute difference <=
`1e-7 + 1e-6*abs(reference)`. All three coefficients, three SEs, nine covariance
entries and log likelihood pass against both Python and R. Coefficient order
(intercept/x1/x2) and all zero-based retained rows compare exactly. No tolerance
was changed after judging results. Separation, invalid outcome, missing predictor
and ragged input tests pass; subsequent nominal fits still succeed.

### Checks / review / exclusions

Test-first unresolved-API compilation failure was observed, then implementation
and both binary tests passed. Full spike suite now has 13 tests. Root/spike
locked format/check/test/Clippy pass; root/spike Cargo advisory checks pass, root
deny passes with duplicate warnings; no dependency or lock change. One first
root-suite attempt failed in the unchanged runtime reshape test while reading
its temporary Parquet fixture (`ParquetRead` at `reshape_contract.rs:337`). The
focused test and full locked workspace rerun both pass; cause is not established
and no runtime source was changed to hide it. Relative links and whitespace pass. Three explicit owner passes covered native by-value
ownership/list checks, covariance/likelihood and separation behavior, and exact
reference/sample/optimizer provenance; no actionable blocker for this bounded
slice. No independent reviewer runtime, new Linux run or sanitizer is claimed.

Binary robust/clustered covariance, absent-intercept reference coverage,
missing-row construction, probabilities, marginal effects, general failed-state
semantics, arbitrary design/rank deficiency and product model persistence remain
unvalidated. This does not satisfy the whole core-estimator exit gate. Count
models use a dynamically loaded `count_data_estimate` plugin; the spike currently
builds only libgretl, so plugin packaging is a prerequisite for those fixtures.

## Initialization and model costs (2026-10-11)

Five fresh release processes on Apple M1 Max / macOS 27.0.1 arm64, repository
Rust 1.97.1 pin, gretl 2026b. Compilation excluded. Filesystem/page caches are
uncontrolled: these are orientation measurements, **not cold-start or release
regression gates**. `initialize_for_measurement` enters the same mutex/once
boundary as ordinary lazy calls, letting initialization be timed separately
without exposing cleanup or requiring callers to initialize manually.

`measure --metadata` returns before fixture parsing or native initialization;
its exact stdout/empty stderr contract is tested. Native libraries are still
linked into this isolated executable, so the baseline includes loader cost and
is not a lazy-distribution certification.

| Fresh process | Init µs | First OLS µs | 100 OLS calls µs | First logit µs | 20 logit calls µs | Wall s | Peak RSS bytes |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 16,994 | 187 | 471 | 286 | 1,324 | 0.14 | 9,027,584 |
| 2 | 15,497 | 37 | 475 | 168 | 1,339 | 0.02 | 9,011,200 |
| 3 | 15,552 | 37 | 485 | 172 | 1,314 | 0.02 | 9,011,200 |
| 4 | 15,742 | 37 | 476 | 176 | 1,326 | 0.02 | 8,978,432 |
| 5 | 15,571 | 35 | 479 | 173 | 1,318 | 0.02 | 9,011,200 |

Median explicit init ~15.6 ms. Median repeated owned adapter calls: OLS ~4.76 µs
and logit ~66.2 µs. These tiny fixtures are the existing 40-row OLS and 200-row
binary reference data; each call includes native dataset allocation/fill,
list setup, estimation, coefficient/covariance/sample copying and RAII teardown.
It is **not isolated libgretl model arithmetic** or measured TabDat overhead.
Logit runs after OLS and shared initialization, not as its own cold process.

Metadata process wall time rounds to 0.00 s in `/usr/bin/time -l`, RSS 7,110,656
bytes. Release executable size is 3,039,872 bytes. This excludes unbuilt native
plugins and linked dynamic dependency distribution. Init still emits the known
`otool exited with status 256` diagnostic; child discovery work is included in
these init measurements, not silently subtracted. A larger representative
workflow, platform distribution and teardown/leak assessment remain open.

Reproduce:

```sh
CARGO_TARGET_DIR=target/gretl-review cargo build --manifest-path spikes/gretl-prototype/Cargo.toml --release --locked
/usr/bin/time -l target/gretl-review/release/measure
/usr/bin/time -l target/gretl-review/release/measure --metadata
```

Fourteen spike tests, locked root/spike format/check/test/Clippy and Cargo
advisory/policy checks pass. Three owner review passes cover timing boundaries,
metadata control flow/lifecycle, and raw measurements/cache/acceptance claims;
no actionable blocker for this measurement slice. No independent reviewer
runtime was available. The full workspace geiger inventory initially timed out
inside Cargo clean/glob traversal of the large shared target tree; the unchanged
full scan in dedicated `target/geiger-policy` completed for all four workspace
packages, each reporting forbid-unsafe and zero first-party usage. The verified
prevention is recorded in CONTRIBUTING.md; no weaker scan replaced it.
Measurements do not resolve native plugin, licensing,
statistical breadth or the overall Phase 1 blockers; see the
[preliminary feasibility matrix](phase1-assessment.md).

## Classical Poisson / plugin boundary (2026-10-12)

The opt-in `count-plugin` slice now validates classical integer-count Poisson
against pinned Python and independent R, with owned full covariance, exact sample,
native-error recovery and serialized mixed calls. Default static tests stay at 14;
the feature suite has 17 passing tests plus a separately exercised ignored loader
probe. See the [detailed report](poisson-plugin.md) and
[ADR 0012](../adr/0012-count-plugin-shared-core-spike.md) for shared-core matching,
verified source, bounded private paths, gettext/host-detection prerequisites,
full/sibling-layout unsafe inventories and platform/licensing limitations.
Python's fractional outcomes, broader count options, other plugins and production
packaging remain unaccepted. The prior measurements above excluded this capability
and must not be relabeled as shared-core/plugin cost measurements.

## Additional estimator evidence (2026-10-12)

[NB2](negative-binomial.md) now validates raw dispersion and full observed joint
covariance under ADR 0013's explicit Python metadata/optimizer normalization.
[IV/2SLS](iv2sls.md) validates classical no-df covariance and original structural
R². [Balanced panel FE/RE](panel.md) validates within slopes and Swamy-Arora RE
with explicit intercept and exact canonical row keys; it does not accept default
Python no-constant RE or unbalanced/robust panel contracts. Fresh Linux emulated
source builds accompany these slices ([platform record](linux-revalidation.md)).
[Classical Tobit](tobit.md) adds independently selected interval-plugin and full
observed beta/logscale covariance evidence (ADR 0014); original Python fallback
SEs are preserved as a recorded deviation, not accepted parity. The same shared
core/lifetime applies, with release first-use/repeated measurements in that report.
[Quantile LP](quantile.md) adds .25/.5/.75 coefficients and canonical iid
Epanechnikov/Hall-Sheather covariance under ADR 0015. Original IRLS stopping and
native sparsity differences are explicit, not hidden parity; additional rq NCSA
notice and independently selected shared module are recorded.
[PH survival](survival.md) adds exponential/Weibull joint observed covariance,
AFT/raw-sigma and event-density normalization under ADR 0016, exact event/censor IDs
and redundant-dummy protections. Original approximate Python SEs remain explicit
deviation; independent duration module, missing-loader and release costs are recorded.
[Proper two-step selection](heckman.md) adds distinct total/selected samples,
corrected outcome and observed probit covariance blocks, sigma/rho and explicit
ordinary-OLS SE deviation under ADR 0017. Independently selected heckit.so shares
owned serialized lifetime. Shim restores temporary Mills series on early second-
stage failure; packed signed-size bounds prevent known upstream overflow. No
purported joint cross-equation covariance or product selection support.
[Collapsed one-step dynamic panel](dynamic-panel.md) adds just-identified difference
GMM, entity covariance, exact native mask/group/instrument metadata under ADR 0018.
Explicit regular x instruments required alongside GMM specification; original
pooled-IVGMM independent-row SE difference preserved. Owned MODEL-type/length sample
and metadata restoration probes, independent dpanel.so and release costs recorded.
No general weighting/system/two-step/command acceptance.
The estimator reports supersede the earlier remaining-family lists for their
bounded fixtures only; general inference/model state, other families, safety,
packaging and primary-backend acceptance remain open.
