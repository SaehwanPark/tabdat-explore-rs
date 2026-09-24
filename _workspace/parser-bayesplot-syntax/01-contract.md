# Bounded Contract: Bayesian Diagnostic Plot Syntax Slice (`bayesplot`)

## 1. Context and Scope

This slice implements the syntax-only parser and AST representations for TabDat's
MCMC diagnostic visualization command: `bayesplot`. It fulfills Phase 7 (§7.4) visualization
language parsing requirements while keeping plot generation, posterior sample extraction,
Vega-Lite/plot rendering, artifact management, and browser/viewer interaction deferred to future
runtime slices.

- **Status**: Active development
- **Branch**: `feature/parser-bayesplot-syntax`
- **Related PRs**: Prior syntax slices (#11–#20, #45, #47, #59–#67, #69, #77, #79, #81, #83, #85, #87, #89, #91, #93, #95, #97, #99, #101, #103, #105, #107, #109, #111, #113, #115, #117, #119, #121, #123, #125, #127, #129)
- **Oracle Revision**: Pinned Python oracle checkout at `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`

---

## 2. Python Oracle Contract

### 2.1 Grammar and Accepted Forms
```stata
bayesplot <trace|density|autocorrelation> [, saving(<path>) noopen]
```
- Exactly one kind argument required:
  - Valid: `bayesplot trace`, `bayesplot density`, `bayesplot autocorrelation`,
    `bayesplot trace, noopen`, `bayesplot density, saving(posterior.svg)`,
    `bayesplot autocorrelation, saving("autocorr.png") noopen`.
  - Zero arguments (e.g. `bayesplot`, `bayesplot, saving(x.svg)`): rejected with
    `bayesplot expects syntax: bayesplot <trace|density|autocorrelation>`.
  - More than one argument (e.g. `bayesplot trace density`): rejected with
    `bayesplot expects syntax: bayesplot <trace|density|autocorrelation>`.
  - Invalid kind name (e.g. `bayesplot foo`, `bayesplot scatter`): rejected with
    `bayesplot kind must be trace, density, or autocorrelation`.
  - Case sensitivity: kind is case-sensitive (e.g. `bayesplot TRACE`): rejected with
    `bayesplot kind must be trace, density, or autocorrelation`.
- Predicates and assignment syntax:
  - If clause (e.g. `bayesplot trace if x > 0`): rejected with
    `bayesplot does not accept if clauses or assignment syntax`.
  - Assignment syntax (e.g. `bayesplot trace = 1`): rejected with
    `bayesplot does not accept if clauses or assignment syntax`.
  - Assignment without target (e.g. `bayesplot =`, `bayesplot = 1`, `bayesplot=1`): rejected with
    `bayesplot assignment requires a target before =`.
  - Assignment without expression (e.g. `bayesplot trace =`): rejected with
    `bayesplot does not accept if clauses or assignment syntax` (or `bayesplot assignment requires an expression after =` if bare).
  - Double equal (e.g. `bayesplot==1`): rejected with
    `unsupported token in command: ==`.
- Supported options:
  - `saving`: file path to save the generated plot artifact.
    - Syntax: `saving(<path>)` (e.g. `saving(figures/posterior.svg)`, `saving("my plot.png")`),
      or `saving = <path>` / `saving = "path"`.
    - If no value provided (e.g. `bayesplot trace, saving`): rejected with
      `bayesplot option saving expects a path`.
    - Duplicate specification (e.g. `saving(a) saving(b)`): rejected with
      `bayesplot option saving may only be supplied once`.
  - `noopen`: flag option instructing not to automatically open the artifact.
    - Default value for `open_artifact` is `true`. When `noopen` is passed, `open_artifact` becomes `false`.
    - If a value is provided (e.g. `noopen=1` or `noopen(yes)`): rejected with
      `bayesplot option noopen does not accept a value`.
    - Duplicate flag (e.g. `noopen noopen`) is accepted and sets `open_artifact = false`.
- Unsupported options:
  - Any options other than `saving`, `noopen` (e.g. `bayesplot trace, missing`, `bayesplot trace, bins=20`): rejected with
    `bayesplot unsupported option: <unsupported_option>`.
  - Multiple unsupported options are sorted alphabetically (e.g. `bayesplot trace, zebra apple`):
    `bayesplot unsupported option: apple, zebra`.
- Delimiter guards:
  - Attached colon (`bayesplot:`): Rejected with `unsupported token in command: :`.
  - Attached assignment (`bayesplot=`): Rejected with `bayesplot assignment requires a target before =`.
  - Attached double equal (`bayesplot==`): Rejected with `unsupported token in command: ==`.
  - Attached comma with nothing following (`bayesplot,`): Rejected with `comma must be followed by at least one option`.

### 2.2 Oracle Sources and References
- Python parser: `tabdat-explore/src/tabdat/parser.py:1590-1610` (`_parse_bayesplot`)
- Python model: `tabdat-explore/src/tabdat/models.py:408-413` (`BayesPlotCommand`)
- Python tests: `tabdat-explore/tests/test_parser.py:1283-1313` (`test_parse_phase_19_bayesplot_command`, `test_parse_phase_19_bayesplot_errors`)
- Python executor: `tabdat-explore/src/tabdat/executor.py:2626-2759` (`_execute_bayesplot`)

### 2.3 Diagnostic Rules and Precedence
1. Attached colon:
   - `unsupported token in command: :`
2. Attached assignment / double equal:
   - `bayesplot=...`: `bayesplot assignment requires a target before =`
   - `bayesplot==...`: `unsupported token in command: ==`
3. Trailing comma without options:
   - `bayesplot,`: `comma must be followed by at least one option`
4. Predicate or assignment syntax:
   - If clauses or assignment expressions: `bayesplot does not accept if clauses or assignment syntax`
   - Missing assignment target before `=`: `bayesplot assignment requires a target before =`
   - Missing assignment expression after `=`: `bayesplot assignment requires an expression after =`
5. Arity check:
   - `bayesplot expects syntax: bayesplot <trace|density|autocorrelation>`
6. Diagnostic kind validation:
   - `bayesplot kind must be trace, density, or autocorrelation`
7. Unsupported options:
   - `bayesplot unsupported option: <sorted_opts>`
8. Flag option value checks:
   - `bayesplot option noopen does not accept a value`
9. Option value checks:
   - `bayesplot option saving expects a path`
   - `bayesplot option saving may only be supplied once`

---

## 3. Rust AST and Language Architecture

### 3.1 AST Representation (`crates/tabdat-language/src/lib.rs`)
```rust
/// The diagnostic plot kinds supported by `bayesplot`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BayesPlotKind {
  /// Trace plot of MCMC iterations.
  Trace,
  /// Density plot of posterior draws.
  Density,
  /// Autocorrelation plot across MCMC lags.
  Autocorrelation,
}

/// Parsed `bayesplot` visualization specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BayesPlotCommand {
  /// The MCMC diagnostic plot kind (trace, density, or autocorrelation).
  pub kind: BayesPlotKind,
  /// Optional file path to save the generated plot.
  pub saving: Option<String>,
  /// Whether to open the generated artifact in the browser/viewer (default true).
  pub open_artifact: bool,
}
```
Add enum variant to `Command`:
```rust
  /// Diagnostic plot of Bayesian MCMC samples (visualization execution is deferred).
  BayesPlot {
    /// Parsed bayesplot command options and target plot kind.
    command: BayesPlotCommand,
  },
```

### 3.2 Parser Implementation
- Implement `parse_bayesplot_command(body: &str) -> Result<Command, ParseError>` using `first_unquoted_comma`, `parse_simple_body(..., false)`, and `parse_use_options`.
- Wire `bayesplot` into `parse_named_command`.
- Add colon guard `command.as_bytes().get(..9) == b"bayesplot"` with byte 9 == `b':'`.
- Add delimiter `=` guard `name.eq_ignore_ascii_case("bayesplot") && delimiter == '='`.
- Export `BayesPlotKind` and `BayesPlotCommand` from `tabdat_language`.

### 3.3 Runtime Wiring (`crates/tabdat-runtime/src/lib.rs`)
- In `command_name(&Command)`:
  - `Command::BayesPlot { .. } => "bayesplot"`
- In `execute(&mut self, command: Command)`:
  - Falls through to `_ => Err(RuntimeError::UnsupportedCommand { name: command_name })`.

---

## 4. Verification Plan

1. Unit tests in `crates/tabdat-language/src/lib.rs`:
   - Basic `bayesplot trace`, `bayesplot density`, `bayesplot autocorrelation` with default options (`saving: None`, `open_artifact: true`).
   - `bayesplot trace, noopen`.
   - `bayesplot density, saving(figures/posterior.svg)`.
   - `bayesplot autocorrelation, saving("my plot.png") noopen`.
   - Error cases:
     - Arity errors: `bayesplot`, `bayesplot trace density`, `bayesplot, noopen`.
     - Invalid kinds: `bayesplot foo`, `bayesplot TRACE`, `bayesplot "TRACE"`.
     - Condition clause: `bayesplot trace if x > 0`.
     - Assignment syntax: `bayesplot trace = 1`, `bayesplot = 1`, `bayesplot trace =`.
     - Double equal: `bayesplot==1`.
     - Attached colon: `bayesplot:`.
     - Trailing comma: `bayesplot,`.
     - Unsupported options: `bayesplot trace, missing`, `bayesplot trace, zebra apple`.
     - Malformed saving: `bayesplot trace, saving`.
     - Duplicate saving: `bayesplot trace, saving(a) saving(b)`.
     - Flags with value: `bayesplot trace, noopen=1`, `bayesplot trace, noopen(yes)`.
2. Integration contract tests in `crates/tabdat-language/tests/parser_contract.rs`.
3. Runtime contract test in `crates/tabdat-runtime/tests/bayesplot_contract.rs`:
   - Parse and execute `bayesplot trace` and verify `RuntimeError::UnsupportedCommand { name: "bayesplot" }`.
4. Workspace checks:
   - `cargo fmt --all -- --check`
   - `cargo check --locked --workspace --all-targets`
   - `cargo test --locked --workspace --all-targets`
   - `cargo clippy --locked --workspace --all-targets -- -D warnings`
