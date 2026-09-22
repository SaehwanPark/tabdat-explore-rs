# Bounded `qreg` syntax slice summary

## Outcome

Accepted bounded language-layer `qreg` syntax slice. PR
[#89](https://github.com/SaehwanPark/tabdat-explore-rs/pull/89) was merged to
`main` as
[`cc6e656`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/cc6e656).

The language layer now exposes typed `QregCommand` struct and AST variant for
direct `qreg <y> <xvars> [, options]` quantile regression commands. The parser
retains the outcome variable, ordered predictor list, quantile numeric spelling
(default `"0.5"`), robust covariance flag, and intercept flag. It enforces exact
Python-compatible diagnostics for missing arguments, conditions, assignment syntax,
unsupported options, quantile numeric parsing and bounds (`0 < quantile < 1`),
option single-use rules, flag option value constraints, and punctuation guards
(`qreg:`). Numeric text for `quantile` remains an owned `String` so that `Command`
retains its `Eq` derive. Runtime execution remains explicitly deferred via
`RuntimeError::UnsupportedCommand`.

No linear programming solver, interior point methods, quantile loss minimization,
backend initialization, standard error estimation, model results, post-estimation
(`estat`), or CLI/JSON/MCP reporting surfaces were added.

Contract is recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge
all passed. The parser-only `qreg` boundary is closed; quantile regression estimation
and numerical validation remain unchecked in the roadmap.
