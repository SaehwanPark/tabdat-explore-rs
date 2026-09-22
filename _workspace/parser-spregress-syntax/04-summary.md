# Bounded `spregress` syntax slice summary

## Outcome

Accepted bounded language-layer `spregress` syntax slice. PR
[#99](https://github.com/SaehwanPark/tabdat-explore-rs/pull/99) was merged to
`main` as
[`eaee45c`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/eaee45c).

The language layer now exposes typed `SpregressModelType` enum (`Lag`, `Error`, `Sarar`),
`SpregressContiguity` enum (`Queen`, `Rook`), and `SpregressCommand` struct and AST variant
for direct
`spregress <y> <xvars>, [coord(<lat_var> <lon_var>) [knn(<k>)] | weights(<path_to_file>) id(<id_var>) [contiguity(queen|rook)]] [model(<lag|error|sarar>) robust]`
spatial autoregressive / spatial error regression commands. The parser retains the dependent
outcome variable, ordered predictor list, spatial model specification (`model`, defaulting to `lag`),
coordinate variables (`coord`, requiring exactly two variables), nearest-neighbor count (`knn`,
defaulting to 5 when `coord` is used and validated to be at least 1), external spatial weights path (`weights`),
required ID variable (`id`), spatial contiguity criterion (`contiguity`, defaulting to `queen` when `weights` is used),
and robust covariance flag (`robust`). It enforces exact Python-compatible diagnostics for missing
arguments, conditions, assignment syntax, missing spatial specifications, mutual exclusivity of `coord` and `weights`,
option compatibility rules (`id`/`contiguity` only with `weights`; `knn` only with `coord`), single-use rules,
unsupported options, flag option values, and punctuation guards (`spregress:`, `spregress=`, `spregress==`). Owned
`String` and `Vec<String>` types are used so that `Command` retains its `Eq` derive. Runtime execution
remains explicitly deferred via `RuntimeError::UnsupportedCommand`.

No spatial weight matrix construction (k-NN / PySAL / Shapefile), 2SLS / GM / SARAR estimation, numerical
validation, FFI backends, model results, post-estimation (`predict`, `estat`), or CLI/JSON/MCP reporting
surfaces were added.

Contract is recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge
all passed. The parser-only `spregress` boundary is closed; spatial regression
estimation and numerical validation remain unchecked in the roadmap.
