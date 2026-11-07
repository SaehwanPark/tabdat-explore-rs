# Household random-effect BP LM controls

Generated with R4.6.2/plm2.6.7 by
`examples/mma24p1olscluster/household_bp_lm_synthetic.R` and the separate
`household_bp_lm_reference.R`. No publisher data is included.

- `balanced`: 128 rows,16 groups of8; positive group score.
- `unbalanced`:151 source/150 selected rows,16 groups, one singleton; source
  row4 has missing outcome but finite predictors/key. Group size is computed
  after selection, not from all source rows.
- `negative`:128 rows,16 groups of8; noise projected off X and commune dummies
  before source rounding. Group residual sums are nearly zero, signed score
  negative and squared-score chi-square p about0.002497. This must NOT be
  presented as evidence for positive random-effect variance or clipped to a
  one-sided boundary test with p1.

Predictors are exact multiples of1/16 and outcomes of1/1024, emitted as short
exact dyadic decimals. An initial17-digit trigonometric-input probe revealed
R's `as.numeric`/`scan` parsing `0.96377089636589053` as
`0.96377089636589042`, while Rust/DuckDB/Python correctly rounded to the adjacent
binary64 value. Replacing only the synthetic inputs with inspectable exact
values avoids a shared-sample ambiguity; source exactness and numerical
comparison tolerances were not relaxed. Original ASCII and DTA contracts are
unchanged, and precise DTA conversion bypasses reparsing decimal tokens in R.

Each case has input, all selected prediction rows,13 residual/LM quantities and
56 pooled-coefficient/full-covariance quantities. Independent base-R QR and
plm `plmtest(type="bp",effect="individual")`/Honda signed score must agree.

Regenerate into a new directory (existing outputs are refused), then compare
its files with the frozen fixtures:

```sh
Rscript examples/mma24p1olscluster/household_bp_lm_synthetic.R /tmp/new-bp-fixtures "$PWD/examples/mma24p1olscluster/household_bp_lm_reference.R"
```

The generator's `case/input.csv` maps to `case_input.csv`; its
`case/reference/{diagnostic,fit,predictions}.csv` map to the corresponding
`case_kind.csv` files. Rust tests run the actual script in JSON/text, verify
these references and byte-identical exports, and cover source/type/design,
negative-score and retained-state/no-clobber failures. Optional Python checker
regressions additionally reject forged numerical, prediction, fit and reference
labels; Python/R are not runtime dependencies.
