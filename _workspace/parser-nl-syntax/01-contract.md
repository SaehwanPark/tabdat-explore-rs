# Bounded syntax-only `nl` command contract

Status: contract checkpoint; implementation and validation are in progress.

Producer: task owner, using `tabdat-migration`, `simple-code-writer`, and
`preferred-workflow`.
Consumer: the bounded Rust language/parser implementation and its focused review.

## Scope

Add the backend-independent parser boundary for the documented nonlinear
regression command:
- `nl <y> = <expr>, params(<params>) start(<values>) [robust] [noconstant]`

The parser owns the primary dependent variable (outcome), the mathematical
expression (`<expr>`, parsed via the shared expression AST), ordered unique
parameter names (`params(<params>)`, required), starting numeric values
(`start(<values>)`, required, matching parameter count), robust covariance flag
(`robust`), and intercept inclusion flag (`noconstant`). It does not inspect
schemas, load data, perform iterative nonlinear least squares optimization
(e.g., Gauss-Newton or Levenberg-Marquardt), initialize backends, or mutate
session model state.

Supported syntax for `nl` in this slice:

- `nl <y> = <expr>, params(<params>) start(<values>)` (defaults: robust false, include_intercept true);
- `params(<params>)` option with one or more unique parameter identifiers;
- `start(<values>)` option with one or more numeric starting values matching params count;
- `robust` flag option;
- `noconstant` flag option (sets `include_intercept: false`);
- Quoted variable names for outcome and column references in `<expr>`.

Execution is explicitly deferred at runtime returning
`RuntimeError::UnsupportedCommand { name: "nl" }`.

## Python contract

Pinned authority:

- repository: `https://github.com/SaehwanPark/tabdat-explore.git`;
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`;
- tree: `601b236788872323af9277d2276a236154a0f129`;
- package: `0.25.0`; and
- `uv.lock` SHA-256:
  `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

Relevant authority paths:

- `src/tabdat/parser.py:2207-2238` — `_parse_nl` implementation and option validations;
- `src/tabdat/models.py:617-624` — `NlCommand` AST dataclass;
- `docs/commands/nl.md` — command syntax and documentation;
- `src/tabdat/help/topics/nl.md` — help reference topic;
- `tests/test_parser.py:1617-1626` — malformed syntax rejections.

Exact error messages from Python oracle:
- Missing `=` or arguments:
  `nl expects syntax: nl <y> = <expr>, params(<params>) start(<values>)`
- Condition clause before `=` or missing target before `=`:
  `nl assignment requires a target before =`
- Incomplete assignment without expression after `=`:
  `nl assignment requires an expression after =`
- If clause after `=`:
  `duplicate if clause`
- Missing `params` option:
  `nl option params expects one-or-more parameter names`
- `params()` empty: `option params expects at least one value`
- `params` flag without parens: `nl option params expects variables`
- `params` repeated: `nl option params may only be supplied once`
- `params` with duplicates: `nl option params must not repeat parameter names`
- `params` with non-identifiers: `option params values must be identifiers`
- Missing `start` option:
  `nl option start expects one-or-more numeric values`
- `start()` empty: `option start expects at least one value`
- `start` flag without parens: `nl option start expects variables`
- `start` repeated: `nl option start may only be supplied once`
- `start` with non-numeric tokens: `option start values must be numeric`
- `start` count != `params` count:
  `nl option start count must match params count`
- Flag with value (`robust=true`, `noconstant=true`):
  `nl option <name> does not accept a value`
- Unsupported option: `nl unsupported option: <name>`
- Punctuation:
  - `nl:` => `unsupported token in command: :`
  - `nl=` => `nl assignment requires a target before =`
  - `nl==` => `unsupported token in command: ==`

## Rust contract

In `tabdat-language`:
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NlCommand {
    pub outcome: String,
    pub expression: GenerateExpression,
    pub parameter_names: Vec<String>,
    pub start_values: Vec<String>,
    pub robust: bool,
    pub include_intercept: bool,
}
```

Add `Command::Nl { command: NlCommand }` to the `Command` enum.
In `tabdat-runtime`:
Update `command_name` to return `"nl"`, ensuring `Session::execute` returns
`RuntimeError::UnsupportedCommand { name: "nl" }`.

## Test contract

Focused tests in `tabdat-language`:
- Valid syntax: outcome + expression, params, start values, robust, noconstant, quoted variables;
- Malformed syntax rejections with exact diagnostic strings;
- Punctuation guards: `nl=`, `nl:`, `nl==`.

Focused tests in `tabdat-runtime`:
- `crates/tabdat-runtime/tests/nl_contract.rs`:
  confirming parsed `nl y = a + b*x, params(a b) start(1 2)` returns
  `RuntimeError::UnsupportedCommand { name: "nl" }`.
