# ADR 0006: libgretl FFI adapter via C ownership shim and Rust-owned OLS contract

- Status: Accepted for the bounded OLS feasibility slice; production adoption deferred
- Scope: `spikes/gretl-prototype/` (gretl-sys + safe facade); no product command, no accepted-backend decision

## Context

ADR 0005 pinned gretl `2026b` and established cross-platform build/link evidence,
deferring the Rust FFI until an independent ownership review and a Rust-owned
estimator/result contract existed. The roadmap's next libgretl items are the
low-level binding crate, the safe wrapper, Rust-owned `EstimationProblem` /
`EstimationResult`, and the OLS fixture with numerical validation.

The binding design is constrained by libgretl's C API: the estimation entry
point `lsq` returns a large `MODEL` struct **by value** whose fields point at
C-allocated memory, and `DATASET`/`MODEL` are mutable C structures. Re-declaring
those layouts in Rust would couple the binding to the exact struct layout of a
pinned C release and risk silent ABI drift.

## Decision

Use a **C ownership shim** (`gretl-sys/shim.c`) compiled from the same pinned
gretl source tree as the low-level binding boundary. The shim owns all native
objects and exposes only opaque pointers plus scalar/array accessors. The Rust
`gretl-sys` crate declares the shim's `extern "C"` surface (the only `unsafe` in
the spike) and wraps the opaque handles in RAII types (`GretlDataset`,
`GretlModel`) so native memory is released on every path. The `tabdat-gretl-spike`
facade is `#![forbid(unsafe_code)]` and defines the Rust-owned
`EstimationProblem` / `EstimationResult` / `Coefficient` contract plus OLS
orchestration.

Explicit boundaries:

- No raw pointer, borrowed C string, or foreign struct appears in any public API
  outside `gretl-sys`; the facade is compiler-enforced `forbid(unsafe_code)`.
- `GretlDataset` / `GretlModel` are not `Send`/`Sync` (libgretl documents no
  thread-safety); handles are thread-confined.
- `libgretl_init` is process-level and called once via `std::sync::Once`.
- The OLS fixture validates against the NIST certified reference **and** the
  pinned Python oracle; a relative 1e-9 tolerance is used (well below the
  observed ~1e-11..1e-13 agreement) and is not widened to hide disagreement.

A minimal, documented `monte_carlo.c` compatibility patch (adding the missing
`prog_cmd_started` macro) is applied in `build.rs` because gretl 2026b's
`monte_carlo.c` references that macro as a function, which breaks a clean
static-library link. This is a build fix for the pinned release, not an OLS
behavior change.

## Alternatives

- **Re-declare `DATASET`/`MODEL` in Rust** (bindgen-style): rejected because it
  couples the binding to the exact C struct layout of a pinned release and makes
  the by-value `MODEL` return + internal C allocations hard to own safely.
- **Link the shared library with undefined-symbol suppression** (as the
  feasibility NIST build did): rejected for the static-library path because it
  masks the `monte_carlo.c` latent bug and is a macOS-specific linker workaround.
- **Provide a stub `prog_cmd_started` symbol**: rejected because it requires
  replicating the `LOOPSET` layout/enum in the shim and would silently
  misbehave if the Monte Carlo path were ever exercised; the source patch fixes
  the root cause.
- **Implement OLS natively in Rust** (no libgretl): deferred; the roadmap
  evaluates libgretl as the compiled classical/econometric backend before
  committing to a native reimplementation.

## Consequences and verification

- The C shim keeps all libgretl layout knowledge in one compiled-from-source
  file, so it cannot drift from the pinned release.
- The RAII wrappers guarantee native memory is freed on success, error, and
  drop; the OLS fixture exercises the success and error paths.
- `tests/ols_fixture.rs` passes on macOS Apple Silicon against both the NIST
  certified values and the Python oracle (see feasibility report for the value
  table). The from-source `build.rs` path (download + verify + patch + configure
  + make) is exercised locally; Linux x86_64 CI for this slice is added with the
  spike workflow.
- This ADR does **not** mark libgretl an accepted production backend, does not
  implement any estimator beyond OLS, and does not resolve GPLv3 redistribution
  terms. Supersede if libgretl is rejected, a newer pinned candidate is selected,
  or the adapter boundary changes (e.g. a different shim/ownership model).
