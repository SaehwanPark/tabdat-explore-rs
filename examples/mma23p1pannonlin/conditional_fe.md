# Chapter 23 patents: conditional fixed-effects Poisson

This bounded **composed SQL workflow** reproduces the model-based Poisson-FE
column of Cameron and Trivedi (2005), §23.3, Table 23.1 (p.794), from the
March 2005 [author program](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.do)
and [output](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.txt).
It does **not** install a general native `xtpoisson` command. The script guards
and selects the panel, solves one conditional score equation, reports the
coefficient/inference and conditional likelihood, and exports the original rows
with fit status, model metadata and in-sample means/residuals. Text and CLI-v1
JSON use the same script; no restorable estimation state is created.

## Sample and conditional model

Use the [pooled workflow's source preparation](pooled_poisson.md): the pinned
`patr7079.asc` SHA-256 is
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`.
Firm `id` is original record position, not CUSIP; ten missing ARDSSIC metadata
values remain missing. All 346 firms' five observations in 1975–79 are
candidates (1,730 rows). Condition on each firm's observed patent total; the
22 zero-total firms/110 rows supply no slope information and are excluded,
leaving **324 firms and 1,620 rows**. The script retains all candidate rows in
its export, sorted by `id,year`, marks `included_in_fe`, and emits NULL means
and residuals for the zero-total firms. No year dummies, extra covariates,
weights or offset are used.

For positive-total firms, the model assumes independent conditional Poisson
counts with `mu_it=exp(a_i+b*logr_it)`. Conditional on each total `Y_i`,
`p_it(b)=exp(b*x_it)/sum_t exp(b*x_it)` and
`U(b)=sum_i sum_t (y_it−Y_i*p_it)*x_it`. Unit intercepts are profiled out;
the reported conditional log-likelihood includes the multinomial factorial
terms. Information is `sum_i Y_i*sum_t p_it*(x_it−sum_s p_is*x_is)^2`;
variance is its inverse. The fitted in-sample mean is `Y_i*p_it`, so fitted
means reproduce each included firm's observed total. These are conditional
within-sample fits, not unconditional or out-of-sample forecasts. The nuisance
unit intercepts are neither reported nor assigned uncertainty.

| Original source, float64 | Value |
| --- | ---: |
| LOGR coefficient | −0.0377642353576 |
| Model-based SE | 0.0325517613504 |
| Normal z / two-sided p | −1.1601287854 / 0.2459963760 |
| Conditional log-likelihood | −3659.5925591281 |
| Information | 943.7370695107 |

The author log rounds these to coefficient `−.0377642`, SE `.0325518`,
p `.246`, and log-likelihood `−3659.5926`. Model-based inference requires the
conditional Poisson distribution; it is **not firm-cluster robust** and can be
invalid under remaining within-firm dependence. It is not a causal R&D effect,
unit-intercept uncertainty, a general FE/RE selection procedure, or the author's
separate whole-firm bootstrap.

## Numerical and failure boundaries

`conditional_fe.td` permits exact integer firm ids 1–346, distinct keys, exactly
one observation in each of 1975–79, nonnegative integer counts ≤1,000 and
finite `abs(logr)≤20`. The within-firm centered predictor must also have
absolute value ≤20. Centering leaves the conditional likelihood unchanged and,
with the slope bracket [−10,10], bounds exponent arguments by ±200. At least two
positive-total firms and a **numerically distinct interior** score sign change
are required. Each endpoint score must exceed `128*machine_epsilon*(1+scale)`
in its required direction, where `scale` is the sum of absolute observed and
fitted score components; cancellation near a bracket boundary is rejected.
The scalar monotone score is bisected for 80 iterations; finite positive
information, finite conditional likelihood and a scale-relative converged score
are checked before publishing the derived relation. Separation, zero within
variation, no bracket, excessive magnitudes, all-zero outcomes and invalid keys
fail rather than yielding a boundary estimate, ridge or dropped extra groups.

The 95% interval uses the standard normal quantile. The two-sided normal tail
is evaluated by an 80-step alternating error-function series only for
`|z|≤3`, with `p_reference=asymptotic_standard_normal`; for larger |z| the
p-value is NULL and `p_reference=unavailable_abs_z_exceeds_3`, while z and the
interval remain available. There is no claim about numerical p-values outside
that domain. `sql` currently surfaces some guard failures as `sql failed` at the
script location; the original input file and any pre-existing export remain
unchanged. Use a fresh output directory; the export refuses to overwrite.

## Independent reproduction

```sh
repo=/absolute/path/to/tabdat-explore-rs
work=/absolute/path/to/empty/work-directory
curl -fsSL https://cameron.econ.ucdavis.edu/mmabook/patr7079.asc -o "$work/patr7079.asc"
(cd "$repo" && cargo build --locked --bin tabdat-explore-rs)
sh "$repo/examples/mma23p1pannonlin/prepare.sh" "$work/patr7079.asc" "$work/patents.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma23p1pannonlin/conditional_fe.td" > result.jsonl)
Rscript "$repo/examples/mma23p1pannonlin/conditional_fe_reference.R" "$work/patr7079.asc" "$work/r-reference"
python3 "$repo/examples/mma23p1pannonlin/check_conditional_fe.py" "$work" "$work/r-reference" "$work/patr7079.asc"
```

The R 4.6.1 reference independently reshapes raw tokens and fits a full
`glm(pat ~ logr + factor(id), family=poisson)` on positive-total firms. It also
independently solves the scalar conditional score and verifies information and
fitted means against the dummy-variable fit. The checker pins the source hash,
compares 14 R quantities and checks all **48,440** exported source, inclusion,
prediction and report cells across 1,730 rows, including firm-total identities.
Numerical tolerance is `1e-9 + 1e-10*abs(reference)`; source fields and
membership are exact, score is required within 1e-8, and selected historical
values are checked at printed precision. Original text/JSON CSVs matched byte
for byte. A 12-firm synthetic fixture validates the same R comparisons, omitted
firms, row ordering and negative cases; separate positive and negative fitted
slopes, including roots around ±9.878 near the bracket limits, validate the
bounded solver and p-value withholding against R. The actual
SQL tail expression was checked against 61 independent R `pnorm` values over [−3,3] within 1e-13
absolute. R/Python are development-only; author data/program are not bundled.

The random-effects Poisson, GEE, NLS, whole-firm bootstrap, Figure23.1 and
remaining Table23.1 outputs remain separate required work. Neither this bounded
composition nor the [pooled Poisson column](pooled_poisson.md) completes the
nonlinear-panel chapter, graduate-method target or book inventory.
