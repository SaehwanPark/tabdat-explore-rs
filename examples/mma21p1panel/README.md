# MMA chapter 21: pooled and first-difference hours/wages analyses

**Partial empirical coverage.** Cameron–Trivedi (2005), §21.3 pp.709–714,
Table21.2 p.710: POLS and first-difference portions of the March2005 author
[`mma21p1panfeandre.do`](https://cameron.econ.ucdavis.edu/mmabook/mma21p1panfeandre.do)
and [output](https://cameron.econ.ucdavis.edu/mmabook/mma21p1panfeandre.txt).
The header's “Table21.1” is inconsistent with the explicit Table21.2 labels in
the POLS/FD sections and final table block; the latter identifies this slice.
This is linear-panel chapter **21**, not clustered-sample chapter24.

These are **composed native SQL/OLS workflows**, not new `panel`, `xtdata` or
`xtreg` execution. Each script loads, guards keys, fits three covariance choices,
tests the wage slope, computes its 95% interval, compares immutable named reports,
summarizes fitted/residual values and exports usable data. Existing CLI-v1 result
types and inference metadata are unchanged; the fitted estimator reports `ols`.

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
- Both scripts reject duplicate/missing person-year keys before fitting and check
  nonempty numeric integer IDs1–532/years1979–1988 without altering source data.
  Unsorted/interleaved rows work; gaps are not bridged and no lag crosses people.
  Missing model values propagate through differences before finite-case fitting;
  exported adjacent-pair rows need not all be estimation rows on modified input.
  `RegressionResult.sample` distinguishes relation rows from dropped model cases.
  No panel declaration is retained and no general calendar/lag DSL is claimed.

## Inference and interpretation

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

**Still required:** within-FE, between, RE-GLS/RE-MLE, Hausman and panel-specific
residual/model-comparison diagnostics, bootstrap/RNG, optional richer designs,
lowess/Figures21.1–21.4, remaining Table21.2/book outputs and other panel
illustrations. This slice completes neither the panel family nor MMA-PANEL.
