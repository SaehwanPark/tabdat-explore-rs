# ADR 0005: Pin libgretl for bounded estimator-backend feasibility work

- Status: Accepted for continued feasibility evaluation; production adoption deferred
- Scope: libgretl source/build/link reconnaissance only; no Rust FFI or estimator API

## Context

The roadmap names libgretl as a candidate compiled classical/econometric backend.
Its C API exposes process-level initialization, mutable `MODEL`/`DATASET` structures,
and matrix/estimation facilities, so native build evidence must precede any safe
facade design. The candidate also carries a broad dependency and GPLv3 licensing
surface. A current gretl release archive is more reproducible for this bounded probe
than a moving source checkout.

## Decision

Use the gretl `2026b` SourceForge archive, verify its SHA-256 before extraction,
and run a path-scoped Linux build/link/NIST probe. Record macOS Apple Silicon
library and NIST evidence separately. Keep libgretl outside the root Cargo
workspace until an explicit FFI ownership, initialization, licensing, and
Rust-owned estimator/result contract is accepted.

A future adapter must own initialization/cleanup boundaries, copy native results into
Rust-owned typed values, define thread/plugin behavior, and keep `MODEL`, `DATASET`,
matrices, C strings, and raw handles out of domain/application APIs. This slice does
not create that adapter or mark an estimator implemented.

## Alternatives

- Build from the gretl Git default branch: rejected for this slice because a moving
  source tree does not provide a stable release/build contract.
- Add a Rust FFI crate now: deferred until Linux evidence, GPLv3/AGPLv3 redistribution
  review, and a fixture-backed ownership/result contract are complete.
- Choose a different native statistics engine immediately: deferred because the
  roadmap candidate has not yet received a bounded build/link decision.

## Consequences and verification

The archive/checksum makes the native probe repeatable and records the GPLv3 notice
obligation. macOS required explicit Apple Clang/libomp flags because the release
configuration reports OpenMP disabled while source code still calls OpenMP APIs.
The generated Darwin link also carries `-lrt`; the local smoke link omitted that
nonexistent library and linked the static archive directly. These compatibility
findings are preserved rather than presented as product support.

The macOS NIST smoke program initialized and cleaned up libgretl and passed 11
reference datasets with zero unexpected errors and zero poor/unacceptable results.
Hosted Linux verification passed in [PR #8 workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/34201655560)
before marking the two roadmap build/link items complete. Neither result
establishes ABI stability, thread safety, plugin safety, GPLv3 redistribution
compatibility, or statistical parity. Supersede this ADR if libgretl is rejected or
a newer pinned candidate is selected.
