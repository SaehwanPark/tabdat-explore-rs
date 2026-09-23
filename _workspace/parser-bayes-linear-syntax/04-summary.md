# Bounded Bayesian linear regression syntax slice summary (`bayes linear`)

## Outcome

Accepted bounded language-layer direct Bayesian linear regression syntax slice. PR
[#105](https://github.com/SaehwanPark/tabdat-explore-rs/pull/105) was merged to
`main` as
[`79f0bc4`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/79f0bc49c65972a1f4e31c6174f7c6e250560b88).

The language layer now exposes a typed `BayesCommand` struct and `Command::Bayes` AST variant for direct:
- `bayes linear <y> <xvars> [, n_iter(<int>) tol(<float>) noconstant]`

The parser retains the dependent outcome variable, ordered predictor list, maximum iteration count
(`n_iter`, defaulting to `300`, validated integer >= 1), convergence tolerance (`tol`, defaulting to `"0.001"`,
validated positive finite float), and intercept inclusion flag (`noconstant`, defaulting `include_intercept` to `true`).
It enforces exact Python-compatible diagnostics for missing arguments (< 3 arguments), invalid model specifications
(requiring unquoted `linear`), conditions (`if ...`), assignment syntax (`bayes=`), delimiter guards (`bayes==`),
prefix disambiguation (routing `bayes:` / `bayes, options:` to `BayesPrefixCommand`), single-use rules, unsupported
options, flag option values, and numeric ranges.
Owned `String`, `Vec<String>`, and `i64` types are used so that `Command` retains its `Eq` derive.
Runtime execution remains explicitly deferred via `RuntimeError::UnsupportedCommand { name: "bayes" }`.

No numerical optimization (Bayesian evidence maximization, conjugate linear regression, scikit-learn BayesianRidge),
MCMC sampling, FFI backends, model results, post-estimation (`predict`, `estat`), report file generation, or
CLI/JSON/MCP reporting surfaces were added.

Contract and implementation notes are recorded in:

- [01-contract.md](01-contract.md)
- [02-implementation-notes.md](02-implementation-notes.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge all passed. The
parser-only direct Bayesian linear regression boundary is closed; estimation and numerical validation
remain unchecked in the roadmap.
