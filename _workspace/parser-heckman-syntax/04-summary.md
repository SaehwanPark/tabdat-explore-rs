# Bounded `heckman` syntax slice summary

## Outcome

Accepted bounded language-layer `heckman` syntax slice. PR
[#93](https://github.com/SaehwanPark/tabdat-explore-rs/pull/93) was merged to
`main` as
[`b0dad85`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/b0dad85).

The language layer now exposes typed `HeckmanCommand` struct and AST variant for
direct `heckman <y> <xvars>, selectdep(<var>) select(<vars>) [options]` sample-selection
regression commands. The parser retains the outcome variable, ordered predictor list,
required selection dependent variable (`selectdep`), required selection predictor list
(`select`), robust covariance flag (`robust`), single cluster variable (`cluster`), and
intercept flag (`noconstant`). It enforces exact Python-compatible diagnostics
for missing arguments, conditions, assignment syntax, missing required `selectdep`
option, missing required `select` option, unsupported options, variable parsing,
option single-use rules, cluster variable count constraints, mutual exclusivity
of `robust` and `cluster`, flag option value constraints, and punctuation guards
(`heckman:`, `heckman=`, `heckman==`). Owned `String` and `Vec<String>` are used so
that `Command` retains its `Eq` derive. Runtime execution remains explicitly deferred
via `RuntimeError::UnsupportedCommand`.

No sample-selection likelihood optimization, two-step estimation, FFI backends, model results,
post-estimation (`estat`), or CLI/JSON/MCP reporting surfaces were added.

Contract is recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge
all passed. The parser-only `heckman` boundary is closed; sample-selection regression
estimation and numerical validation remain unchecked in the roadmap.
