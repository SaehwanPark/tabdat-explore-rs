# Bounded classical linear hypothesis testing `test` syntax slice summary

## Outcome

Accepted bounded language-layer classical linear hypothesis testing `test` syntax slice. PR
[#123](https://github.com/SaehwanPark/tabdat-explore-rs/pull/123) was merged to
`main` as
[`c9d846e`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/c9d846eb5d282b0837181f9c864e62cf1a88f47d).

The language layer now exposes typed `TestCommand` struct and `Command::Test` AST variant for:
- `test <varlist>`
- `test <lhs> = <rhs>`
- `test <lhs> == <rhs>`
- `test (<constraint>) [(<constraint>) ...]`

The parser retains the list of linear constraints (`constraints: Vec<GenerateExpression>`), where single unparenthesized
and parenthesized equalities `lhs = rhs` and `lhs == rhs` are normalized to `lhs - rhs` subtraction expressions, and
bare variable identifiers `x` are represented as `GenerateExpression::Identifier("x")`.
It enforces exact Python-compatible diagnostics for empty command bodies (`test command expects a list of variables or constraints`),
unexpected tokens outside parentheses (`test command: unexpected tokens outside parentheses`), mismatched parentheses
(`test command: mismatched parentheses`), empty constraints in parentheses (`test command: empty constraint inside parentheses`),
multiple equals in constraints (`test command: multiple '=' in a constraint` or `test command: multiple '=' in a single constraint (use parentheses for multiple constraints)`),
missing left-hand or right-hand sides (`test command: missing left-hand side of constraint` or `test command: missing right-hand side of constraint`),
malformed constraints (`test command: malformed constraint`), expected variable names in varlist mode (`test command: expected variable name, got '<token>'`),
delimiter guards (`test:`, `test=`, `test==`), and comma handling (`comma must be followed by at least one option`
when trailing, or `unknown command: test` when followed by options).
Owned `String`, `GenerateExpression`, and primitive types are used so that `Command` retains its `Eq` derive.
Runtime execution remains explicitly deferred via `RuntimeError::UnsupportedCommand { name: "test" }`.

No numerical post-estimation parameter retrieval, restriction matrix $R$ and vector $r$ construction,
Wald/F/chi-squared test statistics, p-values, or post-estimation state manipulation were added.

Contract and implementation notes are recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge all passed. The
parser-only classical linear hypothesis testing `test` boundary is closed; numerical post-estimation
restriction evaluations, Wald test statistics, p-values, and statistical validation remain unchecked in the roadmap.
