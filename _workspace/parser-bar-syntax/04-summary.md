# Bounded visualization `bar` syntax slice summary

## Outcome

Accepted bounded language-layer visualization `bar` syntax slice. PR
[#129](https://github.com/SaehwanPark/tabdat-explore-rs/pull/129) was merged to
`main` as
[`8f5dcfb`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/8f5dcfb9dcbeec16bb6e987c47d69d4d84c316cb).

The language layer now exposes typed `BarCommand` struct and `Command::Bar` AST variant for:
- `bar <variable> [, missing saving(<path>) noopen]`

The parser retains the target categorical column identifier (`variable: String`),
optional plot save path (`saving: Option<String>`), flag to include missing values as a distinct category (`include_missing: bool`, defaulting to `false` unless `missing` is provided),
and open-in-viewer flag (`open_artifact: bool`, defaulting to `true` unless `noopen` is provided).
It enforces exact Python-compatible diagnostics for missing or multiple variables (`bar expects exactly one variable`),
if clauses and assignment syntax (`bar does not accept if clauses or assignment syntax`), assignment missing target or expression
(`bar assignment requires a target before =` and `bar assignment requires an expression after =`),
attached colons and double equals (`unsupported token in command: :` and `unsupported token in command: ==`),
trailing commas without options (`comma must be followed by at least one option`), unsupported options (`bar unsupported option: <sorted_opts>`),
flag option values (`bar option missing does not accept a value`, `bar option noopen does not accept a value`), and saving validation
(`bar option saving expects a path`, `bar option saving may only be supplied once`).
Owned `String` and primitive types are used so that `Command` retains its `Eq` derive.
Runtime execution remains explicitly deferred via `RuntimeError::UnsupportedCommand { name: "bar" }`.

No DuckDB category frequency aggregation, Vega-Lite spec generation, plot SVG/PNG rendering, artifact filesystem emission, or browser/viewer interactions were added.

Contract and implementation notes are recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge all passed. The
parser-only visualization `bar` boundary is closed; DuckDB data aggregation, rendering engines, and
browser display remain unchecked in the roadmap.
