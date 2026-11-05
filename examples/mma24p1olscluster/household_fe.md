# Chapter 24 household commune fixed effects

This bounded workflow covers Cameron–Trivedi (2005) §24.7/Table 24.4,
household commune-fixed-effects columns 7–8, from the pinned author program and
`vietnam_ex1.asc` described in the [pooled OLS workflow](README.md). It models
`LNEXP12M` on an intercept, `LNHHEXP AGE SEX HHSIZE FARM EDUC` and one absorbed
intercept per commune. It is unweighted, not complex-survey or causal inference;
RE/GEE, bootstraps, full survey design and the rest of the table remain required.
As in the source, categorical codes enter as **numeric regressors**, not factor
expansion. Commune effects do not guarantee within-commune exogeneity.

The source contains cross-sectional households, **not time series**. The
required `xtreg ... time(source_row)` uses source-row number solely as an exact,
unique within-commune key; it does not turn household order into time or justify
lags, trends, differencing or serial correlation. Source preparation preserves
all 5,999 publisher records; 993 missing outcomes are removed, leaving 5,006
households in 194 communes, including one singleton. A zero log outcome is
valid. `source_row` is an assigned source-order index, not a survey household ID.

## Run and interpret

Prepare `vietnam.csv` using the pinned source and instructions in [README](README.md),
then, from that directory (with `repo` set to the repository path):

```sh
"$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma24p1olscluster/household_fe.td" > result.jsonl
```

Use `--text` for readable output. Both modes export the same ordered
`household_fe_predictions.csv`. The script requires unique positive BIGINT
source-row and commune keys, finite predictors and at least two complete-case
communes; invalid keys, nonfinite values, an unidentified within design and
insufficient degrees of freedom fail. SQL guards can surface only `sql failed`.
The final retained model is the FE fit, which replaces any previous model.
Export refuses to replace an existing file. The multi-command script is not
transactional: table/model changes may survive a later failure; use an empty
work directory and remove old outputs deliberately.

Slopes are estimated from variation within communes, with classical homoskedastic
covariance and Student-t inference at `N−G−p=4,806` DF (`G=194`, `p=6`). The
common intercept is identified by requiring the **observation-weighted sum**
of commune effects to be zero; its SE and covariance with all slopes respect
that normalization. `fe_xb` excludes the normalized commune effect; `fe_xbu`
includes it; `fe_u` is the normalized fitted effect; `fe_residual` is the
conditional residual. The singleton's conditional residual is zero and it
contributes no within-slope variation, but remains part of sample and
normalization. A reported pooled-versus-FE F test uses `(G−1,N−G−p)` DF under
classical homoskedastic assumptions; exact finite-sample calibration additionally
requires normal errors. It is not a cluster-robust or survey-design test.
The separate joint `_cons`/`lnhhexp` test is a test of the normalized intercept
and that slope, not a test of all commune effects.

## Independent evidence

R 4.6.2 full-dummy QR, using an explicitly observation-weighted-zero-sum
parameterization rather than within formulas, checks 114 scalar quantities:
coefficients, the full 7×7 normalized-intercept/slope covariance, classical
t/F inference, RSS and R². All 5,999 source rows are bound to the pinned ASCII
and independently parsed R source; all 5,006 retained source/xb/xbu/u/residual
rows are checked against QR. Absolute tolerance is `1e-9`; nonzero p-values use
relative `1e-8`, with exact zero on underflow. Original text/JSON exports are
byte-identical. A separate unbalanced synthetic R fixture includes a singleton,
one missing outcome and a valid zero outcome. Tests cover duplicate/invalid or
floating-rounded keys, nonfinite values, rank and one-commune failure, source
preservation, output collision and retained FE post-estimation. Auditor forgeries
of source, coefficient, covariance, unit effect, fitted/residual and dropped
rows are rejected.

```sh
Rscript "$repo/examples/mma24p1olscluster/household_fe_reference.R" /path/to/vietnam_ex1.asc reference
python3 "$repo/examples/mma24p1olscluster/check_household_fe.py" . reference /path/to/vietnam_ex1.asc
```

The original DTA retains more precision than ASCII; the [pooled workflow](README.md)
documents the separate row-by-row comparison and SHA-pinned source/log. A
separate full-dummy R fit on the DTA matches all 14 printed FE coefficient/SE
facts at their displayed rounding precision. This is historical investigation,
not the strict ASCII oracle or a claim of binary identity:

```sh
Rscript "$repo/examples/mma24p1olscluster/dta_reference.R" /path/to/vietnam_ex1.dta dta.csv
Rscript "$repo/examples/mma24p1olscluster/household_fe_reference.R" dta.csv dta-reference --csv
python3 "$repo/examples/mma24p1olscluster/check_household_fe_author.py" /path/to/vietnam_ex1.dta /path/to/mma24p1olscluster.txt dta-reference
```

R and Python are development reference/preparation tools, not runtime
dependencies. Reference directories and DTA conversion refuse existing
targets. Publisher data and code are not redistributed.
