# Bounded control function regression `cfregress` syntax slice summary

## Outcome

Accepted bounded language-layer control function regression `cfregress` syntax slice. PR
[#119](https://github.com/SaehwanPark/tabdat-explore-rs/pull/119) was merged to
`main` as
[`b800fda`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/b800fda0610f438cb56d2e67a030ef5ea7aa4486).

The language layer now exposes typed `CfRegressCommand` struct and `Command::CfRegress` AST variant for:
- `cfregress <y> [exog_vars], endog(<var>) iv(<vars>) [robust cluster(<var>) noconstant]`

The parser retains the dependent variable (`outcome`), optional exogenous variables (`exogenous`),
endogenous regressor variable (`endogenous`), instrumental variables (`instruments`),
robust covariance flag (`robust`), cluster variable (`cluster_variable: Option<String>`),
and intercept inclusion flag (`include_intercept: bool`).
It enforces exact Python-compatible diagnostics for argument counts (< 1 argument),
conditions (`if ...`), assignment syntax (`cfregress=`), delimiter guards (`cfregress==`, `cfregress:`),
required option validation (`cfregress option endog expects one variable`, `cfregress option iv expects at least one variable`),
single-use rules (`endog`, `iv`, `cluster`), flag-without-value checks (`cfregress option robust does not accept a value`,
`cfregress option noconstant does not accept a value`, `cfregress option endog expects variables`,
`cfregress option iv expects variables`, `cfregress option cluster expects variables`), variable arity checks
(`cfregress option endog expects one variable`, `cfregress option cluster expects one variable`),
option conflict rules (`cfregress cannot combine robust and cluster`), variable relationship constraints
(`cfregress endog variable must not appear in exogenous variables`), and unsupported options (alphabetically sorted).
Owned `String`, `Vec<String>`, and primitive types are used so that `Command` retains its `Eq` derive.
Runtime execution remains explicitly deferred via `RuntimeError::UnsupportedCommand { name: "cfregress" }`.

No numerical estimation, first-stage residual calculation, 2SLS/control-function augmentation,
bootstrapped standard errors, or CLI/JSON/MCP reporting surfaces were added.

Contract and implementation notes are recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge all passed. The
parser-only control function regression `cfregress` boundary is closed; numerical estimation,
first-stage residual modeling, second-stage estimation, standard error corrections, and statistical validation remain unchecked in the roadmap.
