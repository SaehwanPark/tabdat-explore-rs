# Bounded `regress` syntax slice summary

## Outcome

Accepted bounded language-layer `regress` syntax slice. PR
[#79](https://github.com/SaehwanPark/tabdat-explore-rs/pull/79) was merged to
`main` as
[`e6b81d3`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/e6b81d3).

The language layer now exposes typed `RegressEstimator` (`Ols`, `Wls`, `Gls`)
and `RegressCommand` for direct `regress <y> <xvars>` commands, with options
`robust`, `cluster(<var>)`, `noconstant`, `wls(<var>)`, and `gls(<var>)`.
The parser validates mutual exclusions (`robust` vs `cluster`, `wls` vs `gls`),
variable counts, single-use rules, flag values, and unquoted/quoted names,
reporting exact Python-compatible diagnostics. Runtime execution remains an explicit
unsupported-command result.

No statistical estimation, linear-algebra computation, FFI backend execution,
model result publication, post-estimation state, reporting, CLI/JSON/MCP surface,
or broad estimator parity was added.

Contract is recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge
all passed. The parser-only `regress` boundary dependency is closed; statistical
estimation and post-estimation remain unchecked in the roadmap.
