# Household equal-commune between-component RE GLS

This opt-in workflow advances §24.7/Table 24.4 household RE estimation, using
the same six numeric predictors and 5,006 complete-case households in 194
communes as the [pooled](README.md) and [trace-corrected RE](household_re_gls.md)
workflows. It is an unweighted model-based error-components analysis, not
complex-survey design or causal inference. `source_row` is a synthetic unique
cross-sectional key, **not time**. Numeric EDUC/FARM/SEX codes are not expanded
into factors.

## Explicit method, unchanged default

```text
xtreg lnexp12m lnhhexp age sex hhsize farm educ, re id(commune) time(source_row) vcmethod(equal_group_between)
```

The omitted method and explicit `vcmethod(swamy_arora)` retain the existing
trace-corrected default. The alternate name describes the formula; it is not a
general Stata mode. With N observations, G communes, p slopes and K=p+1:

```text
sigma_e² = within RSS / (N−G−p)
sigma_u²_raw = unweighted between RSS / (G−K) − sigma_e² * mean_g(1/T_g)
sigma_u² = max(0, sigma_u²_raw)
theta_g = 1 − sqrt(sigma_e² / (sigma_e² + T_g*sigma_u²))
```

The between regression includes an intercept and gives **each commune equal
weight**, unlike the trace-corrected default for unequal groups. Per-size
quasi-demeaning transforms y and all design columns, including the intercept.
Model covariance is transformed RSS/(N−K) times the inverse transformed design
cross-product; coefficient z/p/95% intervals use the normal reference and the
joint slope Wald uses chi-square(p). Here within DF=4,806 and transformed DF=
4,999. Signed raw and truncated components, rho and theta ranges are reported.
On a balanced sample the component formula reduces to the existing one.

The same raw-key audit, joint finite-case/rank/variance guards and population
prediction semantics apply as in [native RE GLS](household_re_gls.md). The
household script requires positive BIGINT keys and finite nonmissing predictors,
excludes only NULL outcomes (993 of 5,999 original rows), retains zero outcomes
and sorts by source key. The remaining groups have 1–39 observations including
one singleton. G>K, positive within DF/variance, and identified within/between
and transformed designs are required. QR and finite-transformation checks, not
an iterative optimizer, determine numerical success.

`vcmethod()` is invalid with FE/ML. This alternate method supports **model-based
covariance only**; robust/cluster requests are rejected rather than silently
ignored. Hausman, variance-component uncertainty, unit-effect predictions,
general restrictions and named RE reports are not supplied. No weights/GEE/ML,
complex-survey or causal interpretation is introduced.

`re_xb` is retained intercept + current X beta, and `re_residual` is current
y−xb, including the commune component. Neither is a posterior commune effect or
idiosyncratic residual. The RE fit remains retained after predictions, summaries
and scope reporting. A failed new runtime fit clears the model without changing
data; parse errors preserve it. An export collision preserves owner/source but
not earlier model/table effects: this multi-command script is **not transactional**.

## Run and reproduce

Build the binary and prepare SHA-pinned `vietnam.csv` using [README](README.md).
From an otherwise empty work directory with `repo` set to the repository:

```sh
"$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma24p1olscluster/household_equal_group_re.td" > result.jsonl
Rscript "$repo/examples/mma24p1olscluster/household_re_gls_reference.R" /path/to/vietnam_ex1.asc reference --equal-group-between
python3 "$repo/examples/mma24p1olscluster/check_household_re_gls.py" . reference /path/to/vietnam_ex1.asc --equal-group-between
```

`--text` produces readable reporting and byte-identical ordered predictions in
`household_equal_group_re_predictions.csv`. JSON retains the existing balanced
or unbalanced RE result type and labels `variance_component_method` as
`equal_group_between`. Both interfaces name the selected method; no default
workflow is relabelled. Reference directories must be new; export refuses to
clobber existing files. R/Python are development tools only, not Rust runtime
dependencies. Publisher data are not redistributed.

Independent R 4.6.2 full-dummy within QR, equal-commune between QR and quasi-QR
validate 108 component/theta, coefficient/full-covariance, normal95 and joint
Wald quantities, all 5,999 source rows and all 5,006 population-prediction rows.
Maximum absolute difference is `5.81e-11`, maximum nonzero-p relative difference
`2.87e-11`. Fixed tolerances remain absolute `1e-9`, p relative `1e-8`, exact zero
for underflow; they were not widened. Text/JSON export bytes match. Independently
regenerated 179-case/16-commune fixtures reuse the inputs under
`tests/fixtures/mma24_household_re/`, with alternate reference/prediction CSVs
under `tests/fixtures/mma24_household_equal_group_re/`. They include a singleton,
a missing outcome, a positive unit-variance interior and a negative raw
component truncated to zero. Kernel tests also verify balanced equivalence and
identification/zero-within-variance failures. Parser/runtime/CLI tests cover
invalid method combinations, retained state, rank/keys/nonfinite guards and
source/output-owner preservation. Original-data auditor challenges reject
altered source/prediction fields, missing rows, fit/covariance/component/theta
or inference metadata and unpinned raw inputs.

## Separate author-output check

The [trace-corrected method](household_re_gls.md) remains independently valid but
differs materially from the author. This explicit equal-commune construction on
the original ASCII gives unit variance `0.2184615698` and LNHHEXP slope
`0.6268898843`. A separate native run on the SHA-pinned DTA matches **all 14**
printed Stata-8 coefficient/SE facts at their displayed rounding precision.
The DTA run also matches all 108 direct R quantities and all source/prediction
rows (maximum absolute `7.41e-11`, maximum nonzero-p relative `3.64e-11`). This
is evidence for this illustration, not a verified claim about Stata generally
or exact printed book-table parity. To reproduce in another fresh directory:

```sh
Rscript "$repo/examples/mma24p1olscluster/dta_reference.R" /path/to/vietnam_ex1.dta vietnam.csv
"$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma24p1olscluster/household_equal_group_re.td" > result.jsonl
Rscript "$repo/examples/mma24p1olscluster/household_re_gls_reference.R" vietnam.csv dta-reference --csv --equal-group-between
python3 "$repo/examples/mma24p1olscluster/check_household_re_author.py" /path/to/vietnam_ex1.dta /path/to/mma24p1olscluster.txt dta-reference result.jsonl
```

**Status:** independently validated bounded native workflow, not complete
Table 24.4/chapter coverage. Printed author within/between/overall R²,
component-summary rounding and other fit-statistic outputs are not covered by
the 14-fact audit. Separate household OLS/FE/RE and individual pooled/FE/RE
bootstrap workflows, household Gaussian GEE and [household RE-ML](household_re_ml.md)
are validated with their own stated limits. Other GEE workflows, genuine survey
weights/strata and remaining illustrated outputs remain required.
