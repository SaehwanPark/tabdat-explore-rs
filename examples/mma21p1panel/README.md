# MMA chapter 21: pooled, between, first-difference within-FE, RE-GLS and interior RE-ML hours/wages analyses

**Partial empirical coverage.** Cameron–Trivedi (2005), §21.3 pp.709–714,
Table21.2 p.710: POLS/first-difference, Between/RE-GLS classical and Within slope/conditional-prediction
portions of the March2005 author
[`mma21p1panfeandre.do`](https://cameron.econ.ucdavis.edu/mmabook/mma21p1panfeandre.do)
and [output](https://cameron.econ.ucdavis.edu/mmabook/mma21p1panfeandre.txt).
The header's “Table21.1” is inconsistent with the explicit Table21.2 labels in
the POLS/FD sections and final table block; the latter identifies this slice.
This is linear-panel chapter **21**, not clustered-sample chapter24.

POLS/FD/Between are **composed native SQL/OLS workflows**. `within.td` executes bounded
native `xtreg ..., fe id(id) time(year)`. All load, guard keys, fit, test the wage
slope, compute its 95% interval, summarize predictions/residuals and export usable
data through text/CLI-v1/scripts. POLS/FD compare three immutable named reports;
FE reports classical and unit-cluster results with a new `FixedEffectsResult`,
not `ols`. Between compares classical and HC1 named reports on person means. Persistent
`panel`/`xtdata` and whole-table coverage remain absent. Interior ML is documented below.

## Data and exact samples

Download the authors' public [MOM.dat](https://cameron.econ.ucdavis.edu/mmabook/MOM.dat).
It has 5,320 records of eight numeric fields: `lnhr lnwg kids ageh agesq disab id year`;
532 men, IDs1–532, each observed in 1979–1988 in person/year order. Source logs
have **two decimal places**; do not substitute chapter22's `MOMprecise.dat`.
Public access is not a redistribution license: no author dataset/program/log or
book pages are bundled. Download and use under the source's applicable terms.

`prepare.sh` invokes the strict AWK token-stream converter, preserves decimal
spellings and adds 1-based original-record `source_row` (not person ID). It accepts
CRLF, wrapped records and an unterminated last line. It rejects empty/incomplete,
malformed/nonfinite data and noninteger/out-of-range MOM keys **before** key
rounding; it stages beside the output, cleans up on failure and never overwrites
an existing file. Output parent directories must exist. CSV inference can store
`.00` keys as DOUBLE; their bounded integer **values** remain exact.

- **POLS:** unweighted `lnhr ~ 1 + lnwg`, all5,320 source rows. No optional
  covariates, year dummies, winsorization or further sample restriction.
- **FD:** compute `LAG` on raw rows, partitioned by `id`, ordered by `year`;
  retain only `year = previous_year + 1`, then fit `dlnhr ~ 1 + dlnwg`.
  Exactly4,788 annual pairs/532 people, ordered by id/year. The estimated FD
  constant is retained as in the author program; it is not the level intercept.
  Exports include current/previous source rows and years for sample audit.
- **Between:** one equally weighted person-mean row for each of 532 people,
  `mean_lnhr ~ 1 + mean_lnwg`. Both means use jointly finite outcome/wage
  pairs; on modified input, retain singleton people, omit people with no usable
  pairs, and never weight means by T_i. The export records each person's usable
  count and first/last usable year; these bounds do not imply gap-free input.
- **Within FE:** unweighted `lnhr ~ lnwg` with 532 absorbed person intercepts,
  all5,320 rows, no year effects or additional covariates. Unit centers are
  computed only after jointly finite outcome/predictor selection; singleton
  units are retained and counted on modified input. FE is not FD at T>2.
- All scripts reject duplicate/missing person-year keys before fitting and check
  nonempty numeric integer IDs1–532/years1979–1988 without altering source data.
  Unsorted/interleaved rows work; gaps are not bridged and no lag crosses people.
  Missing model values propagate through differences before finite-case fitting;
  exported adjacent-pair rows need not all be estimation rows on modified input.
  `RegressionResult.sample` distinguishes relation rows from dropped model cases.
  No panel declaration is retained and no general calendar/lag DSL is claimed.

## POLS/FD inference and interpretation

K=2 includes the intercept. Classical covariance is `(RSS/(N-K)) (X'X)^-1`;
HC1 is `N/(N-K)` times the independent-row score sandwich. Person clustering uses
`G/(G-1) * (N-1)/(N-K)` times the grouped-score sandwich with G=532. Coefficient,
`lincom` and one-restriction F=`t²` inference uses N−K Student-t/F denominator DF
for classical/HC1, and **G−1=531** for clusters. `RegressionResult.df_residual`
remains fit residual DF5318/4786; `lincom`/`test` expose the inference DF instead.
Finite-DF references are conventions, not exact robust/few-cluster coverage.

Classical and HC1 columns are comparison baselines, **not panel-robust**: row
independence is generally implausible, especially after differencing. Cluster
inference assumes independent people and permits arbitrary dependence within
person; it is not household/survey/multiway inference or a model for that
correlation. Differencing removes time-invariant additive person effects, not
all endogeneity; strict-exogeneity/identification assumptions remain necessary.
FD is not within-FE for T>2. Neither clustering nor differencing proves causality.

Native float64 results (rounded here):

| Model | Wage slope | Classical SE | HC1 SE | Person-cluster SE | Cluster p |
| --- | ---: | ---: | ---: | ---: | ---: |
| POLS | 0.0827435453 | 0.0091251365 | 0.0203041511 | 0.0292711464 | 0.0048788984 |
| FD | 0.1089851494 | 0.0213351420 | 0.0688513527 | 0.0837266162 | 0.1935906805 |

The FD cluster result illustrates why the classical significant t-test cannot
simply be interpreted as panel-robust evidence. Residual summaries/exports are
descriptive diagnostics, not serial-correlation or specification tests. Gaussian
working likelihood uses RSS/N, not a true panel likelihood; do not compare a
level-response likelihood with a differenced-response likelihood as an LR test.
Named reports are separate within each cohort, not restorable model caches.

## Within FE: inference, retained effects and predictions

`xtreg lnhr lnwg, fe id(id) time(year)` absorbs G person intercepts but reports
only p=1 slope. Classical covariance uses RSS/(N-G-p), residual/inference
**DF4787**. `cluster(id)` or its FE-specific alias `robust` uses grouped within
scores, correction **G/(G-1)*(N-1)/(N-p)** (R plm Arellano `sss`, nested-unit
convention), and slope/test t/F **DF531**. This is deliberately not OLS HC1 or an
N-G-p cluster correction. Other cluster variables are unsupported. Finite-DF
robust calibration and independence across people remain assumptions, not a
few-cluster guarantee. General native FE allows numeric/text/boolean unit IDs,
exact integer/decimal time keys and gapped/unbalanced/interleaved input; all raw
keys are audited before model-case deletion. Use integer/DECIMAL types for wide
IDs: DOUBLE cannot recover precision lost before fitting.

| Native float64 Within result | Value |
| --- | ---: |
| Wage slope | 0.1676754886 |
| Classical SE | 0.0188700064 |
| Unit-cluster SE | 0.0849626112 |
| Cluster p (DF531) | 0.0489542761 |
| Common-intercept normalization point | 7.2198919797 |
| Within R² | 0.0162265699 |

The common intercept c=mean(y)-mean(X)beta is a **point normalization**, not an
additional inferred coefficient. Retained normalized deviations u have
estimation-observation-weighted mean zero; total unit intercepts are c+u. There
is no nuisance-intercept SE/test, sigma-u/rho inference or likelihood. Conditional
level-fit R² includes fitted unit effects and is **not** Stata overall/between
R². `lincom`/`test` constrain retained slopes only; residual summaries are not
Hausman, serial-correlation or specification tests. FE removes additive
unit-constant effects, not all endogeneity; strict-exogeneity/identification
assumptions still matter.

Predictions use **current rows, retained coefficients and estimation-sample
centers**, without refitting:

- `xb` (also bare predict): c+current X beta, **not conditional fitted levels**.
- `xbu`: retained total unit intercept+current X beta; use for fitted levels.
- `u`: retained normalized unit deviation, independent of current y/X.
- `residuals`: current y minus xbu.

Conditional modes require the original ID name/type, but not time. Missing,
nonfinite or unestimated IDs produce NULL, never a zero effect. An entirely
excluded unit has no retained effect. Missing/nonfinite required values and
unrepresentable calculations produce NULL. xb needs no ID. Changing current
rows/order does not change the retained effects/covariance. Missing columns,
ID type mismatch, unsupported modes/intervals or existing targets fail without
publishing a column or changing fit/data. A failed runtime FE fit clears stale
estimation; syntax errors preserve it. No named FE report snapshots are supplied.

## Reproduce

From the repository root, using pinned Rust/Cargo1.99.1:

```sh
root=$(pwd)
cargo build --locked
work=$(mktemp -d)
cd "$work"
curl -fsSL https://cameron.econ.ucdavis.edu/mmabook/MOM.dat -o MOM.dat
printf '2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77  MOM.dat\n' | shasum -a 256 -c -
sh "$root/examples/mma21p1panel/prepare.sh" MOM.dat MOM.csv
"$root/target/debug/tabdat-explore-rs" --json -f "$root/examples/mma21p1panel/pooled.td" > pooled.jsonl
"$root/target/debug/tabdat-explore-rs" --json -f "$root/examples/mma21p1panel/first_difference.td" > fd.jsonl
# Development-only reference; installed TabDat requires neither R nor Python.
Rscript "$root/examples/mma21p1panel/reference.R" "$work/MOM.dat" "$work/reference"
python3 "$root/examples/mma21p1panel/check_reference.py" "$work" "$work/reference" "$work/MOM.dat"
"$root/target/debug/tabdat-explore-rs" --json -f "$root/examples/mma21p1panel/within.td" > within.jsonl
Rscript "$root/examples/mma21p1panel/within_reference.R" "$work/MOM.dat" "$work/within_reference"
python3 "$root/examples/mma21p1panel/check_within_reference.py" "$work" "$work/within_reference" "$work/MOM.dat"
```

Use a **fresh** directory for reruns: exports intentionally refuse existing
paths. For readable results, copy MOM.csv to another fresh directory and run the
same scripts with `--text` instead of `--json` (save stdout to `.txt`). Default
batch/script success is quiet. Paths inside scripts resolve from the process's
working directory. A script stops on error; it is not a transaction rolling back
all earlier session effects. Source files are never rewritten.

## Independent evidence and limits

R4.6.1 `lm` (QR), sandwich3.1.1 `vcovHC(type='HC1')` / `vcovCL(type='HC1',
cadjust=TRUE)`, independent grouped-score correction algebra and Student-t/F
reference functions validate **180 scalar quantities**: both designs, all three
covariances, coefficients/SEs/tails, full covariance, fit statistics, tests and
slope intervals/DF. All **10,108 exported rows /111,188 numeric cells** are audited,
including original source values/keys, every difference and fitted/residual value.
Absolute tolerance is1e-9; nonzero p-values use relative1e-8 (zero tails must be
zero); row/year/person identities compare exactly. No algorithm or tolerance is
changed to fit the author log.

The author's `describe` explicitly records float32 variables. Its POLS displayed
slope0.0827436 differs in the last displayed digit from rounding native float64
slope0.0827435453. R separately reproduces the float32 ingestion **and generated
differences**, checking24 coefficient/SE facts in `author_printed.csv` within
half a unit of their original printed precision. These float32 facts are not
the native numerical oracle. Author files' SHA256:

- `.do`: `775890ef25903477e5be24027c070705910247daf46f9ed7b13d0ed0b48cc44a`
- `.txt`: `3c4e65bdfcd6fe8f77fe32361bc29e041ad7caf9268ba35ecf52e2c78d5b88f6`

`tests/mma21_panel_workflow.rs` runs the actual scripts on a small synthetic,
reverse/interleaved, gapped/incomplete panel, checks exact previous-row identity,
NULL propagation, estimation/cluster DF, identical JSON/text exports, key-error
stopping, input preservation, refused exports and converter failure cleanup.
The synthetic fixture protects semantics but is not textbook evidence by itself.
Verified on macOS arm64; no broader platform claim follows.

For FE, R4.6.1/plm2.6.7, explicit LSDV and independent grouped-score algebra
validate **64 scalar quantities**, all **5,320 exported rows/90,440 cells** and
**1,064 recovered unit-intercept comparisons** (532 units under both covariance
choices). Every original field/key, common/conditional fit, normalized effect,
residual, slope covariance/SE/tail/test/interval/DF and advertised fit statistic
is checked. Separate float32 ingestion reproduces **six printed Within facts**
(slope, classical SE, common-intercept point, sigma-e, within R² and slope F) at
half a unit of original printed precision. Author bootstrap SEs are not the
native analytic cluster oracle, and neither bootstrap nor nuisance-intercept
inference is claimed. The same absolute1e-9/nonzero-tail relative1e-8 tolerances
apply; native float64 is never rounded to mimic Stata8.

Synthetic numerical fixtures add **843 R/plm/LSDV comparisons** across balanced,
unbalanced/singleton and interleaved designs, including two slopes/full
covariance, shifts and post-estimation. Runtime tests cover exact UBIGINT keys,
joint-case centers, current-vs-retained semantics and failure atomicity. The
actual within script also runs on the small gapped/incomplete CLI fixture, with
absorbed/inference DF, all prediction modes, source order, identical text/JSON
exports and refused overwrites checked. Full-source text/JSON exports were also
byte-identical. No author data/program/log is bundled by these references.

**Still required:** robust ML inference, Hausman and panel-specific
residual/model-comparison diagnostics, bootstrap/RNG, optional richer designs,
lowess/Figures21.1–21.4, remaining Table21.2/book outputs and other panel
illustrations. This slice completes neither the panel family nor MMA-PANEL.

## Between: mean-response inference and reproduction

`between.td` audits all raw keys before case deletion, selects joint-finite
lnhr/lnwg rows, and groups by ID in ascending order into `between_means`.
The active relation becomes the person means, leaving the input CSV intact.
It fits an intercept and wage slope, stores classical/HC1 reports, tests the
slope, computes its interval, summarizes fitted means/residuals and exports
`panel_between_predictions.csv`. Its OLS result reports **N=532 mean rows**,
not 5,320 original records. The initial key audit reports original counts.
Predict/residual values describe person means, not individual-year hours or
recovered individual effects. Other model values/covariates are outside this
design. Empty/rank-deficient mean designs fail without a completed fit/export;
existing exports are never replaced. Earlier script effects are not rolled back.

Classical covariance is RSS/(G-2) times inverse mean-design crossproduct;
HC1 is G/(G-2) times the sandwich of independent-person mean-row scores.
Both use Student-t/F DF530 on the original sample. The wage slope is
0.0668378461, classical SE0.0196634913/p0.0007270164 and HC1
SE0.0243184885/p0.0061921439; between R²0.0213344794 and residual
SD0.1772554957. HC1 assumes independent people, permits heteroskedastic mean
errors, and is an analytic comparison, **not the author's panel bootstrap**.
There is one score row per person, so no additional person-cluster fit is needed.
Neither averaging nor HC1 removes correlation between wages and person effects;
this is not FE or a causal identification argument. The Gaussian working
likelihood is for mean responses, not a panel likelihood or a valid cross-design
LR comparison. The reported R² is mean-response OLS R², not within/overall R².

Append these commands to the fresh-directory reproduction above:

```sh
"$root/target/debug/tabdat-explore-rs" --json -f "$root/examples/mma21p1panel/between.td" > between.jsonl
Rscript "$root/examples/mma21p1panel/between_reference.R" "$work/MOM.dat" "$work/between_reference"
python3 "$root/examples/mma21p1panel/check_between_reference.py" "$work" "$work/between_reference" "$work/MOM.dat"
```

R4.6.1/lm and sandwich3.1.1 independently validate **60 scalar quantities**:
coefficients/SEs/tails, full covariance, fit statistics, tests and slope
intervals/DF under both covariances. All **532 exported rows/4,256 cells**
compare person identity, counts, year bounds, means and predictions/residuals.
The checker pins the original source hash; tolerance is absolute1e-9 for numeric
values, relative1e-8 for nonzero tails, exact identities/counts/year bounds.
Separate float32 ingestion reproduces **seven author printed classical facts**
at half their printed precision; float64 is never rounded to match the old log.
The synthetic CLI case checks unequal row counts, a singleton, an entirely
excluded person, missing outcome/predictor pairs, exact mean membership,
equal-person weighting, inference DF, identical text/JSON exports and overwrite
refusal. Bad keys, no usable pairs and rank-deficient designs fail before export.
No data/program/log is redistributed. This validates the classical Between
coefficient/inference portion and an HC1 extension, not bootstrap, Figure21.2,
all Stata panel fit statistics, general `xtreg ..., be`, or whole-table coverage.

## RE-GLS: balanced Swamy–Arora workflow

`random_effects.td` executes the classical coefficient/inference portion of
Table21.2's RE-GLS column: guard original MOM keys, fit native
`xtreg lnhr lnwg, re id(id) time(year)`, inspect normal coefficient intervals
and joint zero-slope Wald inference, predict population xb/residuals, summarize
and export `panel_re_gls_predictions.csv`. Source order and all original fields
remain in the export. This is a separate `RandomEffectsResult`, not OLS or FE.
There is no production R/Python dependency.

The exact published design is lnhr~1+lnwg on all5,320 rows/532 people/T10.
This author-table fit audits raw unit/time keys before joint finite-case
selection. Retained groups must have equal counts T>=2; G>p+1; both within and
between preliminary designs must identify every slope. Entirely excluded units
are omitted. Interleaved/gapped data are allowed; equal counts do not promise
common years. Time-invariant predictors, weights, arbitrary cluster variables
and alternative component estimators are not supplied. The separate unbalanced
Swamy–Arora extension below covers unequal retained counts.

Variance and inference are explicit:

- sigma_e² = within RSS/(N-G-p).
- raw sigma_u² = between mean-row RSS/(G-p-1) − sigma_e²/T;
  sigma_u²=max(0,raw sigma_u²). The raw estimate is reported, including negatives.
- theta=1−sqrt(sigma_e²/(sigma_e²+T*sigma_u²)); quasi-demean outcome, slopes
  **and intercept**. A zero unit component gives theta0/pooled OLS coefficients.
- Model-based covariance = transformed RSS/(N-p-1) times inverse transformed
  crossproduct, matching balanced R/plm and the author GLS output. The final
  covariance scale is distinct from the preliminary sigma_e² estimate.
- Coefficients use normal z/two-sided tails/95% intervals; the joint slope Wald
  uses asymptotic chi-square(p). Residual DF5318 describes the transformed scale,
  **not a Student-t reference**. Zero/nonfinite sigma_e², rank/DF failure,
  numerically degenerate covariance/intervals or a lost transformed intercept
  fail without a published fit. No within/between/overall R² or panel likelihood
  is reported. General `test`/`lincom` and named RE reports are not installed.

Native float64 wage slope0.1193322439 has SE0.0136312206 (normal inference);
component SDs are about0.23278339/0.16124733 (idiosyncratic/unit), rho0.32424354.
RE assumes unit effects exogenous to regressors and the specified error-component
covariance with independent units; model-based SEs are not panel-robust or a
few-unit guarantee. These assumptions require judgment, not inference from the
successful fit. The analytic SE is not the author's panel bootstrap SE.

`predict ..., xb` (also bare predict) evaluates the retained common intercept
and slopes on **current X**, with no ID/time needed. `residuals` is current y
minus population xb, including the unit component; it is not an idiosyncratic
conditional residual. Missing/nonfinite required cells or arithmetic yield NULL.
Data edits/reordering do not refit the model. Prediction/unsupported restriction
failures preserve fit/data; failed explicit-key RE fit attempts clear stale
estimation and preserve data. Legacy bare RE unsupported requests and parse
errors preserve the previous fit. Exports refuse existing destinations.

From the fresh reproduction directory above:

```sh
"$root/target/debug/tabdat-explore-rs" --json -f "$root/examples/mma21p1panel/random_effects.td" > random_effects.jsonl
Rscript "$root/examples/mma21p1panel/random_effects_reference.R" "$work/MOM.dat" "$work/re_reference"
python3 "$root/examples/mma21p1panel/check_random_effects_reference.py" "$work" "$work/re_reference" "$work/MOM.dat"
```

R4.6.1/plm2.6.7 plus explicit independent matrix algebra validate **30 scalar
quantities** (full covariance, coefficient z/tails/intervals, component variances,
transformation, fit scale and Wald) and **58,520 exported cells/5,320 rows**
covering every original field/key and population prediction/residual. Absolute
1e-9/nonzero-tail relative1e-8 tolerances and exact row/person/year identity
remain unchanged. Separate float32 ingestion reproduces **eight author printed
GLS facts** at half-unit printed precision. Synthetic R/plm/matrix fixtures add
**352 numerical comparisons**, including two slopes/full covariance,
interleaving, outcome shifts and negative raw unit variance truncation. Runtime
and CLI tests check joint-case membership/balance, fully excluded units, current
predictions after edits/dropped IDs, normal reporting, identical text/JSON exports,
raw-key/unbalanced/sample failures, atomic errors and export/source preservation.
Full-source text/JSON exports are also byte-identical.

Still required: unit effects/conditional predictions, robust ML inference,
Hausman and other panel diagnostics, bootstrap and remaining table/figure outputs.
This bounded balanced classical RE-GLS portion does not complete
the column/table, panel family or textbook target.

## Unit-cluster RE extension

`random_effects_cluster.td` supplies both explicit `cluster(id)` and equivalent
`robust` fits on the same balanced MOM design. Neither changes sample,
components, theta, coefficients or population predictions. The clustered
sandwich uses scores of the **quasi-demeaned** design/residuals and multiplier
**G/(G-1)*(N-1)/(N-K)** with K=p+1 including the transformed intercept.
This is R/plm Arellano `sss`, not FE's nested-effect N-p denominator, independent-row
HC1 or a substitution of preliminary sigma_e². Score groups are summed in key
order to make repeated fits/aliases reproducible; this also stabilizes the shared
OLS/IV/FE cluster path without changing its formulas.

Coefficient intervals/tails remain normal and the omnibus zero-slope Wald uses
chi-square(p), not G-1 t/F. A singular slope covariance yields unavailable Wald
statistic/p and an explicit reason, preserving requested p, positive marginal SEs
and predictions; no pseudoinverse or reduced-rank test is invented. Zero/nonfinite
marginal variances remain a fit failure. Component diagnostics and residual DF
retain their meanings under the selected covariance.

The original wage slope remains0.1193322439; cluster SE0.0514015848,
normal p0.0202562407 and 95% interval[0.0185869889,0.2200774990]. The much
larger SE than model-based0.0136312206 demonstrates the inferential consequence
of allowing arbitrary within-person dependence. It assumes independent people
and exogenous unit effects; it is not the author's bootstrap, household/site,
survey/multiway inference or a few-cluster guarantee. It supplies no additional
claimed Table21.2 bootstrap output or whole-column/table completion.

From the fresh reproduction directory above:

```sh
"$root/target/debug/tabdat-explore-rs" --json -f "$root/examples/mma21p1panel/random_effects_cluster.td" > random_effects_cluster.jsonl
Rscript "$root/examples/mma21p1panel/random_effects_cluster_reference.R" "$work/MOM.dat" "$work/re_cluster_reference"
python3 "$root/examples/mma21p1panel/check_random_effects_cluster_reference.py" "$work" "$work/re_cluster_reference" "$work/MOM.dat"
```

R4.6.1/plm2.6.7 and explicit grouped-score matrix algebra validate **31 scalar
quantities per alias /62 across both fits**, with identical native alias results,
and all **69,160 exported cells/5,320 source/design/prediction rows**. Absolute
1e-9/nonzero-tail relative1e-8 tolerances and exact source identity stay unchanged.
Text/JSON full-source exports are byte-identical. Synthetic fixtures add **438
R/plm/group-score comparisons**, including two slopes/full covariance, shifts,
interleaving, zero unit variance and estimable designs with singular clustered
slope covariance. Runtime/CLI tests exercise alias equality, unchanged point fit,
sample/components/predictions, arbitrary-cluster errors/cleared stale fits,
current prediction preservation, unavailable-Wald text/JSON and normal prediction
from that retained fit, as well as source/export/error contracts. Unbalanced GLS
and its unit-cluster covariance are documented in the separate sensitivity
workflow below.
Unit effects, robust ML inference, general restrictions, Hausman, bootstrap and
remaining table/figure outputs stay required.

## Interior RE-ML coefficient/likelihood workflow

`random_effects_ml.td` executes `xtreg lnhr lnwg, mle id(id) time(year)` on
all5,320 MOM rows/532 people/T10, reports normal coefficient uncertainty and
joint slope Wald, marginal likelihood and a re-estimated intercept-only null/LR,
then summarizes and exports population xb/residuals. This is Gaussian marginal
ML, not REML. Assumptions include exogenous unit effects, independent people and
the specified Gaussian error-components distribution; a fit does not establish
identification. Do not use level-versus-FD likelihoods as an LR comparison.

Covariance is the beta block of the inverse joint observed information for beta,
log(sigma_e²), log(sigma_e²+T*sigma_u²). R4.6.1/nlme3.1.169 validates point fits
and marginal likelihoods; independent matrix information is numerically checked
with base-R `optimHess`. The exact-stationary float64 fit has slope0.1195474024,
SE0.0137485749, LL−266.9114970638, null LL−304.9798136110 and slope LR76.1366330945.
The nlme default conditional coefficient SE (about0.0136506997) is a different
convention and is not substituted. Under float32 ingestion, point estimates and
likelihood reproduce author precision, but several printed Stata8 SEs differ
slightly from exact stationary joint information. That historical discrepancy
remains unresolved; this does not validate the published uncertainty column or
whole Table21.2. Tolerances are not widened to match it.

The original-MOM fit is interior; the supported workflow is **balanced**: retained T>=2, G>p+1,
full within rank, no weights/time-invariant predictors/robust ML. The profile
scans eta in[0,24] at0.5 spacing, polishes detected positive-to-negative score
crossings, and compares their likelihoods with the zero-component boundary.
This is not proof of a global maximum for arbitrary designs. Unresolved upper
tails, nonstationarity and nonpositive
joint information fail explicitly, clearing stale model state without altering
data. Zero-unit-variance fits use the constrained inference below. If either the
full or intercept-only null fit is on that boundary, the ordinary slope LR
p-value is withheld with a reason. The one-sided unit-variance LR test is reported
with its boundary-mixture reference. On the original MOM sample it gives
LR=1147.0828086 and mixture p=9.6575701e-252; this method extension is not an
author-reported Table21.2 result.

For positive interior fits, `ml.variance_component_inference` transforms the joint
observed-information covariance to delta-method SEs on the variance scale and
exponentiated 95% log-Wald intervals. Balanced ML stores log(sigma_e²) and
log(sigma_e²+T*sigma_u²), so unit-variance uncertainty uses both covariance
coordinates. At a zero-unit-variance boundary, its SE/interval is null with a
reason; no regular normal or profile interval is claimed. This inference is
model-based and does not enable robust/cluster ML.

Reproduce after preparing MOM.csv and checking the source hash as above:

```sh
cd "$work"
"$root/target/debug/tabdat-explore-rs" --json -f "$root/examples/mma21p1panel/random_effects_ml.td" > random_effects_ml.jsonl
Rscript "$root/examples/mma21p1panel/random_effects_ml_probe.R" "$work/MOM.dat" float64 "$work/re_ml_reference"
python3 "$root/examples/mma21p1panel/check_random_effects_ml_reference.py" "$work" "$work/re_ml_reference" "$work/MOM.dat"
# Optional separate historical-storage audit:
Rscript "$root/examples/mma21p1panel/random_effects_ml_probe.R" "$work/MOM.dat" float32
```

The full-source audit compares **48 scalar quantities** and **58,520 exported
cells/all5,320 source/design/prediction rows**, including complete joint and
coefficient covariance, intervals, normal tails, both likelihoods and LR, with
absolute1e-9/nonzero-tail relative1e-8 and exact source identity. Text/JSON exports
are byte-identical. **495 synthetic comparisons** cover two slopes/full covariance,
the unit-variance boundary-mixture test, shifts/interleaving and the boundary-null
reference policy. The latter also
exercises a negative score at eta=0 with a higher interior optimum; an independently
initialized nlme fit checks that interior solution. Runtime/CLI tests cover joint
sample mapping, current-data predictions, unavailable-mode errors and data/state
preservation, including zero-component fit failure. No production R/Python is used.

Still required: robust ML inference, conditional
unit effects, general restrictions/named reports, Hausman/panel diagnostics,
bootstrap and the remaining table/figure analyses. A manual core-only original
probe remains available via `TABDAT_REML_SOURCE` (the historical variable name):

```sh
(cd "$root" && TABDAT_REML_SOURCE="$work/MOM.dat" cargo test --locked -p tabdat-stats --test random_effects_ml_original_source -- --ignored --nocapture)
```

### Zero-unit-variance ML extension

The same `mle` workflow now accepts a selected eta=0/sigma_u²=0 profile maximum,
checking the nonpositive inward score instead of demanding interior stationarity.
The point fit equals pooled Gaussian ML, with sigma_e²=RSS/N, theta=rho=0.
This is not an automatic OLS-report fallback: coefficient covariance uses the
observed information constrained to beta/log(sigma_e²), with normal rather than
Student-t inference and no RSS/(N-K) correction. There is no regular normal-Wald
unit-variance SE. The one-sided profile LR test uses the asymptotic 50:50
chi-square(0,1) mixture (Self and Liang 1987
[doi:10.1080/01621459.1987.10478472](https://doi.org/10.1080/01621459.1987.10478472));
it is not a finite-sample or few-unit correction. The existing bounded
profile still does not prove global optimality for arbitrary designs.

CLI-v1 marks `ml.unit_variance_boundary`, uses
`constrained_observed_information` at the boundary, and gives the matching K+1
joint parameter names/covariance. Interior fits retain the K+2 representation.
Ordinary slope LR references are withheld if either full or null variance is on
the boundary, with full/null/both reasons. The strict interior-only Rust entry
point and public result-struct fields remain compatible; its boundary rejection
is preserved. Metadata derives the flag from eta, without adding a required Rust
construction field. GLS/cluster behavior remains unchanged.

Three small balanced designs validate **378 base-R Gaussian lm ML quantities**:
coefficients, normal uncertainty/intervals/Wald, complete constrained and beta
covariance, every fitted/residual value, ML variance and both likelihoods/LR.
Base-R `optimHess` independently checks constrained information; `nlme` checks
interior null fits, and base-R intercept-only Gaussian ML checks a boundary null.
Cases include shifted/interleaved input and full/null/both boundary combinations
across this extension and the existing interior fixtures. Tolerances stay
absolute1e-9/nonzero-tail relative1e-8; all447 baseline interior comparisons remain
intact, with40 added variance-component uncertainty values and eight unit-variance
test values.
Runtime/CLI checks protect successful model replacement, unchanged raw data,
population prediction/export, explicit constrained JSON/text, identical text/JSON
CSVs, and genuine degenerate fit failure clearing stale state while preserving data.
This adds synthetic boundary-method evidence; it does not add a published
Table21.2 row or resolve the historical Stata8 SE discrepancy.

### Unbalanced Gaussian ML sensitivity workflow

The workflow targets the same lnhr~1+lnwg analysis on an explicit sensitivity cohort: omit
(id divisible by3, year1980 or1985) and (id divisible by7, year1981–1983),
retaining original source-row identity/order. The audited source gives4,738 rows,
532 people and retained counts5–10. This changes the published sample and is
method/sensitivity evidence, not another author-table replication.

Use per-unit quasi-demeaning and determinant terms for v_g=sigma_e²+T_g*sigma_u²,
with a bounded scalar profile eta=log(1+T_min*sigma_u²/sigma_e²). Re-estimate the
intercept-only null with the same unequal-count likelihood; its intercept is
variance-weighted, not automatically the grand mean. The full beta /
log(sigma_e²) / log(sigma_u²) observed information is checked against independent R/nlme and
numerical Hessians, including singleton/interleaved/shifted and boundary cases.
Balanced/strict-interior APIs and outputs are preserved. Reports give unequal counts and
theta ranges explicitly, with a distinct unbalanced JSON result; keep the
zero-unit boundary constrained and ordinary LR-reference withholding intact.
The script provides load/guard/cohort/fit/report/predict/export. The full native
MOM audit compares65 reported scalars and52,118 exported cells/all4,738 rows;
text/JSON CSVs are byte-identical. The source comparison includes the 10
variance-component uncertainty values and both unit-variance LR values. Across
fixtures, the531 synthetic comparisons include singletons, shifts,
interleaving, interior and zero-unit boundary fits; 65 compare component SEs and
log-Wald intervals across balanced, unbalanced and boundary cases. Runtime tests check actual
joint-case counts, retained metadata after current-data changes, and failed fit
state. Tolerances remain absolute1e-9/nonzero-tail relative1e-8. No production
R/Python dependency is added.

Reproduce in a fresh directory containing prepared MOM.csv (paths as above):

```sh
"$root/target/debug/tabdat-explore-rs" --json -f "$root/examples/mma21p1panel/random_effects_ml_unbalanced.td" > random_effects_ml_unbalanced.jsonl
mkdir unbalanced_reference
Rscript "$root/crates/tabdat-stats/tests/fixtures/random_effects_ml_unbalanced_reference.R" unbalanced_reference/reference.csv unbalanced_reference/predictions.csv "$work/MOM.dat"
python3 "$root/examples/mma21p1panel/check_random_effects_ml_unbalanced_reference.py" . unbalanced_reference "$work/MOM.dat"
```

The full/null LL are−268.1239288317/−306.7565090109 (slope LR77.2651603583).
The log-wage slope is0.124026401777 with joint-information SE0.0141569248153.
Unequal ML JSON uses `UnbalancedRandomEffectsMlResult`, null common T/theta,
actual count/theta ranges and eta reference T_min. The Rust group_size=0 sentinel
uses retained-sample range methods, with no new required result-struct fields.
The bounded profile, covariance assumptions, boundary LR withholding and original
printed-Stata8 uncertainty discrepancy remain as documented above.

### Robust auxiliary Hausman comparison — §21.4.3 p.719

[`robust_hausman.td`](robust_hausman.td) implements block7 of the authors'
March2005 [manual panel program](https://cameron.econ.ucdavis.edu/mmabook/mma21p2panmanual.do)
and [output](https://cameron.econ.ucdavis.edu/mmabook/mma21p2panmanual.txt),
on all5,320 MOM rows/532 people/T10 with lnhr~1+lnwg. The first author program
warns that conventional FE-minus-RE covariance subtraction is invalid when
residual within-person dependence makes RE inefficient. This workflow uses the
preferred auxiliary regression, not that conventional test.

After auditing all raw keys and unique integer source_row1..5320, select joint
finite lnhr/lnwg cases. Compute unit means and balanced Swamy–Arora components:
sigma_e²=within RSS/(N-G-1); raw sigma_u²=between RSS/(G-2)-sigma_e²/T,
truncated at zero. Require balanced T>=2/G>3, identified within/between designs,
positive finite sigma_e² and finite theta in[0,1). The complete MOM sample has
no exclusions; an entire excluded unit is allowed, while residual imbalance
fails. Ordered window/aggregate inputs preserve deterministic source/design
values across text/JSON. No panel declaration or implicit lag operation is used.

Regress quasi_lnhr=lnhr-theta*mean_lnhr on an intercept,
quasi_lnwg=lnwg-theta*mean_lnwg and within_lnwg=lnwg-mean_lnwg, clustered by id.
The restriction within_lnwg=0 is the comparison: use the existing OLS group-score
multiplier G/(G-1)*(N-1)/(N-K), K3, and t/F reference DF G-1. The one-restriction
F equals the author's displayed H=t², **1.649507405254**, with native F(1,531)
p=**0.199586037144**. Its asymptotic chi-square(1) tail would be0.199025989128;
these references are not silently interchanged. `lincom` supplies the auxiliary
coefficient interval; predictions/residuals belong to the auxiliary regression,
not the original lnhr equation. Raw and every transformed column are exported.

This is asymptotic panel-robust inference under independent units and the
maintained linear/exogeneity conditions, not a few-cluster or finite-sample
exact guarantee. The generated transformation is treated as fixed in the
auxiliary covariance, following the author program. Failure to reject does not
prove RE exogeneity or automatically select a model. It is not a test that the
unit variance is zero. No general native `estat hausman`, arbitrary-model
comparison, unbalanced auxiliary workflow or bootstrap is installed.

Reproduce in a fresh directory containing prepared MOM.csv (root/work as above):

```sh
"$root/target/debug/tabdat-explore-rs" --json -f "$root/examples/mma21p1panel/robust_hausman.td" > robust_hausman.jsonl
Rscript "$root/examples/mma21p1panel/robust_hausman_reference.R" MOM.csv hausman_reference
python3 "$root/examples/mma21p1panel/check_robust_hausman_reference.py" . hausman_reference "$work/MOM.dat"
# Separate selected printed-fact audit; never round native float64 references:
Rscript "$root/examples/mma21p1panel/robust_hausman_reference.R" MOM.csv hausman_float32 float32
```

R4.6.1/plm2.6.7/sandwich3.1.1 and independent grouped-score algebra audit
**39 coefficient/covariance/inference/fit quantities and95,760 exported cells /
all5,320 rows**, including complete raw/source/mean/within/quasi/prediction data.
Tolerances remain absolute1e-9/nonzero-tail relative1e-8 with exact identities.
Text/JSON exports are byte-identical. Separate float32 ingestion reproduces8
selected printed coefficient/SE/H/theta facts at half a printed unit; this does
not claim every Stata intermediate storage operation or printed column.

The committed24-row independent R fixture (plus4 excluded-unit raw rows) covers
interleaving, exact joint-case means and full inference/export. CLI checks also
cover empty/bad/duplicate keys, missing-case imbalance, within/between rank,
extreme theta failure, source ordering and refused-overwrite byte preservation.
Only small synthetic fixtures are bundled; no production R/Python dependency.
Earlier successful script effects are not rolled back after a later error.

### Seeded Between IID pairs bootstrap

[`between_bootstrap.td`](between_bootstrap.td) implements the Between-bootstrap
analysis of the March2005 manual program on all5,320 MOM observations/532 people.
First select joint-finite lnhr/lnwg cases, compute person means (ordered by year)
and order the532 means by person. Resample these independent mean-response/design
records, not individual person-years, B200 with explicit native seed10001.
Unweighted OLS supplies the original intercept and wage slope.

`bootstrap mean_lnhr mean_lnwg, reps(200) seed(10001)` reports point estimates,
replicate means/bias, B-1 SE/full covariance and fixed95% percentile intervals
with R-type7 interpolation. It is read-only: the existing OLS fit remains usable
for prediction/export. Every replicate must identify all coefficients; any rank
or numerical failure aborts with its index, with no dropped/redrawn samples.
There are no analytic normal/t p-values, BCa/basic/studentized intervals or
finite-sample/nominal-coverage guarantee. Finite B has Monte Carlo error.

The local xoshiro256++/SplitMix64 stream restarts on every explicit unsigned64-bit
seed, using the versioned u64 rejection mapping documented in `help bootstrap`.
The seed is decimal text in CLI-v1 JSON, preserving all64 bits in JavaScript.
The canonical backend preserves insertion order for the plain projection used
by the sampler; reproducibility requires the same physical sample order/design.
Script context-only seed directives do not initialize this command. This does
not introduce generic command prefixes, raw panel/cluster/residual/parametric
resampling, weights or survey inference. The author's Between block did not
specify this native stream: no equality to its particular random draw or printed
bootstrap SE is claimed. IID person-mean resampling is not a FE/RE panel bootstrap.

Reproduce in a fresh directory containing prepared MOM.csv:

```sh
"$root/target/debug/tabdat-explore-rs" --json -f "$root/examples/mma21p1panel/between_bootstrap.td" > between_bootstrap.jsonl
Rscript "$root/examples/mma21p1panel/between_bootstrap_reference.R" "$work/MOM.dat" between_bootstrap_reference
python3 "$root/examples/mma21p1panel/check_between_bootstrap_reference.py" . between_bootstrap_reference "$work/MOM.dat"
```

Independent Python unsigned-integer reference draws and R4.6.1 lm/cov/quantile
validate18 bootstrap quantities and4,256 person-mean/design/prediction cells,
with all5,320 loaded raw rows pinned to the original source. Native bootstrap
SEs are0.0623527028813 (intercept) and0.0231855272708 (wage slope). Absolute1e-9
numerical tolerances and exact raw identity are retained; text/JSON exports are
byte-identical. Synthetic fixtures validate1,440 exact indices and76 R quantities
for two-slope/no-intercept/shifted designs, plus repeatability, exact64-bit seeds,
joint missingness/sample mapping, original/replicate failure and unchanged
labels/data/model/named reports. The actual script runs on a small inspectable
interleaved-person fixture and refuses existing exports. Native RNG support adds
only rand_xoshiro0.7.0/rand_core0.9.5, with no production R/Python/native dependency.
Current bounds are B2..5000 and N*B*K²<=100,000,000 (K includes the intercept).

### Whole-unit pairs bootstrap of within-transformed OLS

[`within_bootstrap.td`](within_bootstrap.td) implements March2005
mma21p2panmanual.do block4 (§21.3 pp709–714/Table21.2): form joint-finite
lnhr/lnwg within-person deviations, fit ordinary transformed OLS with intercept,
then resample532 whole people with replacement per replication, B500/explicit
native seed10001. All5,320 original MOM rows are retained. Original within-person
row order and repeated whole-unit blocks are preserved. The original transformed
OLS fit remains usable for prediction/export; this is a read-only report.

`bootstrap within_lnhr within_lnwg, cluster(id) reps(500) seed(10001)` reports
B-1 covariance/SE, replicate means/bias and type7 percentile95 intervals. The wage
slope bootstrap SE is0.0849753799988. The centered intercept is numerical zero;
its tiny floating-point variability is not FE nuisance-intercept inference.
These fixed input transformations match the manual program; there is no automatic
FE/RE refit, RE variance-component re-estimation, residual/wild bootstrap or
few-cluster/nominal-coverage guarantee. No equality to the historical Stata draw
stream or printed bootstrap SE is claimed.

Reproduce in a fresh directory containing prepared MOM.csv:

```sh
"$root/target/debug/tabdat-explore-rs" --json -f "$root/examples/mma21p1panel/within_bootstrap.td" > within_bootstrap.jsonl
Rscript "$root/examples/mma21p1panel/within_bootstrap_reference.R" "$work/MOM.dat" within_bootstrap_reference
python3 "$root/examples/mma21p1panel/check_within_bootstrap_reference.py" . within_bootstrap_reference "$work/MOM.dat"
```

Independent Python group draws and R4.6.1 lm/cov/type7 validate18 quantities and
47,880 identity/design/prediction cells; all loaded raw rows are pinned separately.
Text/JSON exports are byte-identical. Absolute1e-9 numerical tolerances are retained;
the mathematically zero centered intercept is checked as numerical zero, not
relative SE accuracy. Synthetic balanced/unbalanced/no-intercept/shifted cases
check640 exact group indices and285 R numerical/size quantities. Runtime tests
cover joint exclusions, first-seen order, exact decimal/wide keys, signed zero,
nonfinite floating keys, variable draw sizes and unchanged models/data/reports.
The actual script also runs on an interleaved-person fixture and refuses existing
exports. Any failed replication aborts without redraw/drop. Work is bounded by
G*largest_cluster_size*B*K²<=100,000,000; IID stream/API behavior is preserved.

### Fixed-transform RE-GLS whole-person bootstrap

[`random_effects_gls_bootstrap.td`](random_effects_gls_bootstrap.td) implements
March2005 mma21p2panmanual.do block5 on all5,320 MOM rows/532 people. Guard raw
person/year/source keys and balanced joint-finite lnhr/lnwg cases (T>=2,G>2).
Estimate balanced Swamy–Arora theta once using within N-G-1 and Between G-2
variance denominators, with negative unit variance truncated at zero. Require
identified within/between wage designs, positive finite within variance and
finite theta in[0,1). Compute ordered means/quasi response/design, fit ordinary
transformed OLS with intercept, then resample532 whole people with replacement,
B500/explicit native seed10001. The retained transformed fit supplies predictions.

The transformed intercept is **(1-theta) times the population RE intercept**.
Theta stays fixed across draws, matching this manual program. The bootstrap wage
slope SE is0.0512913829732 and transformed-intercept SE is0.0568585013273.
This is conditional fixed-transform inference; it does not re-estimate RE variance
components or provide unconditional native RE-GLS uncertainty. No equality to
historical Stata draws/printed SEs, few-cluster accuracy or nominal coverage is
claimed. The report uses B-1 covariance and type7 percentile95 intervals, without
analytic p-values; every replicate must succeed without redraw/discard.

Reproduce in a fresh directory containing prepared MOM.csv:

```sh
"$root/target/debug/tabdat-explore-rs" --json -f "$root/examples/mma21p1panel/random_effects_gls_bootstrap.td" > regls_bootstrap.jsonl
Rscript "$root/examples/mma21p1panel/random_effects_gls_bootstrap_reference.R" "$work/MOM.dat" regls_bootstrap_reference
python3 "$root/examples/mma21p1panel/check_random_effects_gls_bootstrap_reference.py" . regls_bootstrap_reference "$work/MOM.dat"
```

R4.6.1/plm independently estimates theta, checks the original population/quasi
intercept scaling and slope, then uses independent Python group draws and R
lm/cov/type7 summaries. Full-source evidence:19 quantities (18 bootstrap plus
theta),95,760 identity/design/prediction cells, all5,320 loaded raw rows pinned;
text/JSON CSVs are byte-identical. Scalar/cell absolute tolerances are1e-9 and
theta is checked at1e-12. The inspectable shared panel fixture adds18 quantities
and432 cells across24 retained interleaved rows, plus four excluded raw rows.
Duplicate/bad/empty keys, imbalanced joint cases, degenerate within/between designs,
extreme theta failure and existing exports are tested. Only scripts, development
references/tests and status docs change; no production Rust/API/dependency change.

## Unbalanced RE-GLS sensitivity workflow

`random_effects_gls_unbalanced.td` fits Table21.2's log-hours on log-wage
random-intercept specification on the deterministic MOM sensitivity cohort. It
removes years1980/1985 for IDs divisible by3 and years1981–1983 for IDs divisible
by7 from the original ordered source rows. The cohort retains4,738 rows/532
people with unit counts5–10. This extends the balanced book analysis; it is not
an author result or a new published Table21.2 row.

Balanced Swamy–Arora components and strict balanced APIs remain unchanged.
Unbalanced components use a T_i-weighted between regression and trace correction
`(sum_i T_i ebar_i² - (G-p-1) sigma_e²) /
(N-tr((Z'WZ)^-1 Z'W²Z))`, with W=diag(T_i) and Z containing the intercept and
unit means. Quasi-demeaning uses one theta_i per unit and transforms
the intercept by1-theta_i. The final classical GLS covariance uses transformed
RSS/(N-K), K=p+1, not the preliminary within variance. Negative raw unit variance
is clipped at zero and retained diagnostically; singleton units are allowed when
within/between rank and residual degrees of freedom suffice. The result is
`UnbalancedRandomEffectsGlsResult` with retained count/theta ranges and null common
T/theta.

`cluster(id)` and the RE-specific `robust` alias use grouped quasi-demeaned scores
and multiplier G/(G-1)*(N-1)/(N-K). Coefficients and components are unchanged;
coefficient inference is asymptotic normal and slope Wald inference is
chi-square, not few-cluster t/F. Independent clusters and exogenous unit effects
remain assumptions. No variance-component uncertainty, robust ML, time-invariant
predictors, unit effects, general restrictions or automatic FE/RE bootstrap is
added.

R4.6.1/plm2.6.7 and explicit original-source equations validate **9,515 scalar
references** on the 4,738-row/532-person cohort. The native workflow checks all
retained source identities, inputs and population predictions/residuals in
**71,070 cells**. Classical and clustered covariance/inference, transformed fits,
unit-specific theta and weighted trace components are compared; explicit
`cluster(id)` and `robust` JSON results are identical. The original source hash
is pinned, and text/JSON data exports match byte-for-byte. Three synthetic
unequal/interleaved/boundary designs add **321 R/plm/matrix comparisons**.
Runtime tests cover joint-case counts, retained state, current-row prediction,
alias equality, target collisions and failed-refit data preservation. Tolerances
remain absolute1e-9/nonzero-tail relative1e-8; no author data or program is
bundled.

Reproduce from fresh directories containing prepared `MOM.csv`:

```sh
mkdir re_gls_json re_gls_text re_gls_reference
cp MOM.csv re_gls_json/MOM.csv
cp MOM.csv re_gls_text/MOM.csv
(cd re_gls_json && "$root/target/debug/tabdat-explore-rs" --json -f "$root/examples/mma21p1panel/random_effects_gls_unbalanced.td" > random_effects_gls_unbalanced.jsonl)
(cd re_gls_text && "$root/target/debug/tabdat-explore-rs" --text -f "$root/examples/mma21p1panel/random_effects_gls_unbalanced.td" > random_effects_gls_unbalanced.txt)
Rscript "$root/crates/tabdat-stats/tests/fixtures/random_effects_gls_unbalanced_reference.R" re_gls_reference/reference.csv re_gls_reference/inputs.csv "$work/MOM.dat"
python3 "$root/examples/mma21p1panel/check_random_effects_gls_unbalanced_reference.py" re_gls_json re_gls_text re_gls_reference "$work/MOM.dat"
```

This sensitivity estimate does not replace the published balanced estimate.
Original printed-Stata8 uncertainty differences, ML profile limitations, omitted
unit effects and broader panel-diagnostic gaps remain as described above.
