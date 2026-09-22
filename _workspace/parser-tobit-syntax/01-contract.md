# Bounded syntax-only `tobit` command contract

Status: contract checkpoint; implementation and validation are in progress.

Producer: task owner, using `tabdat-migration`, `simple-code-writer`, and
`preferred-workflow`.
Consumer: the bounded Rust language/parser implementation and its focused review.

## Scope

Add the backend-independent parser boundary for the documented Tobit (censored)
regression command:
- `tobit <y> <xvars>, ll(<num>) [ul(<num>)] [robust | cluster(<var>)] [noconstant]`

The parser owns the dependent variable (outcome), the ordered regressor list
(predictors), lower censoring limit (`ll`, required), optional upper censoring
limit (`ul`), robust covariance flag (`robust`), cluster grouping variable
(`cluster(<var>)`), and intercept inclusion (`noconstant`). It does not inspect
schemas, load data, perform likelihood optimization, initialize backends, or
mutate session model state.

Supported syntax for `tobit` in this slice:

- `tobit <y> <xvars>, ll(<num>)` (defaults: upper_limit None, robust false, cluster_variable None, include_intercept true);
- `ul(<num>)` option for upper limit;
- `robust` flag option;
- `cluster(<var>)` option with a single clustering variable;
- `noconstant` flag option (sets `include_intercept: false`);
- Quoted variable names for outcome and predictors (e.g. `'outcome col'`, `` `x col` ``).

Execution is explicitly deferred at runtime returning
`RuntimeError::UnsupportedCommand { name: "tobit" }`.

## Python contract

Pinned authority:

- repository: `https://github.com/SaehwanPark/tabdat-explore.git`;
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`;
- tree: `601b236788872323af9277d2276a236154a0f129`;
- package: `0.25.0`; and
- `uv.lock` SHA-256:
  `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

Relevant authority paths:

- `src/tabdat/parser.py:2141-2171` — `_parse_tobit` implementation and option validations;
- `src/tabdat/models.py:594-603` — `TobitCommand` AST dataclass;
- `tests/test_parser.py:825-845` (`test_parse_phase_15_tobit_command`) — valid syntax fixtures;
- `tests/test_parser.py:1593-1603` — malformed syntax rejections.

Exact error messages from Python oracle:
- Missing arguments / condition / assignment syntax:
  `tobit expects syntax: tobit <y> <xvars>, ll(<num>) [ul(<num>)]`
- Missing `ll` option (when syntax otherwise valid):
  `tobit option ll expects one numeric value`
- `ll()` empty: `option ll expects at least one value`
- `ll(abc)` non-numeric: `option ll expects a numeric value`
- `ll` flag: `tobit option ll expects a numeric value`
- `ll` repeated: `tobit option ll may only be supplied once`
- `ul()` empty: `option ul expects at least one value`
- `ul(abc)` non-numeric: `option ul expects a numeric value`
- `ul` flag: `tobit option ul expects a numeric value`
- `ul` repeated: `tobit option ul may only be supplied once`
- `cluster()` empty: `option cluster expects at least one value`
- `cluster` flag: `tobit option cluster expects variables`
- `cluster(c1 c2)` multiple vars: `tobit option cluster expects one variable`
- `cluster` repeated: `tobit option cluster may only be supplied once`
- `robust` combined with `cluster`: `tobit cannot combine robust and cluster`
- Flag with value (`robust=true`, `noconstant=true`):
  `tobit option <name> does not accept a value`
- Unsupported option: `tobit unsupported option: <name>`

## Rust contract

In `tabdat-language`:
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TobitCommand {
    pub outcome: String,
    pub predictors: Vec<String>,
    pub lower_limit: String,
    pub upper_limit: Option<String>,
    pub robust: bool,
    pub cluster_variable: Option<String>,
    pub include_intercept: bool,
}
```
Numeric text for `lower_limit` and `upper_limit` remains an owned `String` so the
public `Command` enum retains its `Eq` derive; floating-point conversion and
censoring limits belong to a future statistical boundary.

Add `Command::Tobit { command: TobitCommand }` to the `Command` enum.
In `tabdat-runtime`:
Update `command_name` to return `"tobit"`, ensuring `Session::execute` returns
`RuntimeError::UnsupportedCommand { name: "tobit" }`.

## Test contract

Focused tests in `tabdat-language`:
- Valid syntax: outcome + predictors, lower limit, upper limit, robust, cluster, noconstant, quoted variables;
- Malformed syntax rejections with exact diagnostic strings;
- Punctuation guards: `tobit,`, `tobit=`, `tobit:`, `tobit==`.

Focused tests in `tabdat-runtime`:
- `crates/tabdat-runtime/tests/tobit_contract.rs`:
  confirming parsed `tobit outcome x1, ll(0)` returns `RuntimeError::UnsupportedCommand { name: "tobit" }`.
