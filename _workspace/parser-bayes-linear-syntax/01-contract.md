# Bounded Bayesian linear regression syntax contract (`bayes linear`)

## Slice scope

Port bounded direct Bayesian linear regression syntax (`bayes linear`) into `tabdat-language` and
map `Command::Bayes` to its deferred `RuntimeError::UnsupportedCommand` in `tabdat-runtime`.

Scope:
- Command name: `bayes` (case-insensitive)
- Syntax:
  - `bayes linear <y> <xvars> [, n_iter(<int>) tol(<float>) noconstant]`
- AST representation:
  - `BayesCommand` struct:
    - `outcome: String`
    - `predictors: Vec<String>`
    - `n_iter: i64` (default `300`)
    - `tol: String` (default `"0.001"`)
    - `include_intercept: bool` (default `true`)
  - `Command::Bayes(BayesCommand)`
- Diagnostics and validation:
  - Missing arguments (fewer than 3 arguments, or if condition/expression present):
    - `"bayes expects syntax: bayes linear <y> <xvars>"`
  - Model specifier not "linear":
    - `"bayes model must be linear"`
  - Punctuation / delimiter guards:
    - `bayes:` -> delegated to `parse_bayes_prefix_command`
    - `bayes=` / `bayes = 1` -> `"bayes assignment requires a target before ="`
    - `bayes==` -> `"unsupported token in command: =="`
  - Unsupported options:
    - `"bayes unsupported option: <sorted comma-separated names>"`
  - Single-use options:
    - `"bayes option n_iter may only be supplied once"`
    - `"bayes option tol may only be supplied once"`
  - Flag options without arguments:
    - `"bayes option noconstant does not accept a value"`
  - Option validations:
    - `n_iter`:
      - bare / non-numeric: `"bayes option n_iter expects an integer value"`
      - non-integer (fract != 0): `"bayes option n_iter expects an integer value"`
      - integer < 1: `"bayes option n_iter must be at least 1"`
      - non-numeric value in parens: `"option n_iter expects a numeric value"`
    - `tol`:
      - bare / non-numeric: `"bayes option tol expects a numeric value"`
      - float <= 0.0: `"bayes option tol must be positive"`
      - non-numeric value in parens: `"option tol expects a numeric value"`
  - Case insensitivity:
    - `BAYES LINEAR ...` supported.
    - Options `N_ITER(...)`, `TOL(...)`, `NOCONSTANT` recognized case-insensitively.
- Runtime behavior:
  - `tabdat-runtime` execution returns `RuntimeError::UnsupportedCommand { name: "bayes".into() }`.
