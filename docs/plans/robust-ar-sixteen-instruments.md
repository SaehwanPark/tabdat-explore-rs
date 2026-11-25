# Robust scalar AR confidence sets through sixteen instruments

## Scope

Extend retained HC1/one-way CR1 scalar Anderson–Rubin confidence sets to q=16
excluded instruments. Preserve the one-endogenous-coefficient contract,
included-exogenous nuisance treatment, retained sample/groups, corrections,
result metadata, output shape and all q<=15 behavior. The q=16 boundary has
degree at most 32, with up to 32 simple real roots and 17 accepted components;
q>=17 and other endogenous/covariance dimensions remain unsupported.

## Verification plan

- Use two deterministic 384-row designs with one included control,
  K_aux=18, residual DF=366 and 24 clusters of sixteen observations.
- Add an independent Base R reference for HC1 and one-way CR1. Cross-check
  degree-32 determinant coefficients at 33 Chebyshev nodes, `polyroot` roots,
  set endpoints, metadata and direct Wald membership probes.
- Test q=16 HC1/CR1 results, instrument rescaling, retained-fit/runtime state,
  JSON/text output and q=17 rejection before later effects.
- Extend the polynomial boundary test to 32 roots and 17 components; update
  help, result contracts, roadmap, SPEC, coverage evidence and runnable examples
  with q>=17 unsupported.
- Run focused and affected package suites, scoped Clippy, formatting and diff
  checks; regenerate the q=16 reference fixtures byte-for-byte.

## Progress

- [x] Scope and verification plan recorded.
- [x] Open draft PR before substantive implementation (#502).
- [x] Implement q=16 support and independent reference fixtures.
- [x] Update runtime/CLI coverage and specifications/examples.
- [x] Verify reference regeneration and affected checks.

Verification note: all affected package and CLI suites pass. Strict Clippy
passes for affected packages and changed CLI targets. A workspace-wide
`-D warnings` attempt also surfaced one pre-existing warning in the unrelated
`tests/cli_json_count_exposure_contract.rs` target.
- [x] Final PR review and merge: PR #502, squash merge 5e832da48dfd83f3288aee20fdbea1f98ab08ea6.
