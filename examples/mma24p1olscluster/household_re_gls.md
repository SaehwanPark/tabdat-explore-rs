# Chapter 24 household commune RE-GLS — in progress

This workflow targets §24.7/Table 24.4 household random-effects GLS, with the
5,006 complete cases in 194 communes and six numerical predictors from the
[pooled workflow](README.md). It is unweighted and is not a causal, complex-
survey, GEE or ML analysis, or a complete Table 24.4 replication. The groups
are unbalanced and include a singleton. `source_row` is an artificial unique
within-commune key required by `xtreg`, **not time**; no lag, trend or serial
interpretation follows. Categorical variables enter as their original numeric
codes rather than factor expansions.

The script fits native unbalanced Swamy–Arora trace-corrected variance components
and unit-size-specific quasi-demeaning. `re_xb` and `re_residual` are **population**
linear predictions and residuals, not posterior commune effects or idiosyncratic
residuals. Covariance is quasi-demeaned RSS/(N−K) times the inverse transformed
design cross-product; coefficient inference is normal and the slope Wald uses
chi-square. Unit variance is constrained to nonnegative values; both signed raw
and truncated estimates are reported. The retained model is the RE fit; output
collisions preserve the owner but earlier model/table effects can persist because
the script is not transactional. BIGINT keys, finite fields, a full-rank design
and enough usable communes are required.

With `repo` set to this repository, prepare `vietnam.csv` from the SHA-pinned
publisher source as in [README](README.md), then execute in that work directory:

```sh
"$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma24p1olscluster/household_re_gls.td" > result.jsonl
Rscript "$repo/examples/mma24p1olscluster/household_re_gls_reference.R" /path/to/vietnam_ex1.asc reference
python3 "$repo/examples/mma24p1olscluster/check_household_re_gls.py" . reference /path/to/vietnam_ex1.asc
```

**Preliminary evidence:** independent R 4.6.2/plm 2.6.7, full-dummy within QR,
weighted-between trace calculation and quasi-QR agree on 108 fit/covariance,
normal/chi-square inference and variance-component fields, all 5,999 source
rows and all 5,006 population-prediction rows. Fixed absolute tolerance is
`1e-9`; nonzero p-values use relative `1e-8`, with exact zero on underflow.
Original text/JSON prediction exports are byte-identical. Two independent R/plm
synthetic cases test an unbalanced sample with a singleton: one has positive
unit variance and the other has negative raw variance truncated to zero. Focused
CLI tests pass, including invalid keys/rank and output collision; broader
regression tests, final audit and PR review remain.

## Historical difference: separate hypothesis

The publisher Stata-8 log gives sigma_u approximately `0.46739871` and LNHHEXP
slope `.6268899`. Native Swamy–Arora on the ASCII gives unit variance
`0.1869830977` and slope `0.6292205830`. An independent R construction on the
original DTA using **unweighted equal-commune between-regression MSE minus
sigma_e² * mean(1/T_g)** for the unit component (truncated at zero), followed
by per-group quasi-GLS, matches all 14 printed author coefficient/SE facts at
their displayed precision. This explains the observed difference **for this
sample**, not a verified general Stata rule or an implemented author-style
estimator. It is saved as `author_hypothesis.csv` by the R reference and is
separate from native retained state. No canonical tolerances were widened.

**Status:** draft PR; final source/report forgery tests, documentation/ledger
alignment, affected-crate tests, scoped lint and review have not yet completed.
