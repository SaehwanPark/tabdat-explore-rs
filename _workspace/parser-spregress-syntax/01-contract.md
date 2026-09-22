# Bounded `spregress` syntax contract

## Slice scope

Port bounded direct `spregress` spatial econometrics regression syntax into `tabdat-language` and
map `Command::Spregress` to its deferred `RuntimeError::UnsupportedCommand { name: "spregress" }` in
`tabdat-runtime`.

Scope:
- Command name: `spregress` (case-insensitive)
- Syntax: `spregress <y> <xvars>, [coord(<lat_var> <lon_var>) [knn(<k>)] | weights(<path_to_file>) id(<id_var>) [contiguity(queen|rook)]] [model(<lag|error|sarar>) robust]`
- AST representation:
  - `SpregressModelType` enum: `Lag`, `Error`, `Sarar`
  - `SpregressContiguity` enum: `Queen`, `Rook`
  - `SpregressCommand` struct:
    - `outcome: String`
    - `predictors: Vec<String>`
    - `model_type: SpregressModelType` (default `Lag`)
    - `coord_variables: Option<(String, String)>`
    - `knn: Option<i64>` (default 5 when `coord` is used)
    - `weights_file: Option<String>`
    - `id_variable: Option<String>`
    - `contiguity: Option<SpregressContiguity>` (default `Queen` when `weights` is used)
    - `robust: bool`
  - `Command::Spregress { command: SpregressCommand }`
- Diagnostics and validation:
  - Missing dependent variable or predictors (fewer than 2 arguments): `spregress expects syntax: spregress <y> <xvars>, [coord(<lat_var> <lon_var>) [knn(<k>)] | weights(<path_to_file>) id(<id_var>) [contiguity(queen|rook)]] [model(<lag|error|sarar>) robust]`
  - Conditions (`if ...`): syntax error
  - Expression assignment (`spregress y = x`): syntax error
  - Assignment without target (`spregress=`, `spregress = foo`): `spregress assignment requires a target before =`
  - Unsupported tokens: `spregress:`, `spregress==` -> `unsupported token in command: ...`
  - Missing spatial weights / coordinates: `spregress requires either coord() or weights() option`
  - Both coord and weights supplied: `spregress option coord and weights are mutually exclusive`
  - `coord` variables count != 2: `spregress option coord expects exactly two variables representing latitude and longitude coordinates`
  - Option `id` with `coord`: `spregress option id can only be used with weights() option`
  - Option `contiguity` with `coord`: `spregress option contiguity can only be used with weights() option`
  - Option `knn` with `weights`: `spregress option knn/coord can only be used with coord() option`
  - Missing `id` when `weights` is specified: `spregress option id() is required when weights() is specified`
  - Option `knn` must be >= 1: `spregress option knn must be at least 1`
  - Option `model` validation: must be `'lag'`, `'error'`, or `'sarar'` -> `spregress option model must be 'lag', 'error', or 'sarar'`
  - Option `contiguity` validation: must be `'queen'` or `'rook'` -> `spregress option contiguity must be 'queen' or 'rook'`
  - Unsupported options: `spregress unsupported option: <sorted comma-separated names>`
  - Option single-use rules: `spregress option <name> may only be supplied once`
  - Flag validation for `robust`: `spregress option robust does not accept a value`
- Explicit deferrals:
  - Spatial weight matrix construction (k-NN / PySAL / Shapefile)
  - 2SLS / GM / SARAR spatial estimator execution
  - Spatial prediction (`xb`, `spatial_lag`)
  - Spatial diagnostics (`moran`, `lm`)
  - Model result storage and display
  - CLI, JSON, MCP execution surfaces
