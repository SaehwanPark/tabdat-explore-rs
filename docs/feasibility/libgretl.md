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
| Linux x86_64 | Pass ([PR #8 workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/34201353211)) | Pinned archive, static library, `make -C tests check`, and 11 NIST datasets; the workflow passed with zero unexpected errors and zero poor/unacceptable libgretl results |

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
