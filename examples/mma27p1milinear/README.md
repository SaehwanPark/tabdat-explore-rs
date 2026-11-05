# Chapter 27: complete cases versus single mean filling

## Scope and interpretation

For the source-generated data of §27.8.1 pp.936–937, this example compares the
`mma27p1milinear.do` column2–3 linear fits: complete-case OLS versus a single
predictor-wise mean fill followed by OLS. These are **generated simulation data**,
not empirical subjects. This is an instructive baseline, **not multiple
imputation**. The separate author SAS program uses10 normal-MCMC imputations,
500 burn-in iterations,1000 iterations between imputations and `MIANALYZE`
([SAS `NITER`/`NBITER` documentation](https://support.sas.com/documentation/cdl/en/statug/63347/HTML/default/statug_mi_sect002.htm));
that model, pooling and diagnostics remain required and are not implemented here.

Single mean filling reduces variance and can distort associations. The displayed
OLS t/SE/95% intervals after filling **treat the estimated fill means as fixed**
and omit uncertainty about missing values; they are not MI inference and are not
claimed generally valid or unbiased. Complete-case validity also depends on the
missingness mechanism and model assumptions. Neither comparison proves MCAR or
MAR, supplies a causal effect, or validates the author model's assumptions.
Classical coefficient inference otherwise uses homoskedastic nonrobust OLS,
Student-t with `n-3` residual degrees of freedom, and an intercept.

The author program explicitly states that its seed differs from the book tables.
The published ASCII files contain only `y x1missing x2missing`, not the true
complete predictors/errors: the original full-data column1 is unavailable here,
and neither historical book parity nor literal original internal-data parity is
claimed. The case4 overview says35% missing, but the executed program and
published input use25% per predictor; the observed input is authoritative.

## Sample and inputs

[Author program/data index](https://cameron.econ.ucdavis.edu/mmabook/mmaprograms.html),
`mma27p1milinear.do` (SHA256
`a63147ba522aff17d0b724554211cd109f251a4f1595bdd5fe7f4d923b374478`) and
its log (`5896a37748f219d62d9471a86b4a84dcb7d8bf65a67c2b423f2156de969d1186`)
pin the intended comparison. Download the files; they are not redistributed here.

| Case | Source SHA256 | Rows | Missing x1/x2 | Both missing | Complete cases |
| --- | --- | ---: | ---: | ---: | ---: |
| 1 `mma27linear1.asc` | `a2a58a2548096fff062d9c7ff3cb820785a39c70f55a3e34abe1eb5c256d7283` | 1000 | 100/100 | 12 | 812 |
| 2 `mma27linear2.asc` | `7f1662ba056298aafebcf72d0dbc54f49eac839df942de4cf033c527a7417bd2` | 1000 | 250/250 | 64 | 564 |
| 3 `mma27linear3.asc` | `d68cd8b77d0a244092959f4819179e330c5ff40d3ae48a3c983fe884db43d1c6` | 1000 | 100/100 | 12 | 812 |
| 4 `mma27linear4.asc` | `f727f69ade594b5d35a8f43769bad0566f5df1334e4fadad94298a5051ee924e` | 1000 | 250/250 | 64 | 564 |

`prepare.sh` validates exactly three whitespace-separated tokens **per physical
line**; `.` means missing for x only. It preserves numeric spellings/order and
assigns `source_row` from original row position. Outcome must be present; all
observed model values must be finite and at most1,000,000 in absolute value.
The script admits4–10,000 rows, unique integer `source_row`1..10,000 and at least
one observed value of each x. Before printing either fit, a conservative
centered-moment preflight requires each sample to have at least4 rows, positive
outcome/predictor population variances, `det(Var(x1,x2)) > 128*eps*Var(x1)*Var(x2)`
and population residual variance (from those centered moments) `>128*eps*Var(y)`.
The native QR fit can additionally reject rank/degree-of-freedom issues. This
workflow-specific relative guard rejects unresolved near-perfect fits rather than
printing huge t-statistics from rounding noise; it is not a universal OLS rule.
Native fit errors are not converted to missing estimates. Input is never overwritten.

Complete-case `y ~ 1+x1+x2` drops rows with either x missing. Each fill mean is
computed over **all rows where that predictor is observed**, not just the joint
complete subset; `y ~ 1+x1_filled+x2_filled` fits all admitted rows. For case1,
complete-case `(intercept,x1,x2)` is approximately `(0.894865,1.121366,0.922181)`;
the mean-filled fit is `(0.846752,1.187950,0.919134)`. These are descriptive
comparisons, not proof that one method recovers the unobserved complete data.

## Run, inspect and recover

From the checkout root, build the normal Rust binary and use a **fresh** work
directory. Replace the paths and case number as appropriate:

```sh
root="$PWD"; work=/path/to/fresh/work; source=/path/to/mma27linear1.asc
sh "$root/examples/mma27p1milinear/prepare.sh" "$source" "$work/missing.csv"
(cd "$work" && "$root/target/debug/tabdat-explore-rs" --json -f "$root/examples/mma27p1milinear/baseline.td" > out.jsonl)
Rscript "$root/examples/mma27p1milinear/reference.R" "$source" "$work/reference"
python3 "$root/examples/mma27p1milinear/check_reference.py" "$work" "$work/reference" "$source" --case 1 --author-log /path/to/mma27p1milinear.txt
```

Use `--text` in a separate fresh directory for readable output. JSON CLI-v1 and
text report sample/missing counts, both observed means, the uncertainty warning,
both retained-model fit reports/full covariance in JSON, and three Student-t
coefficient `lincom` intervals per fit. `missing_comparison_predictions.csv`
contains ordered original fields, missing indicators, filled x, both fitted means
and residuals. Complete-case predictions/residuals are NULL wherever either x is
missing. The final active table is `missing_comparison_predictions`; the retained
model is **the second, filled-predictor OLS fit**, not an MI model. Generic later
prediction uses `x1_filled` and `x2_filled`.

Converter publication uses a same-directory temporary file and no-clobber hard
link: a failed conversion does not publish partial output or replace an owner.
The script's export also refuses to overwrite. A collision occurs **after** fits,
predictions and session effects; it is not a whole-script transaction. Use a
fresh output path/directory to recover. Guard failures can surface as generic
`sql failed`; inspect the source shape, finite values, missingness and rank.

## Independent evidence and remaining work

Base R4.6.2 QR `lm`/`vcov`/`predict`, using the exact published ASCII values and
independent sample/fill calculations, checks each original case:96 fit,
covariance, t and interval fields;9 diagnostics; all1,000 ordered source,
missingness, filled-value and prediction rows. The Python audit binds all3,000
parsed-f64 values/missing markers per case to the SHA-pinned input (numeric
identity, not exported decimal spelling) and independently recomputes counts
and observed means. Across cases, native/R maximum absolute
errors are4.55e-12 for reported fit values and5.15e-14 for row projections.
Fixed tolerances are absolute1e-9, nonzero p-value relative1e-8, and exact zero
for underflow. No numerical tolerance was widened. Original text/JSON CSV exports
are byte-identical for all four cases.

The optional pinned-log audit compares48 printed coefficients/SEs descriptively:
maximum absolute difference4.83e-7, with47/48 matching after four-decimal
rounding. Case1 mean-filled x1 is a rounding-boundary exception: log `1.18795`,
ASCII-refit `1.1879498647`. The author log uses data before rounded ASCII export;
that is a plausible precision contributor, not proof of every historical
rounding mechanism. Neither the47/48 comparison nor source-program output is
used in place of the strict independent raw-ASCII R oracle.

Offline mixed/fully-observed controls have independently generated R reference
fixtures; both observed means differ from their joint-complete means in the mixed
control. Actual-script tests exercise malformed/nonfinite/missing outcome,
all-missing predictor, incomplete/rank-limited and perfect-fit guards before
inference, duplicate keys, output collisions, retained-model predictions, JSON/text parity, forged fit/covariance/
source/fill/prediction fields and shared-wrong source data. Python and R are
development-only; production has no dependency on them.

The author's full-data comparison, MCMC multiple-imputation model, between/within
variance pooling/finite-df inference, imputation diagnostics, logit comparison,
other chapter examples and full textbook inventory remain required.
