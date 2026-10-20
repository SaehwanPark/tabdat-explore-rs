# MMA chapter 21: pooled, between, first-difference within-FE and RE-GLS hours/wages analyses

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
`panel`/`xtdata`, RE-ML and whole-table coverage remain absent.

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

**Still required:** robust/unbalanced RE-GLS, RE-MLE, Hausman and panel-specific
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

The exact original design is lnhr~1+lnwg on all5,320 rows/532 people/T10.
General bounded execution audits raw unit/time keys before jointly finite y/X
selection. Remaining groups must have equal counts T>=2; G>p+1; both within and
between preliminary designs must identify every slope. Entirely excluded units
are omitted. Interleaved/gapped data are allowed; equal counts do not promise
common years. Time-invariant predictors, weights, unbalanced variance estimation,
robust/cluster covariance and alternative component estimators are not supplied.

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

Native float64 wage slope0.1193322191 has SE0.0136311969 (normal inference);
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

Still required: robust/cluster and unbalanced RE, unit effects/conditional
predictions, RE-ML, Hausman and other panel diagnostics, bootstrap and remaining
table/figure outputs. This bounded classical RE-GLS portion does not complete
the column/table, panel family or textbook target.
