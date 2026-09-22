# Bounded `nl` syntax slice summary

## Outcome

Accepted bounded language-layer `nl` syntax slice. PR
[#95](https://github.com/SaehwanPark/tabdat-explore-rs/pull/95) was merged to
`main` as
[`b0077ff`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/b0077ff).

The language layer now exposes typed `NlCommand` struct and AST variant for
direct `nl <y> = <expr>, params(<params>) start(<values>) [options]` nonlinear
least squares regression commands. The parser retains the outcome variable,
parsed expression tree (`GenerateExpression`), required parameter names
(`params`), required initial values (`start`), robust covariance flag
(`robust`), and intercept flag (`noconstant`). It enforces exact
Python-compatible diagnostics for missing arguments, conditions, assignment
syntax, missing required `params` option, missing required `start` option,
unsupported options, parameter name uniqueness, parameter count matching
start values count, numeric start value validation, option single-use rules,
flag option value constraints, and punctuation guards (`nl:`, `nl=`, `nl==`).
Owned `String` and `Vec<String>` are used so that `Command` retains its `Eq`
derive. Runtime execution remains explicitly deferred via
`RuntimeError::UnsupportedCommand`.

No nonlinear least squares optimization, Gauss-Newton / Levenberg-Marquardt
solvers, FFI backends, model results, post-estimation (`estat`), or CLI/JSON/MCP
reporting surfaces were added.

Contract is recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge
all passed. The parser-only `nl` boundary is closed; nonlinear regression
estimation and numerical validation remain unchecked in the roadmap.
