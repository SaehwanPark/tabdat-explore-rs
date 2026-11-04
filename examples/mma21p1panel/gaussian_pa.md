# Chapter21 Gaussian population-averaged GEE

## Target and supported workflow

C21-P1-PA: author `mma21p1panfeandre.do` lines307–308, Table21.2,
§21.3 pp.709–714. [Inventory/source hashes](inventory.md) distinguish the
program header's Table21.1 typo from its executable Table21.2 labels.
Original MOM.dat SHA256:
`2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77`.
All5,320 rows/532 people/ten years1979–1988 enter unweighted marginal
`lnhr ~ 1 + lnwg`; no missing-case deletion. This is Gaussian identity-link
exchangeable GEE, **not** a positive-variance random-intercept fit relabelled PA.
Admissible negative correlation is supported.

[gaussian_pa.td](gaussian_pa.td) composes existing native SQL, pooled OLS,
prediction, public named-table activation/join and CSV/report interfaces.
No kernel/parser/runtime/schema/dependency changes or production R/Python
requirement. Native retained state remains **pooled OLS**, not GEE. GEE means
and residuals are explicit SQL columns; a later generic `predict` uses pooled
OLS. There is no new owned GEE estimator, store/margins or general predict API.

## Exact conventions and convergence

For person i, T=10, residual vector r_i, and centered within/person wage
moments Wxx/Wxy and Bxx/Bxy (B is a sum over person means):

- phi=RSS/N, rho=(sum_i(sum_t r_it)^2-RSS)/(9*RSS).
- Sw=sum within-person squared residuals; Sb=10*sum squared person-mean
  residuals. The working covariance has within eigenvariance e=Sw/(9G)
  and between eigenvariance v=Sb/G; phi=(9e+v)/10 and rho=(v-e)/(v+9e).
- Starting at native pooled OLS, update slope to
  `(Wxy+10*(e/v)*Bxy)/(Wxx+10*(e/v)*Bxx)`; intercept=mean(y)-slope*mean(x).
  Recompute all moments after every step and at the accepted final point.
- Convergence requires abs(slope_new-slope)/(1+abs(slope))<=1e-12 before
  the100-update exhaustion boundary. The final inverse-bread-scaled score
  must be <=1e-10*(1+abs(coefficient)) for both population coefficients.
  Iteration counts are algorithm diagnostics, not historical Stata parity.

Strict admission: 3–532 complete ten-year people, exact finite unique id/year
and source_row keys (id1..532, year1979..1988, source_row1..5320), finite model
values of absolute magnitude<=20, Wxx and Bxx>1e-12, final coefficients<=100
in magnitude. Require RSS>1e-12 and both Sw and Sb greater than
128*machine_epsilon*(Sw+Sb). Reject unresolved/singular working correlation,
unidentified designs, unrepresentable covariance or exhausted convergence;
never clip rho, delete cases or silently replace the estimator. These are
conservative bounded numerical guards, not universal statistical admission rules.
Ancillary original fields are preserved; the author/reference/control audits
use complete finite raw source fields.

R `gee`4.13.30 with explicit pooled-OLS start, free scale,
`v4.4compat=TRUE` and tol1e-13 independently implements the legacy Pearson
moment/RSS-N convention. On original MOM, modern-default `v4.4compat=FALSE`
changes scale to RSS/(N-2) and correlation corrections, giving a slightly
different slope0.1195382164. Do not silently substitute that convention.

## Covariance and interpretation

Both complete symmetric2x2 matrices use population parameter order
(intercept, lnwg). Model-working covariance is inverse sum_i X_i'V^-1 X_i,
where V=phi*((1-rho)I+rho*11'). In centered coordinates it is diagonal:
intercept variance v/N, slope variance1/D, D=Wxx/e+10*Bxx/v; transform back
to the population intercept. It assumes the specified working second moments.

Separately labelled `uncorrected_unit_sandwich` uses person scores
U_i=X_i'V^-1 r_i, influence h_i=H^-1 U_i and covariance sum_i h_i h_i'.
**No** G/(G-1), N-K or CR1 multiplier is applied; this matches legacy R gee's
robust covariance. Independent people and a correct marginal mean are needed;
the sandwich permits misspecified within-person covariance, not cross-person
dependence or mean confounding. All intervals are asymptotic normal95, not t.
The coefficient report includes z but no p-values: no trusted normal-tail SQL
primitive is installed. No likelihood/LR or component inference is claimed.

Original intercept/slope are7.345479144924085/0.11954740235725124,
phi0.08055113024536754 and rho0.32650965848038677. Working slope SE is
0.01365069968 versus uncorrected person-sandwich SE0.0514941. This is a marginal
log-hours/log-wage association, not a causal elasticity or fixed-effects
adjustment. Differing SEs are not a formal covariance-specification test.
Small-control intervals validate arithmetic, not nominal few-cluster coverage.

## Reproduction and outputs

From repository root, build the native binary and follow
[shared preparation](README.md#reproduce) to create fresh MOM.csv in a
working directory. Run from that directory (paths adjusted to the checkout):

```sh
/path/to/tabdat-explore-rs --json -f /checkout/examples/mma21p1panel/gaussian_pa.td > output.jsonl
# In another fresh directory with the same MOM.csv:
/path/to/tabdat-explore-rs --text -f /checkout/examples/mma21p1panel/gaussian_pa.td > output.txt
```

Default batch mode is quiet; **--text is required** for readable batch results.
The four ordered no-clobber exports are:

1. `panel_gaussian_pa_fit.csv`: sample, conventions, convergence, eigenvariances,
   full working/sandwich covariance and score certificate.
2. `panel_gaussian_pa_coefficients.csv`: four working/unit-raw coefficient rows,
   SE/z/normal95. The same table is readable and typed CLI-v1 `TableResult`.
3. `panel_gaussian_pa_predictions.csv`: original nine source/provenance fields,
   distinct pooled-OLS and GEE means/residuals, ordered by original source_row.
4. `panel_gaussian_pa_unit_scores.csv`: one ordered row/person, raw intercept/
   centered-slope scores and population-parameter influences.

These exports are **not a whole-script transaction**. A later collision leaves
its existing target unchanged, retains earlier exports, and stops later commands.
Named-relation publication/activation retains pooled OLS; the final active table
is gaussian_pa_predictions. Do not infer complete session/file rollback.

Development-only independent reference/audit:

```sh
Rscript /checkout/examples/mma21p1panel/gaussian_pa_reference.R MOM.dat fresh_reference
python3 /checkout/examples/mma21p1panel/check_gaussian_pa.py . fresh_reference MOM.dat
# Controls, each in a fresh directory; reference runs as above:
python3 /checkout/examples/mma21p1panel/gaussian_pa_synthetic.py positive fresh_positive
python3 /checkout/examples/mma21p1panel/gaussian_pa_synthetic.py negative fresh_negative
# Add --synthetic to their checker command; author-data hashing is then disabled.
```

R4.6.2/gee4.13.30 is a development reference only. Dense per-person10x10
covariance inversions independently audit the package's working/robust matrices,
scores and influences. A separate dense GEE iteration preflights singular cases
before legacy Cgee (which can stall there). R numeric CSVs use17 significant
digits. Frozen positive/negative controls in
[tests/fixtures/mma21_gaussian_pa](../../tests/fixtures/mma21_gaussian_pa) contain
only generated data, not redistributed MOM.

## Validation and limits

Original native JSON/text exports are byte-identical. Independent R/Python
checks all18 numerical fit fields, four inference rows,532 unit score/influence
rows,5,320 prediction rows and42,560 exact original raw fields. Twelve author-log
coefficient/SE/z/normal95/scale/Wald facts pass half-unit-of-last-printed-digit
bounds, distinct from the fixed absolute1e-9 native/R tolerance. Maximum original
native/R discrepancies are1.03e-12 across fit fields,2.98e-11 across inference
rows,1.13e-12 across predictions and5.81e-11 across unit-score fields. Seven
forgeries of coefficients, rho, variance convention, z, mean prediction, raw
source and influence are rejected. No tolerance widening is used.
Two32-person controls add640 prediction rows/64 unit rows and all covariances;
rho is positive0.56675 and negative−0.04574. Source order interleaves people and
reverses time. Retained original pooled-OLS predictions are independently checked
on all5,320 original rows and both controls after GEE composition.

Four [actual-script tests](../../tests/mma21_gaussian_pa_workflow.rs) cover frozen
full inference/scores/predictions, typed/readable tables, state and JSON/text
parity, missing/duplicate/incomplete/empty panels, singular correlation/within
rank, forced iteration-budget exhaustion, and every output-collision stage.
For the exhaustion test only, the cap is injected as zero; this is a failure-path
check, not evidence of a naturally nonconvergent original sample.
Adjacent Poisson GEE regressions, scoped Clippy, formatting, Python syntax and
local links are also checked. No whole-workspace/other-platform/release claim.

No weights/unbalanced panels, arbitrary formulas, generic GEE API, retained GEE
state, prediction uncertainty, p-values, likelihood, finite/few-cluster/causal
claim or literal historical iteration/output replay. This bounded source PA
coefficient/covariance/normal95 target does not close complete first-program
reporting, classical analytic FE normalized-level-intercept uncertainty, historical
bootstrap parity, companion-program inventory or full chapter/book coverage.
The separate [normalized FE level-intercept bootstrap](fe_level_bootstrap.md)
now validates a bounded pairs-bootstrap target, not the centered auxiliary-OLS
intercept. Classical analytic common-intercept covariance remains a next candidate.
