# Bounded syntax-only `logit` and `probit` command contract

Status: contract checkpoint; implementation and validation are in progress.

Producer: task owner, using `tabdat-migration`, `simple-code-writer`, and
`preferred-workflow`.
Consumer: the bounded Rust language/parser implementation and its focused review.

## Scope

Add the backend-independent parser boundary for the documented binary-response
commands:
- `logit <y> <xvars> [, options]`
- `probit <y> <xvars> [, options]`

The parser owns the dependent variable (outcome), the ordered regressor list
(predictors), covariance specification (`robust` or `cluster(<var>)`), and
intercept inclusion (`noconstant`). It does not inspect schemas, load data,
perform optimization or maximum likelihood estimation, initialize backends,
or mutate session model state.

Supported syntax for both `logit` and `probit` in this slice:

- `<cmd> <y> <xvars>` (defaults: no robust/cluster, include intercept);
- `robust` flag option;
- `cluster(<var>)` single-variable option (mutually exclusive with `robust`);
- `noconstant` flag option (sets `include_intercept: false`);
- Quoted variable names for outcome, predictors, and option arguments (e.g. `'outcome col'`, `` `var name` ``).

Execution is explicitly deferred at runtime returning
`RuntimeError::UnsupportedCommand { name: "logit" }` and
`RuntimeError::UnsupportedCommand { name: "probit" }`.

## Python contract

Pinned authority:

- repository: `https://github.com/SaehwanPark/tabdat-explore.git`;
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`;
- tree: `601b236788872323af9277d2276a236154a0f129`;
- package: `0.25.0`; and
- `uv.lock` SHA-256:
  `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

Relevant authority paths:

- `src/tabdat/parser.py:2089-2139` — `_parse_logit` and `_parse_probit` implementations and option validations;
- `src/tabdat/models.py:532-549` — `LogitCommand` and `ProbitCommand` AST dataclasses;
- `tests/test_parser.py:781-825` (`test_parse_phase_15_logit_command`, `test_parse_phase_15_probit_command`) — valid syntax fixtures;
- `tests/test_parser.py:1561-1574` — malformed syntax rejections.

Exact error messages from Python oracle (for `<cmd>` in `logit`, `probit`):
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
pub struct LogitCommand {
  pub outcome: String,
  pub predictors: Vec<String>,
  pub robust: bool,
  pub cluster_variable: Option<String>,
  pub include_intercept: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbitCommand {
  pub outcome: String,
  pub predictors: Vec<String>,
  pub robust: bool,
  pub cluster_variable: Option<String>,
  pub include_intercept: bool,
}
```
Add `Command::Logit { command: LogitCommand }` and `Command::Probit { command: ProbitCommand }`
to `Command` enum.
In `tabdat-runtime`:
Update `command_name` to return `"logit"` and `"probit"`, ensuring `Session::execute` returns
`RuntimeError::UnsupportedCommand { name: "<cmd>" }`.

## Test contract

Focused tests in `tabdat-language`:
- Valid syntax: outcome + predictors, robust, cluster, noconstant, quoted variables;
- Malformed syntax rejections with exact diagnostic strings;
- Punctuation guards: `<cmd>,`, `<cmd>=`, `<cmd>:`.

Focused test in `tabdat-runtime`:
- `crates/tabdat-runtime/tests/logit_contract.rs`:
  Verify `session.execute(command)` returns `RuntimeError::UnsupportedCommand { name: "logit" }`.
- `crates/tabdat-runtime/tests/probit_contract.rs`:
  Verify `session.execute(command)` returns `RuntimeError::UnsupportedCommand { name: "probit" }`.

Python probe:
```sh
cd ../tabdat-explore
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py -k 'test_parse_phase_15_logit_command or test_parse_phase_15_probit_command'
```

## State and deferrals

Pure parser and language AST definition. Estimation execution, numerical algorithms,
post-estimation results, and session model state remain explicitly deferred.
