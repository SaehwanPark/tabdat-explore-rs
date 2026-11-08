# Independent two-slope, unbalanced, singleton bootstrap controls

Original input is the existing artificial120-row/40-group control at
`crates/tabdat-stats/tests/fixtures/normal_poisson/multivariate/input.csv`
(repo-relative). Its source mapping is200 candidate rows/80 omitted; there are
8 singleton groups. Each copied sample gives a fresh effect to every sampled
group occurrence. Existing x/z values are copied, not recomputed from copy IDs.

Native seed10001 first-seen group draws (independently verified shared RNG
stream) yield127 rows in copy1 and121 in copy2,40 effects each. The merged-ID
counterfactual uses identical copy1 observations/design but only23 effects.
Native/R lnvariance values are.530350602001616 versus.974796580554228; merging
is not a harmless key relabeling.

Reproduce each selected copy's source_row/id/year/y/x/z CSV using the original
rows and ordered draw indices, assigning sequential source_row and fresh
id=copy_position+1; for the counterfactual retain the original id. Then:

```sh
Rscript examples/mma23p1pannonlin/native_normal_re_multivariate_reference.R \
  COPY_CSV NEW_OUTPUT_DIR copied-csv
```

R4.6.2 starts its own full-X GLM/t0 and uses adaptive improper integration,
complete-data derivatives, BFGS/Newton, full observed PD information, and two
Richardson covariance refinements at unchanged1e-10. Maximum covariance
discrepancies2.917e-11/1.977e-12/1.669e-11 for distinct1/distinct2/merged1.
Each frozen CSV has57 model/score/full4x4 covariance/inference/prediction-summary
facts, not a native-coefficient-fed reference. The bootstrap test compares all
four joint triples, actual sample sizes, metadata/retained-source mapping and
empirical full4x4 covariance/bias/type7. Native merged fit also matches its R
point and differs materially from the independent-copy result. No per-replicate
Wald covariance is exported as bootstrap uncertainty.

The legacy R57-field output remains byte-identical; its CSV input path passes
existing scalar1e-9+1e-10*abs(ref)/positive-p relative1e-9 gates (max difference
1.024e-12, not claimed byte-identical). Bootstrap tests require no R/Python at
runtime. These are kernel controls, not proof of terminal/script integration.
