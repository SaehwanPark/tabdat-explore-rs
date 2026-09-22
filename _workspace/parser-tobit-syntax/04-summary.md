# Bounded `tobit` syntax slice summary

## Outcome

Accepted bounded language-layer `tobit` syntax slice. PR
[#91](https://github.com/SaehwanPark/tabdat-explore-rs/pull/91) was merged to
`main` as
[`708e64f`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/708e64f).

The language layer now exposes typed `TobitCommand` struct and AST variant for
direct `tobit <y> <xvars>, ll(<num>) [ul(<num>)] [options]` censored regression
commands. The parser retains the outcome variable, ordered predictor list,
required lower censoring limit (`ll`), optional upper censoring limit (`ul`),
robust covariance flag (`robust`), single cluster variable (`cluster`), and
intercept flag (`noconstant`). It enforces exact Python-compatible diagnostics
for missing arguments, conditions, assignment syntax, missing required `ll`
option, unsupported options, numeric parsing, option single-use rules, cluster
variable count constraints, mutual exclusivity of `robust` and `cluster`, flag
option value constraints, and punctuation guards (`tobit:`, `tobit=`, `tobit==`).
Numeric text for `lower_limit` and `upper_limit` remains an owned `String` so that
`Command` retains its `Eq` derive. Runtime execution remains explicitly deferred
via `RuntimeError::UnsupportedCommand`.

No likelihood optimization, numerical estimation, FFI backends, model results,
post-estimation (`estat`), or CLI/JSON/MCP reporting surfaces were added.

Contract is recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge
all passed. The parser-only `tobit` boundary is closed; censored regression estimation
and numerical validation remain unchecked in the roadmap.
