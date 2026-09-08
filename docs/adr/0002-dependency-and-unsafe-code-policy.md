# ADR 0002: Dependency and unsafe-code policy checks

- Status: Accepted
- Scope: Rust workspace CI and dependency review; no runtime dependency adoption

## Context

The scaffold currently has no runtime dependencies, but future DuckDB, FFI, and
specialized capabilities will add supply-chain, licensing, advisory, and unsafe-code
review obligations. A passing compiler/linter does not inspect dependency advisories,
license expressions, source registries, or transitive unsafe usage. Tool versions that
float with each CI run would also make policy results harder to reproduce.

## Decision

Run three explicitly versioned tools in a separate Linux CI job:

- `cargo-deny` 0.20.2 with `deny.toml` for advisories, licenses, duplicate/wildcard
  dependencies, and registry/git sources;
- `cargo-audit` 0.22.2 with `-D warnings` for RustSec advisories and yanked crates;
- `cargo-geiger` 0.13.0 with all targets/dependencies to report unsafe usage.

Install each with `--locked`; the policy configuration has no advisory or license
exceptions. The workspace package is `publish = false` and is ignored by the
license-resolution check until a reviewed release-license decision; external
packages remain subject to the allowlist. Allowed licenses are an initial
permissive set, not blanket approval for native or copyleft dependencies. Every
exception needs a reason and a reviewed decision. Geiger is an inventory signal,
not proof of FFI safety or a substitute for unsafe-boundary review.

## Alternatives

- Run only `cargo audit`: misses license/source/duplicate policy and unsafe inventory.
- Install latest tools without versions: less reproducible and can change CI semantics.
- Deny all unsafe code through a scanner: incompatible with approved low-level FFI;
  keep `#![forbid(unsafe_code)]` and adjacent safety review as code-level gates.
- Add security tools as runtime dependencies: unnecessary and violates core distribution
  boundaries.

## Consequences and verification

CI is slower because tools compile on a clean runner; explicit versions and locked
installations make that cost visible. The current no-dependency graph passes locally
with all three tools, while geiger reports the scaffold's `#![forbid(unsafe_code)]`
coverage. Future dependencies can fail CI until their license/source/advisory and
unsafe implications are reviewed. Revisit versions in a dedicated tooling update,
not as part of feature implementation.
