# Bounded difference-in-differences `did` syntax slice summary

## Outcome

Accepted bounded language-layer difference-in-differences `did` syntax slice. PR
[#113](https://github.com/SaehwanPark/tabdat-explore-rs/pull/113) was merged to
`main` as
[`5c6a469`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/5c6a469ed17ecbb2718e811ce7ce56ef84497a13).

The language layer now exposes typed `DidCommand` struct and `Command::Did` AST variant for:
- `did <y> [controls], treat(<var>) post(<var>) [robust]`

The parser retains the dependent variable (`outcome`), optional control variables (`controls`),
treatment indicator variable (`treatment_variable`), post-treatment time period indicator variable (`post_variable`),
and robust covariance flag (`robust`).
It enforces exact Python-compatible diagnostics for argument counts (< 1 argument), conditions (`if ...`),
assignment syntax (`did=`), delimiter guards (`did==`, `did:`), required option validation (`did option treat expects one variable`,
`did option post expects one variable`), single-use rules (`treat`, `post`), flag-without-value checks (`did option robust does not accept a value`),
variable relationship constraints (`did treatment and post variables must be distinct`, `did treatment and post variables must differ from outcome`,
`did treatment and post variables must not appear in controls`), and unsupported options (alphabetically sorted).
Owned `String` and `Vec<String>` types are used so that `Command` retains its `Eq` derive.
Runtime execution remains explicitly deferred via `RuntimeError::UnsupportedCommand { name: "did" }`.

No numerical estimation, interaction matrix formulation, parallel trends diagnostics,
or CLI/JSON/MCP reporting surfaces were added.

Contract and implementation notes are recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge all passed. The
parser-only difference-in-differences `did` boundary is closed; numerical estimation,
two-way fixed effects, and statistical validation remain unchecked in the roadmap.
