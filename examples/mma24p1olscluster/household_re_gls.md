# Chapter 24 household commune RE-GLS

This bounded workflow advances §24.7/Table 24.4 household random-effects GLS,
using the same 5,006 complete cases in 194 communes and six numerical predictors
as the [pooled workflow](README.md). It is unweighted, not a causal, complex-
survey, GEE or ML analysis, and **not a reproduction of the author's RE columns**.
The groups have 1–39 observations, including one singleton. `source_row` is an
artificial unique within-commune key required by `xtreg`, **not time**; no lag,
trend or serial interpretation follows. EDUC/FARM enter as original numeric
codes, not factor expansions; SEX retains its original 1/2 coding.

## Sample, estimation and reporting

The script requires unique positive BIGINT source keys, positive BIGINT commune
keys and finite nonmissing predictors. A NULL outcome is excluded; a nonmissing
nonfinite outcome is invalid. Decimal keys are rejected rather than rounded.
The publisher's 5,999 rows lose 993 missing outcomes; log-outcome zero is retained.
The remaining rows are sorted by source key. The fit itself drops no further rows.
Smaller valid numeric CSVs are supported, subject to full-rank within and between
designs, positive within residual variance/DF and G>K usable communes (K=7).
There is no iterative optimizer or convergence criterion: QR/rank, finite variance
and transformation checks can reject an unidentified or numerically unusable fit.
SQL guard failures can report only `sql failed`.

Native unbalanced Swamy–Arora uses within RSS/(N−G−6), with DF=4,806 here,
for sigma_e². Let Z contain commune means of the intercept and predictors,
T=diag(T_g), and r be residuals from between regression weighted by T_g. The
raw unit component is

```text
sigma_u²_raw = [r'Tr − (G−K)*sigma_e²] / [N − trace((Z'TZ)^−1 Z'T²Z)]
sigma_u² = max(0, sigma_u²_raw)
theta_g = 1 − sqrt(sigma_e² / (sigma_e² + T_g*sigma_u²))
```

Both signed raw and truncated estimates are reported, together with the theta
range and rho. Per-commune quasi-demeaning transforms y and the entire design,
including the intercept. Coefficient covariance is transformed RSS/(N−K), with
DF=4,999, times the inverse transformed design cross-product. Coefficients use
normal z tests/95% intervals; the joint slope Wald uses chi-square with 6 DF,
not a t/F correction. These are model-based RE assumptions, not cluster-robust
or survey-design uncertainty.

`re_xb` and `re_residual` are **population** linear predictions and y−xb,
not posterior commune effects or idiosyncratic residuals. The ordered export
contains all complete-case source fields and these two prediction columns.
The retained model is the RE fit, including after prediction, summary and scope
reporting. An export collision preserves its owner and the source file, but the
RE model and prepared table already exist: this script is **not transactional**.
Use an empty work directory; deliberately remove an old export before rerunning.

## Reproduction and independent evidence

Build the binary first. With `repo` set to this repository, prepare `vietnam.csv`
from the SHA-pinned publisher source as in [README](README.md), then run in that
work directory:

```sh
"$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma24p1olscluster/household_re_gls.td" > result.jsonl
Rscript "$repo/examples/mma24p1olscluster/household_re_gls_reference.R" /path/to/vietnam_ex1.asc reference
python3 "$repo/examples/mma24p1olscluster/check_household_re_gls.py" . reference /path/to/vietnam_ex1.asc
```

`--text` gives readable reporting and the same CSV bytes. Reference directories
must not already exist. R/plm and Python are development reference/audit tools,
not production dependencies; publisher data are not redistributed.

Independent R 4.6.2/plm 2.6.7, full-dummy within QR, weighted-between trace
calculation and quasi-QR agree on 108 fit/full-covariance, normal/chi-square
inference and variance-component fields. Maximum absolute difference is
`5.81e-11`; maximum nonzero-p relative difference is `2.85e-11`. Fixed absolute
tolerance is `1e-9`; nonzero p-values use relative `1e-8`, with exact zero on
underflow. Every field of all 5,999 source rows is tied to the pinned ASCII and
independently parsed R source, and all 5,006 population-prediction rows agree.
Original text/JSON exports are byte-identical. Two committed synthetic R/plm
fixtures each have 179 complete cases/16 unbalanced communes, a singleton and
one missing outcome: one has positive unit variance, the other a negative raw
component truncated to zero. Integration tests cover both interfaces, invalid
and rounded-decimal keys, nonfinite values, rank failure, insufficient groups,
source integrity, retained RE state and export-owner collision. Original-data
auditor challenges reject altered source/prediction cells, dropped rows,
coefficients, covariance, tiny p-values rounded to zero, components, theta,
inference labels, parameter order and an unpinned source.

## Historical difference: separate hypothesis, not author parity

The publisher Stata-8 log gives sigma_u approximately `0.46739871` and LNHHEXP
slope `.6268899`. Native Swamy–Arora on the ASCII gives unit variance
`0.1869830977` and slope `0.6292205830`. This discrepancy is not merely the
DTA/ASCII precision difference discussed in [README](README.md).

An independent R construction on the original DTA using **unweighted equal-
commune between-regression MSE minus sigma_e² * mean(1/T_g)** for the unit
component (truncated at zero), followed by per-group quasi-GLS, matches all 14
printed author coefficient/SE facts at their displayed rounding precision.
This explains the observed difference **for this sample**, not a verified general
Stata rule. It is now executable as the separate opt-in
[equal-group-between method](household_equal_group_re.md), which validates
all 14 native DTA coefficient/SE facts without changing this workflow's default.
`author_hypothesis.csv` remains separate reference output and never replaces
this script's trace-corrected retained state. No canonical
tolerances were widened. To reproduce separately, using the pinned DTA/log from
README:

```sh
Rscript "$repo/examples/mma24p1olscluster/dta_reference.R" /path/to/vietnam_ex1.dta dta.csv
Rscript "$repo/examples/mma24p1olscluster/household_re_gls_reference.R" dta.csv dta-reference --csv
python3 "$repo/examples/mma24p1olscluster/check_household_re_author.py" /path/to/vietnam_ex1.dta /path/to/mma24p1olscluster.txt dta-reference
```

**Status:** independently validated native RE-GLS workflow, partial Table 24.4
method coverage only. Household RE outputs beyond the separately validated
equal-group coefficient/SE portion, GEE, bootstraps, survey weights/strata,
individual FE/RE and remaining tables remain required.
