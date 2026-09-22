# Bounded syntax-only `zip` and `zinb` command contract

Status: contract checkpoint; implementation and validation are in progress.

Producer: task owner, using `tabdat-migration`, `simple-code-writer`, and
`preferred-workflow`.
Consumer: the bounded Rust language/parser implementation and its focused review.

## Scope

Add the backend-independent parser boundary for the documented zero-inflated
count-model commands:
- `zip <y> <xvars>, inflate(<zvars>) [, options]`
- `zinb <y> <xvars>, inflate(<zvars>) [, options]`

The parser owns the dependent variable (outcome), the ordered regressor list
(predictors), the ordered zero-inflation regressor list (`inflate_predictors`),
covariance specification (`robust` or `cluster(<var>)`), and intercept
inclusion (`noconstant`). It does not inspect schemas, load data, perform
optimization or maximum likelihood estimation, initialize backends, or mutate
session model state.

Supported syntax for both `zip` and `zinb` in this slice:

- `<cmd> <y> <xvars>, inflate(<zvars>)` (defaults: no robust/cluster, include intercept);
- `inflate(<zvars>)` mandatory option with one or more variables;
- `robust` flag option;
- `cluster(<var>)` single-variable option (mutually exclusive with `robust`);
- `noconstant` flag option (sets `include_intercept: false`);
- Quoted variable names for outcome, predictors, inflate predictors, and cluster variable (e.g. `'outcome col'`, `` `var name` ``).

Execution is explicitly deferred at runtime returning
`RuntimeError::UnsupportedCommand { name: "zip" }` and
`RuntimeError::UnsupportedCommand { name: "zinb" }`.

## Python contract

Pinned authority:

- repository: `https://github.com/SaehwanPark/tabdat-explore.git`;
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`;
- tree: `601b236788872323af9277d2276a236154a0f129`;
- package: `0.25.0`; and
- `uv.lock` SHA-256:
  `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

Relevant authority paths:

- `src/tabdat/parser.py:2293-2335` — `_parse_zip` and `_parse_zinb` implementations and option validations;
- `src/tabdat/models.py:616-633` — `ZipCommand` and `ZinbCommand` AST dataclasses;
- `tests/test_parser.py:959-974` (`test_parse_phase_16_zip_command`, `test_parse_phase_16_zinb_command`) — valid syntax fixtures;
- `tests/test_parser.py:1641-1647, 1714-1720` — malformed syntax rejections.

Exact error messages from Python oracle (for `<cmd>` in `zip`, `zinb`):
- Missing arguments: `<cmd> expects syntax: <cmd> <y> <xvars>, inflate(<zvars>)`
- Condition clause (`if ...`): `<cmd> expects syntax: <cmd> <y> <xvars>, inflate(<zvars>)`
- Assignment syntax: `<cmd> expects syntax: <cmd> <y> <xvars>, inflate(<zvars>)`
- Missing inflate option: `<cmd> option inflate expects one-or-more variables`
- Inflate without args or flag: `<cmd> option inflate expects variables`
- Inflate empty: `option inflate expects at least one value`
- Inflate repeated: `<cmd> option inflate may only be supplied once`
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
pub struct ZipCommand {
    pub outcome: String,
    pub predictors: Vec<String>,
    pub inflate_predictors: Vec<String>,
    pub robust: bool,
    pub cluster_variable: Option<String>,
    pub include_intercept: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZinbCommand {
    pub outcome: String,
    pub predictors: Vec<String>,
    pub inflate_predictors: Vec<String>,
    pub robust: bool,
    pub cluster_variable: Option<String>,
    pub include_intercept: bool,
}
```
Add `Command::Zip { command: ZipCommand }` and `Command::Zinb { command: ZinbCommand }`
to the `Command` enum.
In `tabdat-runtime`:
Update `command_name` to return `"zip"` and `"zinb"`, ensuring `Session::execute` returns
`RuntimeError::UnsupportedCommand { name: "<cmd>" }`.

## Test contract

Focused tests in `tabdat-language`:
- Valid syntax: outcome + predictors, inflate predictors, robust, cluster, noconstant, quoted variables;
- Malformed syntax rejections with exact diagnostic strings;
- Punctuation guards: `<cmd>,`, `<cmd>=`, `<cmd>:`.

Focused tests in `tabdat-runtime`:
- `crates/tabdat-runtime/tests/zip_contract.rs`:
  confirming parsed `zip outcome x1 x2, inflate(z1 z2)` returns `RuntimeError::UnsupportedCommand { name: "zip" }`.
- `crates/tabdat-runtime/tests/zinb_contract.rs`:
  confirming parsed `zinb outcome x1 x2, inflate(z1 z2)` returns `RuntimeError::UnsupportedCommand { name: "zinb" }`.
