# Bounded cross-validation regularized regression syntax contract (`cvlasso`, `cvridge`, `cvelasticnet`)

## Slice scope

Port bounded direct cross-validation regularized linear regression syntax (`cvlasso`, `cvridge`, `cvelasticnet`) into `tabdat-language` and
map `Command::Cvlasso`, `Command::Cvridge`, and `Command::Cvelasticnet` to their deferred `RuntimeError::UnsupportedCommand` in
`tabdat-runtime`.

Scope:
- Command names: `cvlasso`, `cvridge`, `cvelasticnet` (case-insensitive)
- Syntax:
  - `cvlasso linear <y> <xvars> [, cv(<int>) noconstant]`
  - `cvridge linear <y> <xvars> [, cv(<int>) noconstant]`
  - `cvelasticnet linear <y> <xvars> [, cv(<int>) l1_ratio(<float|floats>) noconstant]`
- AST representation:
  - `CvlassoCommand` struct:
    - `outcome: String`
    - `predictors: Vec<String>`
    - `cv: i64` (default `5`)
    - `include_intercept: bool` (default `true`)
  - `CvridgeCommand` struct:
    - `outcome: String`
    - `predictors: Vec<String>`
    - `cv: i64` (default `5`)
    - `include_intercept: bool` (default `true`)
  - `CvelasticnetL1Ratio` enum:
    - `Single(String)`
    - `Multiple(Vec<String>)`
  - `CvelasticnetCommand` struct:
    - `outcome: String`
    - `predictors: Vec<String>`
    - `cv: i64` (default `5`)
    - `l1_ratio: CvelasticnetL1Ratio` (default `Multiple(vec!["0.1", "0.5", "0.7", "0.9", "0.95", "0.99", "1.0"])`)
    - `include_intercept: bool` (default `true`)
  - `Command::Cvlasso(CvlassoCommand)`
  - `Command::Cvridge(CvridgeCommand)`
  - `Command::Cvelasticnet(CvelasticnetCommand)`
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
    - `"{cmd} option cv may only be supplied once"`
    - `"cvelasticnet option l1_ratio may only be supplied once"`
  - Flag options without arguments:
    - `"{cmd} option noconstant does not accept a value"`
  - Option validations:
    - `cv`:
      - bare/non-numeric: `"{cmd} option cv expects an integer value"`
      - non-integer (fract != 0): `"{cmd} option cv expects an integer value"`
      - integer < 2: `"{cmd} option cv must be at least 2"`
      - non-numeric value passed in parens: `"option cv expects a numeric value"`
    - `l1_ratio`:
      - bare: `"cvelasticnet option l1_ratio expects a numeric value or list of numeric values"`
      - non-numeric value in parens: `"option l1_ratio values must be numeric"`
      - value outside [0.0, 1.0]: `"cvelasticnet option l1_ratio values must be between 0 and 1 inclusive"`
- Explicit deferrals:
  - Direct Bayesian regression syntax (`bayes linear`)
  - Cross-validation grid search and K-fold split generation
  - Scikit-learn / coordinate descent regression engine execution
  - Grid search report generation and output file emission
  - Stored post-estimation results and prediction
  - CLI, JSON, MCP execution surfaces
