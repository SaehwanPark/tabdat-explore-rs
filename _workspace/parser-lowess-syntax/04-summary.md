# Bounded locally weighted regression smoother `lowess` syntax slice summary

## Outcome

Accepted bounded language-layer locally weighted regression smoother `lowess` syntax slice. PR
[#111](https://github.com/SaehwanPark/tabdat-explore-rs/pull/111) was merged to
`main` as
[`d188b33`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/d188b338cb88f21950e8d048ba8c3e8a4a580663).

The language layer now exposes typed `LowessCommand` struct and `Command::Lowess` AST variant for:
- `lowess <y> <x>, gen(<newvar>) [bandwidth=<0,1>]`

The parser retains the dependent variable (`outcome`), single predictor variable (`predictor`),
target smoothed variable name (`target_variable`), and smoothing bandwidth (`bandwidth`, defaulting to `"0.6666666666666666"`).
It enforces exact Python-compatible diagnostics for argument counts (!= 2 arguments), conditions (`if ...`),
assignment syntax (`lowess=`), delimiter guards (`lowess==`, `lowess:`), required option validation (`lowess option gen expects one variable`),
single-use rules (`gen`, `bandwidth`), flag-without-value checks, out-of-range bandwidth ($val \le 0$ or $val \ge 1$),
and unsupported options (alphabetically sorted).
Owned `String` types are used so that `Command` retains its `Eq` derive.
Runtime execution remains explicitly deferred via `RuntimeError::UnsupportedCommand { name: "lowess" }`.

No non-parametric regression smoothing, tricube kernel weighting, local polynomial fitting,
or CLI/JSON/MCP reporting surfaces were added.

Contract and implementation notes are recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge all passed. The
parser-only locally weighted regression smoother `lowess` boundary is closed; numerical estimation,
smoothing kernels, and statistical validation remain unchecked in the roadmap.
