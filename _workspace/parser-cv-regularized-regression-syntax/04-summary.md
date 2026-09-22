# Bounded cross-validation regularized regression syntax slice summary (`cvlasso`, `cvridge`, `cvelasticnet`)

## Outcome

Accepted bounded language-layer cross-validation regularized regression syntax slice. PR
[#103](https://github.com/SaehwanPark/tabdat-explore-rs/pull/103) was merged to
`main` as
[`cc08faa`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/cc08faa).

The language layer now exposes typed `CvlassoCommand`, `CvridgeCommand`, `CvelasticnetL1Ratio`,
and `CvelasticnetCommand` structs and AST variants for direct:
- `cvlasso linear <y> <xvars> [, cv(<int>) noconstant]`
- `cvridge linear <y> <xvars> [, cv(<int>) noconstant]`
- `cvelasticnet linear <y> <xvars> [, cv(<int>) l1_ratio(<f64|f64...>) noconstant]`

The parser retains the dependent outcome variable, ordered predictor list, cross-validation folds
(`cv`, defaulting to `5`, validated integer >= 2), elastic net mixing parameter (`l1_ratio` for
cvelasticnet, defaulting to `(0.1, 0.5, 0.7, 0.9, 0.95, 0.99, 1.0)`, validated in `[0.0, 1.0]`),
and intercept inclusion flag (`noconstant`, defaulting `include_intercept` to `true`). It enforces
exact Python-compatible diagnostics for missing arguments (< 3 arguments), invalid model specifications
(requiring unquoted `linear`), conditions (`if ...`), assignment syntax, single-use rules, unsupported
options, flag option values, numeric ranges, and punctuation guards (`cvlasso:`, `cvlasso=`, `cvlasso==`).
Owned `String`, `Vec<String>`, and `i64` types are used so that `Command` retains its `Eq` derive.
Runtime execution remains explicitly deferred via `RuntimeError::UnsupportedCommand`.

No numerical optimization (coordinate descent, scikit-learn), K-fold splitting, grid search, FFI
backends, model results, post-estimation (`predict`, `estat`), report file generation, or CLI/JSON/MCP
reporting surfaces were added.

Contract is recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge all passed. The
parser-only cross-validation regularized regression boundary is closed; estimation and numerical
validation remain unchecked in the roadmap.
