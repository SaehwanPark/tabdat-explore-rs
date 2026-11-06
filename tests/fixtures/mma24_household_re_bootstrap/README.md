# Household RE bootstrap controls

These small synthetic CSVs are test inputs and **independent R QR outputs**, not
publisher data. The `boundary` and `mixed` inputs share the unbalanced FE
fixture's predictors and group sizes (48 groups, 169 complete rows, including
singletons and one missing outcome). The mixed outcome adds group-specific
variation; among its 500 refits, 28 unit-variance estimates truncate to zero
and 472 are positive. All 500 boundary estimates truncate to zero. The
`conditioned` reference uses the existing FE `conditioned_input.csv`; its
correlated columns exercise an accepted design. The balanced input has 48
four-row groups and was generated with R 4.6.2, seed 7575, six independent
normal columns, one normal group effect and normal row noise with SD 0.5. Its
response is `1 + X %*% c(.3,-.2,.4,.15,-.25,.1) + group_effect + noise`.
The stored CSVs define the exact inputs for reproduction.

For each input, run `household_re_bootstrap.td` in a fresh directory, then:

```sh
Rscript "$repo/examples/mma24p1olscluster/household_re_bootstrap_reference.R" \
  vietnam.csv household_re_bootstrap_draws.csv reference --csv
python3 "$repo/examples/mma24p1olscluster/check_household_re_bootstrap.py" \
  . reference vietnam.csv --synthetic
```

The R program expands the drawn raw observations with distinct copied IDs,
re-estimates within and between variances and performs actual quasi-transformed
QR refits. The auditor independently checks every ordered draw with the Python
RNG oracle, then checks the R quantities and predictions; R/Python are not
runtime dependencies. `*_fits.csv`, `*_summary.csv`, `*_baseline.csv`,
`*_groups.csv`, and `*_predictions.csv` are the corresponding R outputs.
`draws_head.csv` is the first three replications of the 48-group RNG stream,
checked against the separately implemented Python oracle.
