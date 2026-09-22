# Bounded `poisson` and `nbreg` syntax slice summary

## Outcome

Accepted bounded language-layer `poisson` and `nbreg` syntax slice. PR
[#85](https://github.com/SaehwanPark/tabdat-explore-rs/pull/85) was merged to
`main` as
[`4244936`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/4244936).

The language layer now exposes typed `PoissonCommand` and `NbregCommand` structs
and AST variants for direct `poisson <y> <xvars> [, options]` and
`nbreg <y> <xvars> [, options]` count-model commands. The parser retains the
outcome variable, ordered predictor list, robust covariance flag, optional
single cluster variable, and intercept flag. It enforces exact Python-compatible
diagnostics for missing arguments, conditions, assignment syntax, unsupported
options, mutual exclusivity between `robust` and `cluster`, single cluster variable
validation, and punctuation guards (`poisson:`, `nbreg:`). Runtime execution
remains explicitly deferred via `RuntimeError::UnsupportedCommand`.

No optimization or maximum likelihood estimation, backend initialization,
dispersion parameter estimation, model results, post-estimation (`estat`),
or CLI/JSON/MCP reporting surfaces were added.

Contract is recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge
all passed. The parser-only `poisson` and `nbreg` boundaries are closed; count
model estimation and numerical validation remain unchecked in the roadmap.
