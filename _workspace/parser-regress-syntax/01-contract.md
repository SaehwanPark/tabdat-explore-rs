# Bounded syntax-only `regress` command contract

Status: contract checkpoint; implementation and validation are in progress.

Producer: task owner, using `tabdat-migration`, `simple-code-writer`, and
`preferred-workflow`.
Consumer: the bounded Rust language/parser implementation and its focused review.

## Scope

Add the backend-independent parser boundary for the documented
`regress <y> <xvars> [, options]` command. The parser owns the dependent variable
(outcome), the ordered regressor list (predictors), the chosen estimator
(`ols`, `wls`, or `gls`), an optional weight variable (`wls(<var>)` or `gls(<var>)`),
covariance specification (`robust` or `cluster(<var>)`), and intercept inclusion
(`noconstant`). It does not inspect schemas, load data, perform matrix arithmetic,
initialize estimation backends, or mutate session model state.

Supported syntax in this slice:

- `regress <y> <xvars>` (defaults: estimator `ols`, no weights, no robust/cluster, include intercept);
- `robust` flag option;
- `cluster(<var>)` single-variable option (mutually exclusive with `robust`);
- `noconstant` flag option (sets `include_intercept: false`);
- `wls(<var>)` single-variable option (sets estimator `wls` and `weight_variable`);
- `gls(<var>)` single-variable option (sets estimator `gls` and `weight_variable`, mutually exclusive with `wls`);
- Quoted variable names for outcome, predictors, and option arguments (e.g. `'outcome col'`, `` `var name` ``).

Execution is explicitly deferred at runtime returning
`RuntimeError::UnsupportedCommand { name: "regress" }`.

## Python contract

Pinned authority:

- repository: `https://github.com/SaehwanPark/tabdat-explore.git`;
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`;
- tree: `601b236788872323af9277d2276a236154a0f129`;
- package: `0.25.0`; and
- `uv.lock` SHA-256:
  `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

Relevant authority paths:

- `src/tabdat/parser.py:1627-1670` — `_parse_regress` implementation and option validations;
- `src/tabdat/models.py:461-470` — `RegressCommand` AST dataclass;
- `tests/test_parser.py:394-440` (`test_parse_phase_13_regress_command`) — valid syntax fixtures;
- `tests/test_parser.py:1545-1560` — malformed syntax rejections.

Exact error messages from Python oracle:
- Missing arguments: `regress expects syntax: regress <y> <xvars>`
- Condition clause (`if ...`): `regress expects syntax: regress <y> <xvars>`
- Assignment syntax: `regress expects syntax: regress <y> <xvars>`
- Combining robust and cluster: `regress cannot combine robust and cluster`
- Combining wls and gls: `regress cannot combine wls and gls`
- Cluster without args or flag: `regress option cluster expects variables`
- Cluster empty: `option cluster expects at least one value`
- Cluster multiple vars: `regress option cluster expects one variable`
- Cluster repeated: `regress option cluster may only be supplied once`
- WLS without args: `regress option wls expects variables`
- WLS empty: `option wls expects at least one value`
- WLS multiple vars: `regress option wls expects one variable`
- GLS without args: `regress option gls expects variables`
- GLS empty: `option gls expects at least one value`
- GLS multiple vars: `regress option gls expects one variable`
- Flag with value: `regress option <name> does not accept a value`
- Unsupported option: `regress unsupported option: <name>`

## Rust contract

In `tabdat-language`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegressEstimator {
  Ols,
  Wls,
  Gls,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegressCommand {
  pub outcome: String,
  pub predictors: Vec<String>,
  pub estimator: RegressEstimator,
  pub weight_variable: Option<String>,
  pub robust: bool,
  pub cluster_variable: Option<String>,
  pub include_intercept: bool,
}
```
Add `Command::Regress { command: RegressCommand }` to `Command` enum.
In `tabdat-runtime`:
Update `command_name` to return `"regress"`, and ensure `Session::execute` returns
`RuntimeError::UnsupportedCommand { name: "regress" }`.

## Test contract

Focused tests in `tabdat-language`:
- Valid syntax: outcome + predictors, robust, cluster, noconstant, wls, gls, quoted variables;
- Malformed syntax rejections with exact diagnostic strings;
- Punctuation guards: `regress,`, `regress=`, `regress:`.

Focused test in `tabdat-runtime`:
- `crates/tabdat-runtime/tests/regress_contract.rs`:
  Verify `session.execute(command)` returns `RuntimeError::UnsupportedCommand { name: "regress" }`.

Python probe:
```sh
cd ../tabdat-explore
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py -k 'test_parse_phase_13_regress_command'
```

## State and deferrals

Pure parser and language AST definition. Estimation execution, numerical algorithms,
post-estimation results, and session model state remain explicitly deferred.
