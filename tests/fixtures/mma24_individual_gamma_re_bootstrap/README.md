# Individual shared-gamma RE bootstrap synthetic reference

The shared input `../mma24_individual_gamma_re/input.csv` is an inspectable
279-record unbalanced, interleaved synthetic fixture. One wholly missing record
is excluded; all 278 complete people and 48 groups remain, including four
zero-total groups and two singletons.

- `draw_indices.csv`: all 24,000 ordered zero-based indices from the independent
  Python xoshiro oracle, one row per replicate; 48 copies per replicate.
- `fits.csv`: 500 independent base-R 4.6.2 shared-gamma integrated-likelihood
  refits of intercept, ten slopes and ln(alpha), plus copied Ns.
- `summary.csv`: all 144 B−1 covariance cells, original points, means/bias/SE
  and type7 percentile95 bounds. `groups.csv`: complete-case groups in
  first-seen source-row order and sizes. `predictions.csv`: all 278 independent
  original-model marginal means/indices/residuals and inclusion/total fields.

The R reference weights each original group's likelihood by draw multiplicity.
A separate independently optimized raw-row expansion assigns distinct IDs to
copies and agrees for replicates1,250,500, including zero groups, singletons and
repeated IDs. A merged-source-ID counterfactual differs substantially; it is
not a historical software replay. Native and reference results use fixed
absolute 1e-9 tolerance; indices, membership and flags are exact. No publisher
data are checked in.

From the repository root, set `repo=$PWD` and `work` to a fresh directory:

```sh
cp tests/fixtures/mma24_individual_gamma_re/input.csv "$work/vietnam_individual.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma24p2poiscluster/individual_gamma_re_bootstrap.td" > result.jsonl)
python3 examples/mma24p2poiscluster/check_individual_gamma_re_bootstrap.py oracle "$work" "$work/vietnam_individual.csv" "$work/draws.csv" --synthetic
Rscript examples/mma24p2poiscluster/individual_gamma_re_bootstrap_reference.R "$work/vietnam_individual.csv" "$work/draws.csv" "$work/reference" --csv
python3 examples/mma24p2poiscluster/check_individual_gamma_re_bootstrap.py check "$work" "$work/reference" "$work/vietnam_individual.csv" --synthetic
Rscript examples/mma24p2poiscluster/individual_gamma_re_bootstrap_raw_check.R "$work/vietnam_individual.csv" "$work/draws.csv" "$work/reference"
```

`cargo test --locked --test mma24_individual_gamma_re_bootstrap_workflow`
explicitly checks every fixture draw, joint fit, covariance and prediction,
text/JSON, source order, invalid inputs, full-width seed and export/state
ownership. R/Python are optional development references only.
