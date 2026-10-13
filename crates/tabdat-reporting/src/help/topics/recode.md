# recode

How to invoke:
`recode varlist (rule) [(rule) ...], generate(new_varlist)` or `..., replace`

What it does:
Map values or inclusive numeric ranges using existing bounded eager rules, generating one new column per source or replacing selected columns.

What problem it answers:
How do I clean values or create categories before tabulation/modeling?

Examples:
- `recode age (min/17=0) (18/max=1), generate(adult)`
- `recode grade (1 2 3=1) (4 5=2) (else=0), replace`
- `recode sex ('F'='female') (missing='unknown'), replace`

Notes:
- Exactly one of `generate()` or `replace` is required; conditions/options beyond those targets and lazy/remote execution are not installed. Generated targets must be new/distinct, one per source, and are appended in request order. Backtick-quoted names and empty data are supported.
- Conditional rules use original values, first match wins without cascading. Numeric ranges include endpoints; `min`/`max` are unbounded. `else` supplies the final fallback regardless of its position; unmatched values otherwise remain.
- Ranges require numeric source columns. Text output or nonnumeric sources use text conversion; native output types/promotion are reported, not forced to source types.
- `missing`/`nonmissing` means SQL NULL/non-NULL. Stored empty strings are nonmissing, as are stored NaN/infinity. CSV loading may interpret empty fields as NULL; Parquet retains stored empty strings. No general missing-value normalization is implied.
- Generate retains existing labels; successful replacement removes selected value-label attachments even when no rule matches. Variable labels/definitions and the original cached fit/sample remain; explicitly refit for estimation on recoded values. Prediction uses current data with the retained fit.
- CLI-v1 JSON returns `RecodeResult` with owned `data.dataset`: source path, total rows and ordered native schema, not rule/changed-cell/overflow fields. Label/panel fields remain null placeholders. Inspect values with `head`/`tail`.
- Unknown sources, invalid target counts/duplicates/existing targets, string ranges and native case-collisions fail without changing prior data/session state. Successful effects are not rolled back by later output failure (MIG-0008).
