# Individual conditional-FE bootstrap synthetic reference

The shared source `../mma24_individual_conditional_fe/input.csv` is an
inspectable 149-record unbalanced, interleaved synthetic fixture. It contains
one wholly missing row, one positive singleton and one eight-person zero-total
commune. The original conditional FE estimation sample is 140 people in 18
positive-total communes; prediction output contains all 148 nonmissing records.

- `draw_indices.csv`: all 9,000 ordered zero-based indices from the independent
  Python xoshiro oracle, one row per replicate; 18 copies per replicate.
- `fits.csv`: 500 ten-slope conditional-score/information refits and copied Ns
  from independent base R 4.6.2; `summary.csv`: all 100 B−1 covariance cells,
  original estimates, means/bias/SE and type7 percentile95 bounds.
- `groups.csv`: retained communes in first-seen source-row order and sizes.
  `predictions.csv`: all 148 independently calculated original-model predictions,
  including flags and NULL means/residuals for the zero-total group.

No publisher data are checked in. R's score method weights each retained group's
contribution by its sampled multiplicity. A separate expanded raw-copy
`glm.fit` dummy-Poisson check verifies replicates 1,250,500, including repeated
IDs and a positive singleton. Native and independent R results are compared at
fixed absolute 1e-9; draws, sample membership and flags are exact.

From the repository root, set `repo=$PWD` and `work` to a fresh directory:

```sh
cp tests/fixtures/mma24_individual_conditional_fe/input.csv "$work/vietnam_individual.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma24p2poiscluster/individual_conditional_fe_bootstrap.td" > result.jsonl)
python3 examples/mma24p2poiscluster/check_individual_conditional_fe_bootstrap.py oracle "$work" "$work/vietnam_individual.csv" "$work/draws.csv" --synthetic
Rscript examples/mma24p2poiscluster/individual_conditional_fe_bootstrap_reference.R "$work/vietnam_individual.csv" "$work/draws.csv" "$work/reference" --csv
python3 examples/mma24p2poiscluster/check_individual_conditional_fe_bootstrap.py check "$work" "$work/reference" "$work/vietnam_individual.csv" --synthetic
Rscript examples/mma24p2poiscluster/individual_conditional_fe_bootstrap_glm_check.R "$work/vietnam_individual.csv" "$work/draws.csv" "$work/reference"
```

The focused executable test explicitly checks every fixture draw, fit,
covariance and prediction: `cargo test --locked --test
mma24_individual_conditional_fe_bootstrap_workflow`. It also covers text/JSON,
input-order invariance, invalid sources, full-width seed, output collision and
retained-fit/data ownership. R/Python are optional reference tools only.
