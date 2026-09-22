# Bounded regularized regression syntax contract (`lasso`, `postlasso`, `ridge`, `elasticnet`)

## Slice scope

Port bounded direct regularized linear regression syntax (`lasso`, `postlasso`, `ridge`, `elasticnet`) into `tabdat-language` and
map `Command::Lasso`, `Command::Postlasso`, `Command::Ridge`, and `Command::Elasticnet` to their deferred `RuntimeError::UnsupportedCommand` in
`tabdat-runtime`.

Scope:
- Command names: `lasso`, `postlasso`, `ridge`, `elasticnet` (case-insensitive)
- Syntax:
  - `lasso linear <y> <xvars> [, alpha(<f64>) noconstant]`
  - `postlasso linear <y> <xvars> [, alpha(<f64>) robust noconstant]`
  - `ridge linear <y> <xvars> [, alpha(<f64>) noconstant]`
  - `elasticnet linear <y> <xvars> [, alpha(<f64>) l1_ratio(<f64>) noconstant]`
- AST representation:
  - `LassoCommand` struct:
    - `outcome: String`
    - `predictors: Vec<String>`
    - `alpha: String` (default `"1.0"`)
    - `include_intercept: bool` (default `true`)
  - `PostlassoCommand` struct:
    - `outcome: String`
    - `predictors: Vec<String>`
    - `alpha: String` (default `"1.0"`)
    - `robust: bool` (default `false`)
    - `include_intercept: bool` (default `true`)
  - `RidgeCommand` struct:
    - `outcome: String`
    - `predictors: Vec<String>`
    - `alpha: String` (default `"1.0"`)
    - `include_intercept: bool` (default `true`)
  - `ElasticnetCommand` struct:
    - `outcome: String`
    - `predictors: Vec<String>`
    - `alpha: String` (default `"1.0"`)
    - `l1_ratio: String` (default `"0.5"`)
    - `include_intercept: bool` (default `true`)
  - `Command::Lasso(LassoCommand)`
  - `Command::Postlasso(PostlassoCommand)`
  - `Command::Ridge(RidgeCommand)`
  - `Command::Elasticnet(ElasticnetCommand)`
- Diagnostics and validation:
  - Missing arguments (fewer than 3 arguments, or if condition/expression present):
    - `"{cmd} expects syntax: {cmd} linear <y> <xvars>"`
  - Model specifier not "linear":
    - `"{cmd} model must be linear"`
  - Punctuation / delimiter guards:
    - `"{cmd}:"` -> `"unsupported token in command: :"`
    - `"{cmd}="`, `"{cmd} ="` -> `"{cmd} assignment requires a target before ="`
    - `"{cmd}=="` -> `"unsupported token in command: =="`
  - Unsupported options:
    - `"{cmd} unsupported option: <sorted comma-separated names>"`
  - Single-use options:
    - `"{cmd} option alpha may only be supplied once"`
    - `"elasticnet option l1_ratio may only be supplied once"`
  - Flag options without arguments:
    - `"{cmd} option {opt} does not accept a value"`
  - Numeric validations:
    - `alpha`: must be positive float (`alpha <= 0.0` -> `"{cmd} option alpha must be positive"`)
    - `alpha`: if missing value or non-numeric -> `"option alpha expects a numeric value"`
    - `l1_ratio`: must be between 0.0 and 1.0 inclusive (`"elasticnet option l1_ratio must be between 0 and 1 inclusive"`)
    - `l1_ratio`: non-numeric -> `"option l1_ratio values must be numeric"`
    - `l1_ratio`: multiple values for elasticnet -> `"elasticnet option l1_ratio expects one value"`
- Explicit deferrals:
  - Cross-validation regularized regression (`cvlasso`, `cvridge`, `cvelasticnet`)
  - Bayesian regression (`bayes linear`)
  - Coordinate descent / proximal gradient optimization and coefficient shrinkage
  - Post-lasso OLS refitting and variance-covariance estimation
  - Cross-validation grid search and fold partitioning
  - Stored model results, residual calculation, and post-estimation prediction
  - CLI, JSON, MCP execution surfaces
