# MMA chapter 21: pooled, first-difference and within-FE hours/wages analyses

**Partial empirical coverage.** Cameron–Trivedi (2005), §21.3 pp.709–714,
Table21.2 p.710: POLS/first-difference and Within slope/conditional-prediction
portions of the March2005 author
[`mma21p1panfeandre.do`](https://cameron.econ.ucdavis.edu/mmabook/mma21p1panfeandre.do)
and [output](https://cameron.econ.ucdavis.edu/mmabook/mma21p1panfeandre.txt).
The header's “Table21.1” is inconsistent with the explicit Table21.2 labels in
the POLS/FD sections and final table block; the latter identifies this slice.
This is linear-panel chapter **21**, not clustered-sample chapter24.

POLS/FD are **composed native SQL/OLS workflows**. `within.td` executes bounded
native `xtreg ..., fe id(id) time(year)`. All load, guard keys, fit, test the wage
slope, compute its 95% interval, summarize predictions/residuals and export usable
data through text/CLI-v1/scripts. POLS/FD compare three immutable named reports;
FE reports classical and unit-cluster results with a new `FixedEffectsResult`,
not `ols`. Persistent `panel`/`xtdata`, RE and whole-table coverage remain absent.

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

**Still required:** between, RE-GLS/RE-MLE, Hausman and panel-specific
residual/model-comparison diagnostics, bootstrap/RNG, optional richer designs,
lowess/Figures21.1–21.4, remaining Table21.2/book outputs and other panel
illustrations. This slice completes neither the panel family nor MMA-PANEL.

## Between portion under development

The next bounded slice reproduces Table21.2's Between classical coefficients
and inference using one equally weighted joint-finite person-mean row per ID,
with an intercept and wage slope. It will add an independent-person HC1
comparison, retained named reports, mean-response predictions/residuals and an
export that audits contributing row counts and year ranges. Raw person/year
keys must pass the existing guards before model-case selection. The original
balanced sample has 532 mean rows and residual/inference DF530. Modified input
will use jointly finite outcome/wage pairs before centering, retain singleton
people, exclude people with no usable pairs, and never weight means by T_i.
Independent R lm/sandwich and small gapped/missing/error CLI cases will verify
this contract. RE, bootstrap and Figure21.2 remain outside this slice.
