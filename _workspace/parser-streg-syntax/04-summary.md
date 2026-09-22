# Bounded `streg` syntax slice summary

## Outcome

Accepted bounded language-layer `streg` syntax slice. PR
[#97](https://github.com/SaehwanPark/tabdat-explore-rs/pull/97) was merged to
`main` as
[`735e6a1`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/735e6a1).

The language layer now exposes typed `StregDistribution` enum (`Weibull`, `Exponential`)
and `StregCommand` struct and AST variant for direct
`streg <time_var> <xvars>, failure(<event>) dist(<weibull|exponential>) [options]`
parametric survival regression commands. The parser retains the survival time
variable, ordered predictor list, required failure indicator variable
(`failure`), required baseline distribution (`dist`), robust covariance flag
(`robust`), single cluster variable (`cluster`), and intercept flag (`noconstant`).
It enforces exact Python-compatible diagnostics for missing arguments, conditions,
assignment syntax, missing required `failure` option, missing required `dist` option,
unsupported options, single-use rules, cluster variable count constraints, mutual
exclusivity of `robust` and `cluster`, flag option values, and punctuation guards
(`streg:`, `streg=`, `streg==`). Owned `String` and `Vec<String>` are used so that
`Command` retains its `Eq` derive. Runtime execution remains explicitly deferred
via `RuntimeError::UnsupportedCommand`.

No parametric survival distribution estimation, numerical validation, FFI backends,
model results, post-estimation (`predict`, `estat`), or CLI/JSON/MCP reporting
surfaces were added.

Contract is recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge
all passed. The parser-only `streg` boundary is closed; survival regression
estimation and numerical validation remain unchecked in the roadmap.
