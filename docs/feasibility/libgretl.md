# libgretl feasibility reconnaissance

- Status: **partial**; the pinned native library builds on macOS Apple Silicon and
  Linux x86_64 and its NIST linear-regression smoke program passes on both. No Rust
  binding or estimator support is accepted.
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
