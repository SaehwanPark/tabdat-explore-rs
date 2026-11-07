# Native patents conditional-FE bootstrap control

Raw source is shared, not duplicated: `../mma23_conditional_fe/source.asc`.
Twelve firms/60 records, two all-zero firms/10 excluded rows leave ten positive
firms/50 estimation records. Resample those ten firms500 times, seed10001,
first-seen canonical source order; copies are distinct groups, failures abort.

R4.6.2 generated11 summary fields and all500 scalar refits in these two CSVs,
using unchanged `examples/mma23p1pannonlin/conditional_bootstrap_reference.R`.
The oracle solves independent conditional score roots, cross-checks three draws
with weighted dummy-variable Poisson GLMs, and computes B−1 variance/SD and
type7 percentile95. No native coefficient was used to generate reference values;
only independently checked native draw indices are supplied as the design.
Fixed absolute1e-9 gates apply to coefficients/summaries; membership/indices/
counts are exact. Original publisher data stay external.

Reproduce in fresh directories after building the native executable:

```sh
repo=$PWD
work=$(mktemp -d)
sh examples/mma23p1pannonlin/prepare.sh \
  tests/fixtures/mma23_conditional_fe/source.asc "$work/patents.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/mma23p1pannonlin/native_conditional_fe_bootstrap.td" > result.jsonl)
python3 examples/mma23p1pannonlin/check_native_conditional_fe_bootstrap.py oracle \
  "$work" tests/fixtures/mma23_conditional_fe/source.asc "$work/draws.csv" --control
Rscript examples/mma23p1pannonlin/conditional_bootstrap_reference.R \
  tests/fixtures/mma23_conditional_fe/source.asc "$work/draws.csv" "$work/reference"
python3 examples/mma23p1pannonlin/check_native_conditional_fe_bootstrap.py check \
  "$work" tests/fixtures/mma23_conditional_fe/source.asc "$work/reference" --control
```

The auditor independently checks RNG/draw identities, raw conditional replicate
scores, empirical covariance/quantiles and original prediction normalization.
The scalar covariance supplements all11 R fields; it is SD², not the original
model information inverse, a new p-value or a normal interval. This is neither
the separate SQL bootstrap stream nor historical Stata draw/SE/interval parity.
R/Python remain development-only dependencies.
