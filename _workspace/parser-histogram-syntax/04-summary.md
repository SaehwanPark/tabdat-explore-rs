# Bounded visualization `histogram` syntax slice summary

## Outcome

Accepted bounded language-layer visualization `histogram` syntax slice. PR
[#125](https://github.com/SaehwanPark/tabdat-explore-rs/pull/125) was merged to
`main` as
[`8b768b6`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/8b768b65e905a5dc6a2bcae2ee105d15c8bf3978).

The language layer now exposes typed `HistogramCommand` struct and `Command::Histogram` AST variant for:
- `histogram <variable> [, bins=<int> saving(<path>) noopen]`

The parser retains the target column identifier (`variable: String`), optional bin count (`bins: Option<i64>`),
optional plot save path (`saving: Option<String>`), and open-in-viewer flag (`open_artifact: bool`, defaulting to `true`
unless `noopen` is provided).
It enforces exact Python-compatible diagnostics for missing or multiple variables (`histogram expects exactly one variable`),
if clauses and assignment syntax (`histogram does not accept if clauses or assignment syntax`), assignment missing target or expression
(`histogram assignment requires a target before =` and `histogram assignment requires an expression after =`),
attached colons and double equals (`unsupported token in command: :` and `unsupported token in command: ==`),
trailing commas without options (`comma must be followed by at least one option`), unsupported options (`histogram unsupported option: <sorted_opts>`),
flag option values (`histogram option noopen does not accept a value`), bins validation (`histogram option bins must be at least 1`,
`histogram option bins expects an integer value`, `histogram option bins may only be supplied once`), and saving validation
(`histogram option saving expects a path`, `histogram option saving may only be supplied once`).
Owned `String`, `Option<i64>`, and primitive types are used so that `Command` retains its `Eq` derive.
Runtime execution remains explicitly deferred via `RuntimeError::UnsupportedCommand { name: "histogram" }`.

No DuckDB binned frequency aggregation, plot SVG/PNG rendering, artifact filesystem emission, or browser/viewer interactions were added.

Contract and implementation notes are recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge all passed. The
parser-only visualization `histogram` boundary is closed; DuckDB binned aggregation, rendering engines, and
browser display remain unchecked in the roadmap.
