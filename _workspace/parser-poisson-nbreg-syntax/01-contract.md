# Bounded syntax-only `poisson` and `nbreg` command contract

Status: contract checkpoint; implementation and validation are in progress.

Producer: task owner, using `tabdat-migration`, `simple-code-writer`, and
`preferred-workflow`.
Consumer: the bounded Rust language/parser implementation and its focused review.

## Scope

Add the backend-independent parser boundary for the documented count-model
commands:
- `poisson <y> <xvars> [, options]`
- `nbreg <y> <xvars> [, options]`

The parser owns the dependent variable (outcome), the ordered regressor list
(predictors), covariance specification (`robust` or `cluster(<var>)`), and
intercept inclusion (`noconstant`). It does not inspect schemas, load data,
perform optimization or maximum likelihood estimation, initialize backends,
or mutate session model state.

Supported syntax for both `poisson` and `nbreg` in this slice:

- `<cmd> <y> <xvars>` (defaults: no robust/cluster, include intercept);
- `robust` flag option;
- `cluster(<var>)` single-variable option (mutually exclusive with `robust`);
- `noconstant` flag option (sets `include_intercept: false`);
- Quoted variable names for outcome, predictors, and option arguments (e.g. `'outcome col'`, `` `var name` ``).

Execution is explicitly deferred at runtime returning
`RuntimeError::UnsupportedCommand { name: "poisson" }` and
`RuntimeError::UnsupportedCommand { name: "nbreg" }`.

## Python contract

Pinned authority:

- repository: `https://github.com/SaehwanPark/tabdat-explore.git`;
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`;
- tree: `601b236788872323af9277d2276a236154a0f129`;
- package: `0.25.0`; and
- `uv.lock` SHA-256:
  `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

Relevant authority paths:

- `src/tabdat/parser.py:2241-2290` — `_parse_poisson` and `_parse_nbreg` implementations and option validations;
- `src/tabdat/models.py:596-613` — `PoissonCommand` and `NbregCommand` AST dataclasses;
- `tests/test_parser.py:915-958` (`test_parse_phase_16_poisson_command`, `test_parse_phase_16_nbreg_command`) — valid syntax fixtures;
- `tests/test_parser.py:1627-1633, 1700-1706` — malformed syntax rejections.

Exact error messages from Python oracle (for `<cmd>` in `poisson`, `nbreg`):
- Missing arguments: `<cmd> expects syntax: <cmd> <y> <xvars>`
- Condition clause (`if ...`): `<cmd> expects syntax: <cmd> <y> <xvars>`
- Assignment syntax: `<cmd> expects syntax: <cmd> <y> <xvars>`
- Combining robust and cluster: `<cmd> cannot combine robust and cluster`
- Cluster without args or flag: `<cmd> option cluster expects variables`
- Cluster empty: `option cluster expects at least one value`
- Cluster multiple vars: `<cmd> option cluster expects one variable`
- Cluster repeated: `<cmd> option cluster may only be supplied once`
- Flag with value: `<cmd> option <name> does not accept a value`
- Unsupported option: `<cmd> unsupported option: <name>`

## Rust contract

In `tabdat-language`:
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoissonCommand {
    pub outcome: String,
    pub predictors: Vec<String>,
    pub robust: bool,
    pub cluster_variable: Option<String>,
    pub include_intercept: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NbregCommand {
    pub outcome: String,
    pub predictors: Vec<String>,
    pub robust: bool,
    pub cluster_variable: Option<String>,
    pub include_intercept: bool,
}
```
Add `Command::Poisson { command: PoissonCommand }` and `Command::Nbreg { command: NbregCommand }`
to the `Command` enum.
In `tabdat-runtime`:
Update `command_name` to return `"poisson"` and `"nbreg"`, ensuring `Session::execute` returns
`RuntimeError::UnsupportedCommand { name: "<cmd>" }`.

## Test contract

Focused tests in `tabdat-language`:
- Valid syntax: outcome + predictors, robust, cluster, noconstant, quoted variables;
- Malformed syntax rejections with exact diagnostic strings;
- Punctuation guards: `<cmd>,`, `<cmd>=`, `<cmd>:`.

Focused tests in `tabdat-runtime`:
- `crates/tabdat-runtime/tests/poisson_contract.rs`:
  confirming parsed `poisson outcome x1 x2` returns `RuntimeError::UnsupportedCommand { name: "poisson" }`.
- `crates/tabdat-runtime/tests/nbreg_contract.rs`:
  confirming parsed `nbreg outcome x1 x2` returns `RuntimeError::UnsupportedCommand { name: "nbreg" }`.
