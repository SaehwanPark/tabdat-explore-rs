# Bounded `streg` syntax contract

## Slice scope

Port bounded direct `streg` parametric survival regression syntax into `tabdat-language` and
map `Command::Streg` to its deferred `RuntimeError::UnsupportedCommand { name: "streg" }` in
`tabdat-runtime`.

Scope:
- Command name: `streg` (case-insensitive)
- Syntax: `streg <time_var> <xvars>, failure(<event>) dist(<weibull|exponential>) [robust] [cluster(<var>)] [noconstant]`
- AST representation:
  - `StregDistribution` enum: `Weibull`, `Exponential`
  - `StregCommand` struct:
    - `time_variable: String`
    - `predictors: Vec<String>`
    - `failure_variable: String`
    - `distribution: StregDistribution`
    - `robust: bool`
    - `cluster_variable: Option<String>`
    - `include_intercept: bool`
  - `Command::Streg { command: StregCommand }`
- Diagnostics and validation:
  - Missing time variable or predictors (fewer than 2 arguments): `streg expects syntax: streg <time_var> <xvars>, failure(<event>) dist(...)`
  - Conditions (`if ...`): `streg expects syntax: streg <time_var> <xvars>, failure(<event>) dist(...)`
  - Expression assignment (`streg time = age`): `streg expects syntax: streg <time_var> <xvars>, failure(<event>) dist(...)`
  - Assignment without target (`streg=`, `streg = foo`): `streg assignment requires a target before =`
  - Unsupported tokens: `streg:`, `streg==` -> `unsupported token in command: ...`
  - Missing required `failure` option: `streg option failure expects one variable`
  - Empty `failure()` option: `option failure expects at least one value`
  - Non-single `failure(...)`: `streg option failure expects one variable`
  - Repeated `failure`: `streg option failure may only be supplied once`
  - Invalid `failure=var`: `streg option failure expects variables`
  - Missing required `dist` option: `streg option dist expects one value`
  - Empty `dist()` option: `option dist expects at least one value`
  - Non-single `dist(...)`: `streg option dist expects one value`
  - Repeated `dist`: `streg option dist may only be supplied once`
  - Invalid distribution value: `streg option dist must be weibull or exponential` (case-insensitive: accepts `weibull`, `exponential`, `Weibull`, `EXPONENTIAL`)
  - Invalid `dist=weibull`: `streg option dist expects variables`
  - Unsupported options: `streg unsupported option: <sorted comma-separated names>`
  - Flag options with values: `streg option <robust|noconstant> does not accept a value`
  - Empty `cluster()`: `option cluster expects at least one value`
  - Non-single `cluster(...)`: `streg option cluster expects one variable`
  - Repeated `cluster`: `streg option cluster may only be supplied once`
  - Invalid `cluster=var`: `streg option cluster expects variables`
  - Conflict between `robust` and `cluster`: `streg cannot combine robust and cluster`
- Explicit deferrals:
  - Survival time distribution fitting (maximum likelihood optimization)
  - Post-estimation (`predict`, `estat`)
  - Model result storage and display
  - CLI, JSON, MCP execution surfaces
