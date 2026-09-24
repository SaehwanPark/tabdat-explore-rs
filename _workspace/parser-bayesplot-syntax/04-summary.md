# Bounded visualization `bayesplot` syntax slice summary

## Outcome

Accepted bounded language-layer visualization `bayesplot` syntax slice. PR
[#131](https://github.com/SaehwanPark/tabdat-explore-rs/pull/131) was merged to
`main` as
[`bacf757`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/bacf75756ce6a5a6617e3bdf5aefb67abeb5596b).

The language layer now exposes typed `BayesPlotKind` enum, `BayesPlotCommand` struct, and `Command::BayesPlot` AST variant for:
- `bayesplot <trace|density|autocorrelation> [, saving(<path>) noopen]`

The parser validates the single target diagnostic plot kind (`kind: BayesPlotKind`, supporting `Trace`, `Density`, and `Autocorrelation`),
optional plot save path (`saving: Option<String>`),
and open-in-viewer flag (`open_artifact: bool`, defaulting to `true` unless `noopen` is provided).
It enforces exact Python-compatible diagnostics for missing or multiple arguments (`bayesplot expects syntax: bayesplot <trace|density|autocorrelation>`),
unrecognized or uppercase plot kinds (`bayesplot kind must be trace, density, or autocorrelation`),
if clauses and assignment syntax (`bayesplot does not accept if clauses or assignment syntax`), assignment missing target or expression
(`bayesplot assignment requires a target before =` and `bayesplot assignment requires an expression after =`),
attached colons and double equals (`unsupported token in command: :` and `unsupported token in command: ==`),
trailing commas without options (`comma must be followed by at least one option`), unsupported options (`bayesplot unsupported option: <sorted_opts>`),
flag option values (`bayesplot option noopen does not accept a value`), and saving validation
(`bayesplot option saving expects a path`, `bayesplot option saving may only be supplied once`).
Owned `String` and primitive/enum types are used so that `Command` retains its `Eq` derive.
Runtime execution remains explicitly deferred via `RuntimeError::UnsupportedCommand { name: "bayesplot" }`.

No posterior draw extraction, chain iteration processing, Vega-Lite spec generation, plot SVG/PNG rendering, artifact filesystem emission, or browser/viewer interactions were added.

Contract and implementation notes are recorded in:

- [01-contract.md](01-contract.md)

The pinned oracle, local locked/policy checks, PR-head workflows, and squash merge all passed. The
parser-only visualization `bayesplot` boundary is closed; DuckDB data extraction, MCMC posterior sampling, rendering engines, and
browser display remain unchecked in the roadmap.
