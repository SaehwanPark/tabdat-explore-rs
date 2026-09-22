# Bounded `bayes:` prefixed command syntax contract

## 1. Context and purpose

Phase 5.1 of the migration roadmap includes:

```markdown
- [ ] Port prefixed command syntax such as `bayes:`.
```

Following the bounded syntax slices for `regress` (PR #79) and `logit`/`probit` (PR #81),
this slice ports the syntax-only command prefix `bayes:` into `crates/tabdat-language`.
In Python TabDat (`src/tabdat/parser.py:433-509`), `bayes:` accepts an optional options
block before a colon (`:`), followed by an inner estimation command that is strictly
validated to be either `regress` or `logit`.

This slice establishes the language boundary types, option parsing (including MCMC
tuning parameters and custom priors), colon dispatch, exact diagnostic parity, and
parser-only runtime deferral.

## 2. Python oracle contract

- Pinned revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`
- Source files:
  - `src/tabdat/parser.py:295-300`: Routing logic for `bayes:`, `bayes,`, or `bayes` with colon.
  - `src/tabdat/parser.py:433-509`: `_parse_bayes_prefix_result` implementation.
  - `src/tabdat/parser.py:3247-3252`: Parenthesized numeric option parsing for `draws`, `burnin`, `tune`, `chains`, `thin`, `seed`, `rseed`.
  - `src/tabdat/parser.py:3259-3271`: Parenthesized `prior(variable, distribution)` parsing.
  - `src/tabdat/models.py:421-430`: `BayesPrefixCommand` definition.
  - `tests/test_parser.py:662-704`: Unit tests for `bayes:` prefix.

### Accepted syntax

```text
bayes [, options]: <command>
```

Where `<command>` is either a `regress` or `logit` command.

### Options

1. `draws(<int>)`: Number of MCMC draws (`Option<i64>`).
2. `burnin(<int>)` (alias `tune(<int>)`): Warmup iterations (`Option<i64>`).
3. `chains(<int>)`: Number of chains (`Option<i64>`).
4. `thin(<int>)`: Thinning interval (`Option<i64>`).
5. `seed(<int>)` (alias `rseed(<int>)`): Random seed (`Option<i64>`).
6. `prior(<variable>, <distribution>)`: Custom prior specifications, e.g. `prior(x, normal(0, 10))`. Multiple `prior` options accumulate in order.

### Error diagnostics

- Missing colon: `bayes prefix expects syntax: bayes [, options]: command`
- Options not starting with comma: `bayes prefix options must start with a comma`
- Empty command after colon: `bayes expects a command after :`
- Unsupported inner command: `bayes prefix only supports regress and logit commands`
- Unsupported option: `unsupported bayes option: <name>`
- Non-numeric option value: `<name> must be a numeric value`
- Malformed prior: `prior option expects prior(variable, distribution) syntax`

## 3. Rust contract

### Types in `crates/tabdat-language/src/lib.rs`

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BayesPrefixCommand {
  pub command: Box<Command>,
  pub draws: Option<i64>,
  pub burnin: Option<i64>,
  pub chains: Option<i64>,
  pub thin: Option<i64>,
  pub seed: Option<i64>,
  pub priors: Vec<(String, String)>,
}
```

Add variant to `Command`:

```rust
pub enum Command {
  // ...
  /// Run a Bayesian estimation model using MCMC sampling (execution is deferred).
  BayesPrefix { command: BayesPrefixCommand },
}
```

### Runtime deferral in `crates/tabdat-runtime/src/lib.rs`

Runtime execution remains an explicit unsupported-command result:
```rust
Command::BayesPrefix { .. } => "bayes"
```

## 4. Exclusions and deferrals

- MCMC sampling (No-U-Turn Sampler / Stan / PyMC / custom sampler) is deferred.
- Prior distribution validation and distribution calculus are deferred.
- Model results and posterior chain storage are deferred.
- Post-estimation commands (`estat bayes`, `bayesplot`) are deferred.
- Non-linear Bayesian models beyond `regress` and `logit` are deferred.
