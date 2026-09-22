# Bounded `zip` and `zinb` syntax slice summary

## Outcome

Accepted bounded language-layer `zip` and `zinb` syntax slice. PR
[#87](https://github.com/SaehwanPark/tabdat-explore-rs/pull/87) was merged to
`main` as
[`78cf4d8`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/78cf4d8).

The language layer now exposes typed `ZipCommand` and `ZinbCommand` structs
and AST variants for direct `zip <y> <xvars>, inflate(<zvars>) [, options]` and
`zinb <y> <xvars>, inflate(<zvars>) [, options]` zero-inflated count-model
commands. The parser retains the outcome variable, ordered predictor list, ordered
inflate predictor list, robust covariance flag, optional single cluster variable,
and intercept flag. It enforces exact Python-compatible diagnostics for missing
arguments, conditions, assignment syntax, unsupported options, missing or malformed
`inflate(...)` specification, mutual exclusivity between `robust` and `cluster`,
single cluster variable validation, and punctuation guards (`zip:`, `zinb:`). Runtime
execution remains explicitly deferred via `RuntimeError::UnsupportedCommand`.

No optimization or maximum likelihood estimation, EM algorithm, backend
initialization, dispersion parameter estimation, model results, post-estimation
(`estat`), or CLI/JSON/MCP reporting surfaces were added.

Contract is recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge
all passed. The parser-only `zip` and `zinb` boundaries are closed; zero-inflated
count model estimation and numerical validation remain unchecked in the roadmap.
