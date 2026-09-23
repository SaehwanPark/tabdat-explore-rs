# Bounded linear combination hypothesis testing `lincom` syntax slice summary

## Outcome

Accepted bounded language-layer post-estimation linear combination hypothesis testing `lincom` syntax slice. PR
[#121](https://github.com/SaehwanPark/tabdat-explore-rs/pull/121) was merged to
`main` as
[`939bdd7`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/939bdd7c2ffc6e7f8bfb2a47290f1d533ba17f0f).

The language layer now exposes typed `LincomCommand` struct and `Command::Lincom` AST variant for:
- `lincom <expression>`

The parser retains the linear combination expression (`expression: GenerateExpression`).
It enforces exact Python-compatible diagnostics for empty command bodies (`lincom command expects a linear combination expression`),
incomplete expressions (e.g. `incomplete expression after +`), missing closing parentheses
(`missing closing ) in expression`), unsupported tokens in expression (`unsupported token in expression: <token>`),
delimiter guards (`lincom:`, `lincom=`, `lincom==`), and comma handling (`comma must be followed by at least one option`
when trailing, or `unknown command: lincom` when followed by options).
Owned `String`, `GenerateExpression`, and primitive types are used so that `Command` retains its `Eq` derive.
Runtime execution remains explicitly deferred via `RuntimeError::UnsupportedCommand { name: "lincom" }`.

No numerical post-estimation parameter retrieval, symbolic gradient/differentiation, covariance matrix
transformation, standard error computation, t/z test statistics, p-values, or confidence intervals were added.

Contract and implementation notes are recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge all passed. The
parser-only linear combination hypothesis testing `lincom` boundary is closed; numerical post-estimation
parameter calculations, inference, and statistical validation remain unchecked in the roadmap.
