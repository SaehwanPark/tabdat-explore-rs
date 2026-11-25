# Robust scalar AR confidence sets through eighteen instruments

## Scope

Extend retained HC1/one-way CR1 scalar Anderson–Rubin confidence sets to q=18
excluded instruments. Preserve the one-endogenous-coefficient contract,
included-exogenous nuisance treatment, retained sample/groups, corrections,
result metadata, output shape and all q<=17 behavior. The q=18 boundary has
degree at most 36, with up to 36 simple real roots and 19 accepted components;
q>=19 and other endogenous/covariance dimensions remain unsupported.

## Verification plan

- Use two deterministic complete 432-row designs with one included control,
  K_aux=20, residual DF=412 and 24 clusters of eighteen observations. Confirm
  exact sample membership in both reference and Rust.
- Add an independent Base R reference for HC1 and one-way CR1. Cross-check
  degree-36 determinant coefficients at 37 Chebyshev nodes, `polyroot` roots,
  set endpoints, metadata and direct Wald membership probes.
- Test q=18 HC1/CR1 results, instrument rescaling, retained-fit/runtime state,
  JSON/text output and q=19 rejection before later effects.
- Extend the polynomial boundary test to 36 roots and 19 components; update
  help, result contracts, roadmap, SPEC, coverage evidence and runnable examples
  with q>=19 unsupported.
- Run focused and affected package suites, scoped Clippy, formatting and diff
  checks; regenerate the q=18 reference fixtures byte-for-byte.

## Progress

- [x] Scope and verification plan recorded.
- [x] Open draft PR before substantive implementation.
- [x] Implement q=18 support and independent reference fixtures.
- [x] Update runtime/CLI coverage and specifications/examples.
- [x] Verify reference regeneration and affected checks.
- [x] Final PR review and merge (PR #504, merge commit `a1a7eda7`).

Verification note: the two generated data files and Base R reference CSV match
the checked-in fixtures byte-for-byte. The q=18 statistics reference test passes
all four design/covariance combinations; the 36-root/19-component boundary,
runtime retained-fit, q=19 rejection, and CLI/JSON/text workflows pass. The
statistics, execution, reporting and runtime package suites pass; 59 affected
root integration tests pass. Strict Clippy passes for the affected packages,
binary and CLI targets. `cargo fmt --all --check` and `git diff --check` pass.
One existing manual source probe remains intentionally ignored.
