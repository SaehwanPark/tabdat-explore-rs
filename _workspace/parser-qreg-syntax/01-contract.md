# Bounded syntax-only `qreg` command contract

Status: contract checkpoint; implementation and validation are in progress.

Producer: task owner, using `tabdat-migration`, `simple-code-writer`, and
`preferred-workflow`.
Consumer: the bounded Rust language/parser implementation and its focused review.

## Scope

Add the backend-independent parser boundary for the documented quantile
regression command:
- `qreg <y> <xvars> [, quantile(<num>) robust noconstant]`

The parser owns the dependent variable (outcome), the ordered regressor list
(predictors), the quantile value (`quantile`, default 0.5), robust covariance
flag (`robust`), and intercept inclusion (`noconstant`). It does not inspect
schemas, load data, perform linear programming or quantile loss optimization,
initialize backends, or mutate session model state.

Supported syntax for `qreg` in this slice:

- `qreg <y> <xvars>` (defaults: quantile 0.5, robust false, include intercept true);
- `quantile(<num>)` option with a numeric value strictly between 0 and 1 (0 < q < 1);
- `robust` flag option;
- `noconstant` flag option (sets `include_intercept: false`);
- Quoted variable names for outcome and predictors (e.g. `'outcome col'`, `` `x col` ``).

Execution is explicitly deferred at runtime returning
`RuntimeError::UnsupportedCommand { name: "qreg" }`.

## Python contract

Pinned authority:

- repository: `https://github.com/SaehwanPark/tabdat-explore.git`;
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`;
- tree: `601b236788872323af9277d2276a236154a0f129`;
- package: `0.25.0`; and
- `uv.lock` SHA-256:
  `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

Relevant authority paths:

- `src/tabdat/parser.py:2012-2034` — `_parse_qreg` implementation and option validations;
- `src/tabdat/models.py:558-564` — `QregCommand` AST dataclass;
- `tests/test_parser.py:442-462` (`test_parse_phase_17_qreg_command`) — valid syntax fixtures;
- `tests/test_parser.py:1634-1643` — malformed syntax rejections.

Exact error messages from Python oracle:
- Missing arguments: `qreg expects syntax: qreg <y> <xvars>`
- Condition clause (`if ...`): `qreg expects syntax: qreg <y> <xvars>`
- Assignment syntax: `qreg expects syntax: qreg <y> <xvars>`
- Quantile without value: `option quantile expects at least one value`
- Quantile non-numeric: `option quantile expects a numeric value`
- Quantile <= 0 or >= 1: `qreg option quantile must be between 0 and 1`
- Quantile repeated: `qreg option quantile may only be supplied once`
- Flag with value: `qreg option <name> does not accept a value`
- Unsupported option: `qreg unsupported option: <name>`

## Rust contract

In `tabdat-language`:
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QregCommand {
    pub outcome: String,
    pub predictors: Vec<String>,
    pub quantile: String,
    pub robust: bool,
    pub include_intercept: bool,
}
```
Numeric text for `quantile` remains an owned `String` (default `"0.5"`) so the
public `Command` enum retains its `Eq` derive; floating-point conversion and
loss optimization belong to a future statistical boundary.

Add `Command::Qreg { command: QregCommand }` to the `Command` enum.
In `tabdat-runtime`:
Update `command_name` to return `"qreg"`, ensuring `Session::execute` returns
`RuntimeError::UnsupportedCommand { name: "qreg" }`.

## Test contract

Focused tests in `tabdat-language`:
- Valid syntax: outcome + predictors, default quantile 0.5, custom quantile (e.g. 0.25), robust, noconstant, quoted variables;
- Malformed syntax rejections with exact diagnostic strings;
- Punctuation guards: `qreg,`, `qreg=`, `qreg:`.

Focused tests in `tabdat-runtime`:
- `crates/tabdat-runtime/tests/qreg_contract.rs`:
  confirming parsed `qreg outcome x1 x2` returns `RuntimeError::UnsupportedCommand { name: "qreg" }`.
