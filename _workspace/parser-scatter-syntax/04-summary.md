# Bounded visualization `scatter` syntax slice summary

## Outcome

Accepted bounded language-layer visualization `scatter` syntax slice. PR
[#127](https://github.com/SaehwanPark/tabdat-explore-rs/pull/127) was merged to
`main` as
[`587fc07`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/587fc0799f2a7db1efc609594119d6718a7b6d19).

The language layer now exposes typed `ScatterCommand` struct and `Command::Scatter` AST variant for:
- `scatter <y_var> <x_var> [, saving(<path>) noopen]`

The parser retains the target y-axis column identifier (`y_variable: String`), target x-axis column identifier (`x_variable: String`),
optional plot save path (`saving: Option<String>`), and open-in-viewer flag (`open_artifact: bool`, defaulting to `true`
unless `noopen` is provided).
It enforces exact Python-compatible diagnostics for missing or extraneous variables (`scatter expects syntax: scatter y_var x_var`),
if clauses and assignment syntax (`scatter does not accept if clauses or assignment syntax`), assignment missing target or expression
(`scatter assignment requires a target before =` and `scatter assignment requires an expression after =`),
attached colons and double equals (`unsupported token in command: :` and `unsupported token in command: ==`),
trailing commas without options (`comma must be followed by at least one option`), unsupported options (`scatter unsupported option: <sorted_opts>`),
flag option values (`scatter option noopen does not accept a value`), and saving validation
(`scatter option saving expects a path`, `scatter option saving may only be supplied once`).
Owned `String` and primitive types are used so that `Command` retains its `Eq` derive.
Runtime execution remains explicitly deferred via `RuntimeError::UnsupportedCommand { name: "scatter" }`.

No DuckDB data extraction, Vega-Lite spec generation, plot SVG/PNG rendering, artifact filesystem emission, or browser/viewer interactions were added.

Contract and implementation notes are recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge all passed. The
parser-only visualization `scatter` boundary is closed; DuckDB data extraction, rendering engines, and
browser display remain unchecked in the roadmap.
