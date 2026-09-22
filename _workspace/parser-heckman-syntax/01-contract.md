# Bounded syntax-only `heckman` command contract

Status: contract checkpoint; implementation and validation are in progress.

Producer: task owner, using `tabdat-migration`, `simple-code-writer`, and
`preferred-workflow`.
Consumer: the bounded Rust language/parser implementation and its focused review.

## Scope

Add the backend-independent parser boundary for the documented Heckman
sample-selection regression command:
- `heckman <y> <xvars>, selectdep(<var>) select(<vars>) [robust | cluster(<var>)] [noconstant]`

The parser owns the primary dependent variable (outcome), ordered regressors
(predictors), selection equation dependent variable (`selectdep(<var>)`, required),
selection equation predictors (`select(<vars>)`, required), robust covariance
flag (`robust`), cluster grouping variable (`cluster(<var>)`), and intercept
inclusion (`noconstant`). It does not inspect schemas, load data, perform
two-step or full-information likelihood estimation, initialize backends, or
mutate session model state.

Supported syntax for `heckman` in this slice:

- `heckman <y> <xvars>, selectdep(<var>) select(<vars>)` (defaults: robust false, cluster_variable None, include_intercept true);
- `selectdep(<var>)` option with a single selection dependent variable;
- `select(<vars>)` option with one or more selection predictor variables;
- `robust` flag option;
- `cluster(<var>)` option with a single clustering variable;
- `noconstant` flag option (sets `include_intercept: false`);
- Quoted variable names for outcome, predictors, selection_dependent, and selection_predictors.

Execution is explicitly deferred at runtime returning
`RuntimeError::UnsupportedCommand { name: "heckman" }`.

## Python contract

Pinned authority:

- repository: `https://github.com/SaehwanPark/tabdat-explore.git`;
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`;
- tree: `601b236788872323af9277d2276a236154a0f129`;
- package: `0.25.0`; and
- `uv.lock` SHA-256:
  `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

Relevant authority paths:

- `src/tabdat/parser.py:2173-2204` — `_parse_heckman` implementation and option validations;
- `src/tabdat/models.py:605-614` — `HeckmanCommand` AST dataclass;
- `docs/commands/heckman.md` — command syntax and documentation;
- `src/tabdat/help/topics/heckman.md` — help reference topic;
- `tests/test_parser.py:1604-1616` — malformed syntax rejections.

Exact error messages from Python oracle:
- Missing arguments / condition / assignment syntax:
  `heckman expects syntax: heckman <y> <xvars>, selectdep(<var>) select(<vars>)`
- Missing `selectdep` option (when syntax otherwise valid):
  `heckman option selectdep expects one variable`
- `selectdep()` empty: `option selectdep expects at least one value`
- `selectdep` multiple variables: `heckman option selectdep expects one variable`
- `selectdep` flag without parens: `heckman option selectdep expects variables`
- `selectdep` repeated: `heckman option selectdep may only be supplied once`
- Missing `select` option (when `selectdep` is present):
  `heckman option select expects at least one variable`
- `select()` empty: `option select expects at least one value`
- `select` flag without parens: `heckman option select expects variables`
- `select` repeated: `heckman option select may only be supplied once`
- `cluster()` empty: `option cluster expects at least one value`
- `cluster` flag without parens: `heckman option cluster expects variables`
- `cluster(c1 c2)` multiple vars: `heckman option cluster expects one variable`
- `cluster` repeated: `heckman option cluster may only be supplied once`
- `robust` combined with `cluster`: `heckman cannot combine robust and cluster`
- Flag with value (`robust=true`, `noconstant=true`):
  `heckman option <name> does not accept a value`
- Unsupported option: `heckman unsupported option: <name>`
- Punctuation:
  - `heckman:` => `unsupported token in command: :`
  - `heckman=` => `heckman assignment requires a target before =`
  - `heckman==` => `unsupported token in command: ==`
  - `heckman,` => `comma must be followed by at least one option`
  - `heckman y x,` => `comma must be followed by at least one option`

## Rust contract

In `tabdat-language`:
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeckmanCommand {
    pub outcome: String,
    pub predictors: Vec<String>,
    pub selection_dependent: String,
    pub selection_predictors: Vec<String>,
    pub robust: bool,
    pub cluster_variable: Option<String>,
    pub include_intercept: bool,
}
```

Add `Command::Heckman { command: HeckmanCommand }` to the `Command` enum.
In `tabdat-runtime`:
Update `command_name` to return `"heckman"`, ensuring `Session::execute` returns
`RuntimeError::UnsupportedCommand { name: "heckman" }`.

## Test contract

Focused tests in `tabdat-language`:
- Valid syntax: outcome + predictors, selectdep, select, robust, cluster, noconstant, quoted variables;
- Malformed syntax rejections with exact diagnostic strings;
- Punctuation guards: `heckman,`, `heckman=`, `heckman:`, `heckman==`.

Focused tests in `tabdat-runtime`:
- `crates/tabdat-runtime/tests/heckman_contract.rs`:
  confirming parsed `heckman y x1 x2, selectdep(s) select(z1 z2)` returns
  `RuntimeError::UnsupportedCommand { name: "heckman" }`.
