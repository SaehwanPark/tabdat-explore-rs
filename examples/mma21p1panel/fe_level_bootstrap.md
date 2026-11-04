# Chapter21 FE normalized-level-intercept bootstrap

## Target and workflow

C21-P1-FE-BOOT: author first program lines211–213/Table21.2, §21.3 pp.709–714.
[Inventory/provenance](inventory.md) pins the program/log. Original MOM.dat
SHA256 `2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77`:
5,320 rows/532 people/T10/year1979–1988; unweighted lnhr on lnwg plus absorbed
person intercepts, no missing-case deletion.

[fe_level_bootstrap.td](fe_level_bootstrap.td) composes existing SQL with native
`xtreg ..., fe id(id) time(year)`, original FE prediction/report interfaces and
ordered CSV exports. B500 whole-person pairs refit the within slope **and
re-normalize the level intercept on every draw**. No shared kernel/parser/
runtime/schema/dependency changes or production R/Python requirement.

This is **not** [within_bootstrap.td](within_bootstrap.td)'s centered auxiliary-OLS
intercept (approximately zero). The old workflow remains unchanged. It also
does not implement classical analytic common-intercept inference: the native
`common_intercept_inference=normalization_point_only` contract is unchanged.
Bootstrap level-intercept uncertainty is a separate, explicitly labelled report.

## Sample, normalization and resampling semantics

Strict admission: 3..532 people, complete ten years1979–1988, unique finite
integer person/year/source_row keys within1..532/1979..1988/1..5320, finite
model values of absolute magnitude<=20. The prepared MOM.csv schema/original
ancillary fields are preserved. Every draw has G independent ten-row copies;
repeated source people do not reduce the copy count or residual DF.

For source unit i define within moments Wxx_i/Wxy_i and raw means mx_i/my_i.
Given multiplicities m_i (sum m_i=G):

- b=sum(m_i*Wxy_i)/sum(m_i*Wxx_i).
- mx=sum(m_i*mx_i)/G, my=sum(m_i*my_i)/G; **a=my-b*mx**.
- Unit total intercept alpha_i=my_i-b*mx_i; copy effect u_i=alpha_i-a.
  Thus sum(m_i*u_i)/G=0. On this balanced panel this is the native
  observation-weighted normalization, also the mean of copied unit intercepts.
- Within RSS is a direct weighted sum of squared demeaned residuals, not a
  cancellation-prone quadratic subtraction. Residual DF=N-G-1=9G-1 counts copies.

The zero-normalization certificate is checked on all501 original/bootstrap fits:
absolute effect mean<=1e-10*(1+abs(a)). Require within SS>1e-12, finite
coefficients of absolute magnitude<=100, finite nonnegative within RSS, correct
G and complete RNG output. Zero RSS/zero bootstrap variance are not alone invalid
point fits (replicate standard errors are not used). Any unidentified/invalid
replicate aborts the whole bootstrap: **no redraw, deletion or partial summary**.
The CLI's current error surface is `sql failed` with script line, not replication
number. There is no nonlinear optimizer/convergence claim here.

Each replicate uses xoshiro256++ seeded by SplitMix64 with10001+r-1, fixed-u64
rejection convention `u64_rejection_v1`, independent of the previous replicate's
stream. Maximum G+1024 raw steps per replicate; exhaustion fails. This is the
existing bounded RE-ML/refitting-GLS composition convention, **not** the generic
OLS bootstrap's continuing stream or a historical Stata RNG replay.

Coefficient order is (population/common level intercept, lnwg), explicitly named
rather than the historical stored matrix's slope/intercept order. Full symmetric
covariance uses denominator B-1; SE is its diagonal square root, bias is bootstrap
mean minus original fit, and intervals are R-type7 2.5/97.5 percentiles. No t/z,
p-values, BCa/studentization or unit-effect intervals are reported. Independent
sampled people, not arbitrary cross-person dependence, underlie pairs resampling.
It measures variability of the re-normalized empirical estimator under sampled
person composition; it is not the fixed-cohort analytic intercept standard error,
a causal claim or a finite/few-cluster coverage guarantee. Degenerate empirical
bootstrap intervals validate arithmetic, not population certainty.

## Reproduce and interpret

Build the native binary and follow [shared preparation](README.md#reproduce).
Run in a fresh working directory containing MOM.csv, with paths adjusted:

```sh
/path/to/tabdat-explore-rs --json -f /checkout/examples/mma21p1panel/fe_level_bootstrap.td > output.jsonl
# In another fresh directory with the same MOM.csv:
/path/to/tabdat-explore-rs --text -f /checkout/examples/mma21p1panel/fe_level_bootstrap.td > output.txt
```

The bounded report is a five-column terminal/CLI-v1 `TableResult`:
parameter/estimate/bootstrap_se/percentile95_lower/percentile95_upper.
Three ordered no-clobber exports:

1. `panel_fe_level_bootstrap_summary.csv`: B/sample/conventions, original and
   bootstrap means/bias, full2x2 covariance/SE and percentile95.
2. `panel_fe_level_bootstrap_draws.csv`: ordered replicate/copy/step/source person,
   all seven refit fields (a/b/raw means/DF/RSS/normalized effect mean), original
   coefficients and seed/sample metadata repeated consistently for audit.
3. `panel_fe_level_bootstrap_predictions.csv`: original source/provenance fields,
   native original FE xb/xbu/u/residual, after reloading the unchanged MOM.csv.

Original common intercept/slope:7.219891979652273/0.16767548862931483;
bootstrap SEs0.22629883080576527/0.08519411418969813. These are level-normalized
FE coefficients, not the auxiliary intercept or a Between/RE fit. Historical
Stata bootstrap SE equality is **not** asserted with a different RNG stream.
The native original FE model/sample/effects remain owned throughout; reloading
restores the original input order and generic prediction uses that original fit,
not any replicate. No resampled model is installed as current native state.

Exports are **not a whole-script transaction**: collision at a later target
preserves it, retains earlier successful exports and stops later commands.
Invalid draws stop before any bootstrap exports, but do not imply full session
rollback. Fresh directories avoid collisions without destructive overwrite.

## Independent reference and evidence

Development-only base-R reference (no extra R package):

```sh
Rscript /checkout/examples/mma21p1panel/fe_level_bootstrap_reference.R MOM.dat panel_fe_level_bootstrap_draws.csv fresh_reference
python3 /checkout/examples/mma21p1panel/check_fe_level_bootstrap.py . fresh_reference MOM.dat
python3 /checkout/examples/mma21p1panel/fe_level_bootstrap_synthetic.py heterogeneous fresh_control
python3 /checkout/examples/mma21p1panel/fe_level_bootstrap_synthetic.py zero_variance fresh_zero
# Run the native/reference workflow in each control directory; checker adds --synthetic.
```

R4.6.2 expands copied raw panels with fresh identities, demeans actual copied
observations, solves within least squares with `lm.fit` QR, and recovers all copy
intercepts/normalization. A separate full sum-to-zero contrast dummy QR verifies
the original level-intercept constraint on each original/control baseline.
Python independently replays every xoshiro/uint64-rejection draw and binds native
and R refits/predictions to exact raw source values. R outputs17 significant
digits. Only [generated fixtures](../../tests/fixtures/mma21_fe_level_bootstrap)
are checked in; no redistributed author MOM data.

Original evidence:266,000 exact draws, all500 seven-field R refits,17 summary
fields and5,320 FE prediction rows/42,560 exact raw fields. Maximum discrepancy:
refit fields2.90e-12 (RSS), summaries5.33e-15 and predictions3.56e-15. Fixed
absolute native/R tolerance1e-9 is unchanged. Original JSON/text CSVs are
byte-identical. Eight forgeries are rejected: person/coefficient/normalization/
DF/covariance/method/conditional prediction/raw source.

Two16-person interleaved/reverse-time controls add1,000 refits/16,000 exact draws
and320 original predictions. The heterogeneous control exercises per-draw means/
normalization; the binary-exact control has slope0.25 with zero slope variance,
zero cross-covariance, a point percentile interval and positive intercept variance.
All valid singular empirical covariances are retained. An identified three-person
original followed by replicate2 copies(2,3,3), none with within wage variation,
fails with no exports; independent dense QR confirms rank4 original versus
rank3 copied design (four parameters required).

Four [actual-script tests](../../tests/mma21_fe_level_bootstrap_workflow.rs) pass:
all frozen refits/summaries, typed/readable uncertainty, native normalization/DF,
original source/order/all four predictions, repeated people, zero variance,
JSON/text equality, empty/missing/out-of-bound/duplicate/incomplete panels, failed
later draw and all three collision stages. Three adjacent native-FE/cluster-
bootstrap regressions, scoped Clippy, formatting, Python syntax and local links
also pass. Author review; no delegated review. No whole-workspace/platform/
release certification or complete first-program/chapter/book closure.

Remaining required work includes classical analytic common-level-intercept
uncertainty, individual FE-effect intervals, full saved-model comparison reports,
historical replay and companion/full-book inventory. A next bounded candidate is
the classical FE common-intercept covariance under its explicit normalization;
do not substitute pairs-bootstrap uncertainty for that different inference target.
