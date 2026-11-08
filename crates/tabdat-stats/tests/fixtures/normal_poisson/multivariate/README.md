# Two-slope unbalanced normal-Poisson integration reference

Derived deterministically from the existing synthetic40-firm/200-record
`tests/fixtures/mma23_gamma_re_bootstrap/source.asc`, not publisher data.
Retain first `1+(id mod5)` years per firm:120 observations/40 groups/eight
singletons, with sizes1..5; zero-total groups remain. `source_row` is the
zero-based position in the original200 records, so the supplied estimation
sample explicitly records80 deliberate cohort exclusions (not missing values
or optimizer exclusions).

`x` is source LOGR; `z=sin(.37*id)+(time-3)^2/4+.2*x²`, time1..5. Intercept,
x,z,ln(sigma²) are jointly fit. Input and prediction design columns are written
with17 significant digits to preserve the exact R-fitted binary64 design;
ordinary write.csv numeric output is insufficient for a bit-exact round trip.

Generate in a nonexistent directory using base R4.6.2:

```sh
Rscript examples/mma23p1pannonlin/native_normal_re_multivariate_reference.R \
  tests/fixtures/mma23_gamma_re_bootstrap/source.asc /tmp/normal-multivariate-reference
```

The independent oracle uses direct adaptive improper integration, complete-
data derivative expectations and centered score covariance, not native GH
rules or its Stein/within-design decomposition. BFGS starts from an independent
Poisson GLM, followed by full Newton polishing. Two Richardson refinements of
numerically differentiated scores independently check the full4x4 covariance
against inverse integrated information (maximum difference5.45e-12; fixed
absolute1e-10 oracle check).

`reference.csv`:57 independent metadata, point, full joint covariance, score,
SE/z/p/normal95/log-Wald sigma, pooled boundary LR, joint two-slope Wald and
population-mean summary fields. `predictions.csv`: all120 exact source keys/
design/counts and population means/residuals. `input.csv` supplies no native
results to R. Native fixed gates1e-9+1e-10*abs(reference), nonzero p relative
1e-9; no widened tolerances. A reversed-row/opposite-unit rescaling replay
checks sample mapping, parameter/covariance transformations, Wald and every
prediction under RMS preprocessing. These are native statistics-API controls,
not parser/execution/state or whole-source/book replication evidence.
