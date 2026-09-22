# Implementation Notes: Bayesian Linear Regression Syntax Slice

## Implementation Details

1. **AST Representation (`tabdat-language`)**:
   - Added `BayesCommand`:
     - `outcome: String`
     - `predictors: Vec<String>`
     - `n_iter: i64` (default `300`)
     - `tol: String` (default `"0.001"`)
     - `include_intercept: bool` (default `true`)
   - Added `Command::Bayes { command: BayesCommand }` variant to `Command`.

2. **Parser (`tabdat-language`)**:
   - Implemented `parse_bayes_command` using `parse_regularized_linear_command("bayes", body)`.
   - Option extractors:
     - `extract_n_iter_option`: validates single supply, integer format (`fract() == 0.0`), finite bounds, and minimum >= 1.
     - `extract_tol_option`: validates single supply, finite positive float (`val > 0.0`), and preserves string spelling for `Eq` compatibility.
     - `noconstant`: validates flag usage (`does not accept a value`).
   - Unsupported option detection: sorts, dedups, and emits `"bayes unsupported option: <sorted options>"`.
   - Reused existing delimiter guards:
     - Colon prefix disambiguation: `bayes:` or `bayes, options:` is routed to `parse_bayes_prefix_command`.
     - Assignment guards: `bayes=` or `bayes = 1` -> `"bayes assignment requires a target before ="`.
     - Equality guard: `bayes==` -> `"unsupported token in command: =="`.

3. **Runtime Deferral (`tabdat-runtime`)**:
   - Mapped `Command::Bayes { .. }` in `unsupported_command_name` to `"bayes"`, producing `RuntimeError::UnsupportedCommand { name: "bayes" }`.

4. **Verification**:
   - Unit tests in `tabdat-language` covering valid syntax, options, case-insensitivity, backtick quoting, and diagnostic error cases.
   - Integration contract tests in `tabdat-runtime/tests/bayes_linear_contract.rs`.
   - Workspace checks passed:
     - `cargo fmt --all -- --check`
     - `cargo check --locked --workspace --all-targets`
     - `cargo test --locked --workspace --all-targets`
     - `cargo clippy --locked --workspace --all-targets -- -D warnings`
     - `cargo deny check`
     - `cargo audit -D warnings`
