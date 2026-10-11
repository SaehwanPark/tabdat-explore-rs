# ADR 0002: Dependency and unsafe-code policy checks

- Status: Superseded for hosted execution by ADR 0009; policy tools remain available locally
- Scope: Rust workspace dependency and unsafe-code review; no runtime dependency adoption

## Context

The scaffold currently has no runtime dependencies, but future DuckDB, FFI, and
specialized capabilities will add supply-chain, licensing, advisory, and unsafe-code
review obligations. A passing compiler/linter does not inspect dependency advisories,
license expressions, source registries, or transitive unsafe usage. Tool versions that
float with each CI run would also make policy results harder to reproduce.

## Current status

GitHub Actions are disabled at the repository level and all tracked workflow
definitions are removed under [ADR 0009](0009-disable-github-actions.md). The
tool versions and policy configuration in this record remain the local review
procedure documented in `CONTRIBUTING.md`; no hosted policy job currently runs.

## Original decision

The original hosted policy ran three explicitly versioned tools in a separate
Linux CI job:

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

The former CI job was slower because tools compiled on a clean runner; explicit
versions and locked installations made that cost visible. The no-dependency graph
passed locally with all three tools. The isolated DuckDB spike also required
`CDLA-Permissive-2.0` for `webpki-roots`; that license is explicitly listed for
feasibility evaluation and is not a blanket native-redistribution approval. Geiger
reports the scaffold's `#![forbid(unsafe_code)]`
coverage. [PR #4 CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/34187594609)
passed both the Rust baseline and policy jobs. Future dependencies can fail local
policy checks, or hosted checks if reintroduced, until license/source/advisory and
unsafe implications are reviewed. Revisit versions in a dedicated tooling update,
not as part of feature implementation.
