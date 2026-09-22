# Bounded regularized regression syntax slice summary (`lasso`, `postlasso`, `ridge`, `elasticnet`)

## Outcome

Accepted bounded language-layer regularized regression syntax slice. PR
[#101](https://github.com/SaehwanPark/tabdat-explore-rs/pull/101) was merged to
`main` as
[`477e815`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/477e815).

The language layer now exposes typed `LassoCommand`, `PostlassoCommand`, `RidgeCommand`,
and `ElasticnetCommand` structs and AST variants for direct:
- `lasso linear <y> <xvars> [, alpha(<f64>) noconstant]`
- `postlasso linear <y> <xvars> [, alpha(<f64>) robust noconstant]`
- `ridge linear <y> <xvars> [, alpha(<f64>) noconstant]`
- `elasticnet linear <y> <xvars> [, alpha(<f64>) l1_ratio(<f64>) noconstant]`

The parser retains the dependent outcome variable, ordered predictor list, regularization penalty
(`alpha`, defaulting to `"1.0"`, validated positive), elastic net mixing parameter (`l1_ratio`,
defaulting to `"0.5"`, validated in `[0.0, 1.0]`), robust covariance flag (`robust` on `postlasso`),
and intercept inclusion flag (`noconstant`, defaulting `include_intercept` to `true`). It enforces
exact Python-compatible diagnostics for missing arguments (< 3 arguments), invalid model specifications
(requiring unquoted `linear`), conditions (`if ...`), assignment syntax, single-use rules, unsupported
options, flag option values, numeric ranges, and punctuation guards (`lasso:`, `lasso=`, `lasso==`).
Owned `String` and `Vec<String>` types are used so that `Command` retains its `Eq` derive. Runtime
execution remains explicitly deferred via `RuntimeError::UnsupportedCommand`.

No numerical optimization (coordinate descent, proximal gradient), post-lasso refitting, cross-validation,
FFI backends, model results, post-estimation (`predict`, `estat`), or CLI/JSON/MCP reporting surfaces
were added.

Contract is recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge all passed. The
parser-only regularized regression boundary is closed; estimation and numerical validation remain
unchecked in the roadmap.
