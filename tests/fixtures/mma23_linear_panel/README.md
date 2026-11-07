# Chapter23 linear-panel reference

Control raw source is `../mma23_gamma_re_bootstrap/source.asc`:40 firms/200
1975–79 observations, including8 all-zero firms and nonmodel-missing ARDSSIC.
LPAT=ln(0.5) at zero, otherwise ln(PAT); linear FE retains all200 rows/40 firms,
not conditional-count FE's positive-total subset. Classical FE residual df159.

`reference.csv` keeps the original91 field/header bytes unchanged and appends15
R4.6.2/plm2.6.7 normalized-intercept/full-covariance fields,106 total. Existing
`predictions.csv` is unchanged. Fresh independent R still reproduces all106 at
fixed scalar1e-9+1e-10*abs(ref), nonzero-p relative1e-9; no tolerance widening.

Generate in a fresh directory:

```sh
Rscript examples/mma23p1pannonlin/linear_panel_reference.R \
  tests/fixtures/mma23_gamma_re_bootstrap/source.asc /tmp/fresh-linear-reference
```

The new oracle fits `lm(lpat ~ logr + factor(id))`, requiring full rank/df and
agreement with the independent plm within fit/RSS/predictions. The dummy baseline
intercept is not the normalized common intercept. A two-row linear map adds
observation-weighted dummy effects and preserves the slope; transform its full
OLS covariance for the complete2x2 normalized matrix. Confidence intervals and
nonzero p-values use Student-t159, not normal/G−1; original1730/346 has df1383.

`forged_normalization.csv` is **deliberately wrong**, not an estimation oracle:
unchanged full-dummy point/df but normalized covariance multiplied by1.25, with
R-consistent SE/t/p/95CI. It exercises shared native/R corruption rejection via
independent raw-design within/mean variance identities. Reproduce after sourcing
the reference generator with its normal two raw/output arguments:

```r
fields <- normalization_fields(normalized, normalized_cov * 1.25, df)
write.csv(data.frame(model='fe', path=names(fields), value=unname(fields)),
  '/tmp/forged_normalization.csv', row.names=FALSE, quote=FALSE)
```

`student_t_tails.csv` freezes30 independent base-R two-sided t tails, df7/159/
1383 and ten statistics including near zero, both actual common-intercept t
values and40. It checks the auditor-only incomplete-beta implementation at
unchanged relative1e-9, including complement cancellation near t=0:

```r
d <- expand.grid(df=c(7,159,1383),
  statistic=c(0,1e-6,.1,1,1.96,5,20,23.91854386526995,25.65713523504,40))
d$p_value <- 2*pt(-abs(d$statistic), d$df)
write.csv(d, '/tmp/student_t_tails.csv', row.names=FALSE, quote=FALSE)
```

No native outputs or tolerances were used to create the new reference blocks.
R/Python are development tools only; no production dependency or whole-table
historical/RNG/optimizer parity is implied by this fixture.
