# Bounded `logit` and `probit` syntax slice summary

## Outcome

Accepted bounded language-layer `logit` and `probit` syntax slice. PR
[#81](https://github.com/SaehwanPark/tabdat-explore-rs/pull/81) was merged to
`main` as
[`a14d552`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/a14d552).

The language layer now exposes typed `LogitCommand` and `ProbitCommand` for direct
`logit <y> <xvars>` and `probit <y> <xvars>` commands, with options `robust`,
`cluster(<var>)`, and `noconstant`. The parser validates mutual exclusions
(`robust` vs `cluster`), single tuple variable requirement for `cluster`,
single-use rules, flag values, and unquoted/quoted names, reporting exact
Python-compatible diagnostics. Runtime execution remains an explicit
unsupported-command result.

No statistical estimation, iterative optimization (Newton-Raphson/BFGS), link
functions, FFI backend execution, model result publication, post-estimation
state, reporting, CLI/JSON/MCP surface, or broad estimator parity was added.

Contract is recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge
all passed. The parser-only `logit` and `probit` boundary dependency is closed;
statistical estimation and post-estimation remain unchecked in the roadmap.
