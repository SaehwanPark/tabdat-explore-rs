# Bounded direct `predict` post-estimation syntax slice summary

## Outcome

Accepted bounded language-layer direct `predict` post-estimation prediction syntax slice. PR
[#107](https://github.com/SaehwanPark/tabdat-explore-rs/pull/107) was merged to
`main` as
[`aba64a8`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/aba64a8a07f7c469fefb3d1b72e5dcce251df83e).

The language layer now exposes typed `PredictKind` enum, `PredictCommand` struct, and `Command::Predict` AST variant for:
- `predict <newvar> [, xb | residuals | pr | spatial_lag | posterior_predictive] [interval] [level(<float>)] [std] [saving(<path>)]`

The parser retains the target variable name, prediction kind (`Xb`, `Residuals`, `Pr`, `SpatialLag`, `PosteriorPredictive`, defaulting to `Xb`),
interval flag (`interval`, defaulting to `false`), credible interval level (`level`, defaulting to `"95.0"`, validated `0 < level < 100`),
standard deviation flag (`std`, defaulting to `false`), and optional draws destination path (`saving`).
It enforces exact Python-compatible diagnostics for missing arguments (< 1 or > 1 argument), conditions (`if ...`), assignment syntax (`predict=`),
delimiter guards (`predict==`, `predict:`), flag option values, mutual exclusivity among prediction kinds, single-use rules, unsupported
options, path requirements, and option interdependencies (`interval`/`level`/`std`/`saving` requiring `posterior_predictive`; `level` requiring `interval`; `saving` cannot combine with `std` or `interval`).
Owned `String` and enum types are used so that `Command` retains its `Eq` derive.
Runtime execution remains explicitly deferred via `RuntimeError::UnsupportedCommand { name: "predict" }`.

No model scoring, post-estimation state lookup, dataset mutation, DuckDB column generation, Parquet draws persistence,
or CLI/JSON/MCP reporting surfaces were added.

Contract and implementation notes are recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge all passed. The
parser-only direct `predict` post-estimation prediction boundary is closed; post-estimation execution,
scoring, and numerical validation remain unchecked in the roadmap.
