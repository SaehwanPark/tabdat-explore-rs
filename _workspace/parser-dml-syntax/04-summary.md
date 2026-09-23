# Bounded double machine learning `dml` syntax slice summary

## Outcome

Accepted bounded language-layer double machine learning `dml` syntax slice. PR
[#117](https://github.com/SaehwanPark/tabdat-explore-rs/pull/117) was merged to
`main` as
[`f2e537c`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/f2e537c44ea092a95c4794e7df6504a794cb1f0e).

The language layer now exposes typed `DmlCommand` struct and `Command::Dml` AST variant for:
- `dml linear <y> <controls>, treat(<var>) [folds(<int>) alpha(<num>) robust seed(<int>) noconstant]`

The parser retains the dependent variable (`outcome`), control variables (`controls`),
treatment indicator variable (`treatment_variable`), cross-fitting folds count (`folds`, defaulting to `5`),
regularization penalty parameter (`alpha: String`, retained as source spelling, defaulting to `"1.0"`),
robust covariance flag (`robust`), random seed (`seed: Option<i64>`), and intercept inclusion flag (`include_intercept: bool`).
It enforces exact Python-compatible diagnostics for argument counts (< 3 arguments), model validation (`dml model must be linear`),
conditions (`if ...`), assignment syntax (`dml=`), delimiter guards (`dml==`, `dml:`), required option validation (`dml option treat expects one variable`),
single-use rules (`treat`, `folds`, `alpha`, `seed`), flag-without-value checks (`dml option robust does not accept a value`,
`dml option noconstant does not accept a value`, `dml option folds expects an integer value`, `dml option alpha expects a numeric value`,
`dml option seed expects an integer value`), numeric range limits (`dml option folds must be at least 2`,
`dml option alpha must be positive`, `dml option seed must be at least 0`), variable relationship constraints
(`dml treatment variable must differ from outcome`, `dml treatment variable must not appear in controls`),
and unsupported options (alphabetically sorted).
Owned `String`, `Vec<String>`, and primitive types are used so that `Command` retains its `Eq` derive.
Runtime execution remains explicitly deferred via `RuntimeError::UnsupportedCommand { name: "dml" }`.

No numerical estimation, cross-fitting, regularized nuisance estimation (Lasso), Neyman-orthogonal scoring,
or CLI/JSON/MCP reporting surfaces were added.

Contract and implementation notes are recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge all passed. The
parser-only double machine learning `dml` boundary is closed; numerical estimation,
cross-fitting nuisance modeling, score evaluation, and statistical validation remain unchecked in the roadmap.
