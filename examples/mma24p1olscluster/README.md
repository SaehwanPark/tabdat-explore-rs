# Chapter 24 household OLS cluster comparison

This bounded workflow covers Cameron–Trivedi (2005) §24.7 Table 24.4,
OLS columns 1–4: intercept regression of `LNEXP12M` on `LNHHEXP AGE SEX
HHSIZE FARM EDUC`, comparing IID, HC1 and commune-clustered CR1 covariance.
The final `lnhhexp` test and confidence interval use the retained cluster fit.
These are alternative uncertainty assumptions for the same OLS coefficients;
they are not causal effects or a complex-survey design analysis. In particular,
cluster independence is an assumption, not established by this output. As in
the author regression, EDUC and FARM enter as their numeric codes, not as
one-hot factors; SEX retains its original 1/2 coding.
A separate [commune fixed-effects workflow](household_fe.md) covers the
household FE columns. A separate [commune RE-GLS workflow](household_re_gls.md)
validates native unbalanced Swamy–Arora estimation, **not** the author's different
RE component convention. Author-style RE, GEE, bootstrap, and survey weights/strata
remain required; the [individual pooled Poisson workflow](../mma24p2poiscluster/README.md)
covers a separate portion of Table 24.6.

## Source and preparation

Download publisher [`vietnam_ex1.asc`](https://cameron.econ.ucdavis.edu/mmabook/vietnam_ex1.asc)
(SHA256 `74cb37e8c14609e382d334e368a54f63bccd4e952d02bcbf535baf19115d4061`).
It has 5,999 ordered records across physical line breaks, with quoted FARM and
EDUC value labels, not numeric codes. [`prepare.py`](prepare.py) uses the
standard Python library once to verify the source hash and decode these labels
to their actual numeric values; it refuses to replace any existing destination.
The mapping was checked against the numeric value-label metadata in publisher
[`vietnam_ex1.dta`](https://cameron.econ.ucdavis.edu/mmabook/vietnam_ex1.dta),
SHA256 `0ea6639f93a5bc3f53bcbd26b6865ad6908a04f41ab04d90e7c9aa4b7e9787bf`.
The data and publisher code are not redistributed. Alternatively, supply a CSV
with the same header, numeric category values and stable integer `source_row`;
Python and R are not needed by the Rust runtime.

From an otherwise empty work directory, with `repo` set to the repository path:

```sh
python3 "$repo/examples/mma24p1olscluster/prepare.py" /path/to/vietnam_ex1.asc vietnam.csv
"$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma24p1olscluster/household_ols.td" > run.jsonl
```

Build the binary first if necessary. `--text` provides the readable interface;
the same ordered prediction CSV is produced in either mode. The script validates
unique, positive, BIGINT source-row and commune keys and finite required
predictors. A nonmissing nonfinite outcome is invalid; a NULL outcome is
excluded. Source key types are intentional: loading a decimal key as floating
point can round a noninteger into an integer. The publisher sample excludes 993
missing outcomes but retains log-outcome zero, leaving 5,006 households in 194
communes. `source_row` identifies a source record, not a household identifier
in the original survey. The script also accepts smaller valid numeric CSVs,
but rejects missing required predictors, duplicate/invalid keys, singular
design, insufficient residual degrees of freedom and fewer than two clusters.
SQL guard failures can report only `sql failed`.

Classical covariance uses `RSS/(N-K)`; HC1 is `N/(N-K)` times the row-score
sandwich. CR1 uses `G/(G-1)*(N-1)/(N-K)` times the commune-score sandwich.
Here K=7 includes the intercept; IID and HC1 t inference uses 4,999 DF, while
CR1 t/one-constraint F inference uses 193 DF. The regression result's residual
DF remains 4,999; the named estimates table identifies inference DF and the
number of clusters. None of these adjustments accounts for additional survey
stratification, weighting or design features. A non-rejection or small SE does
not establish the relevant assumptions.

`household_ols_predictions.csv` contains the complete-case source fields,
fitted values and residuals in source-row order. It is created without clobber.
The final retained model is the commune-clustered OLS model; it replaces any
prior model. This multi-command script is not transactional: an export collision
preserves the existing output owner and source, but prior model/table changes
and output events may already have occurred. Use an empty work directory and
remove an old export deliberately before rerunning.

## Independent evidence and reproduction

R 4.6.2's base QR and separately constructed row/commune-score sandwiches
validate all 240 coefficient, full covariance, SE, t, p-value, R², adjusted R²
and root-MSE fields, plus eight clustered test/linear-combination/interval
fields. Absolute tolerance is `1e-9`; nonzero p-values use relative `1e-8`,
and an underflowed reference zero requires exact zero. Every field of all 5,999
prepared source rows is tied to the pinned ASCII and independently parsed R
source; all 5,006 retained rows and fitted/residual values are checked against R.
The two actual interface exports are byte-identical. Synthetic R-reference
fixtures also exercise the positive zero outcome, missing-row exclusion, all
three covariance matrices, small cluster DF and prediction order. Integration
tests cover duplicate/invalid/rounded-decimal keys, nonfinite inputs, singular
design, one cluster, source integrity and export-owner collision. Auditor
forgery checks altered source, coefficient, covariance, fitted/residual cells,
and dropped rows.

To repeat the original-source numerical audit (R and Python are development
reference/audit tools, not runtime dependencies):

```sh
Rscript "$repo/examples/mma24p1olscluster/reference.R" /path/to/vietnam_ex1.asc reference
python3 "$repo/examples/mma24p1olscluster/check_reference.py" . reference /path/to/vietnam_ex1.asc
```

The publisher's [program](https://cameron.econ.ucdavis.edu/mmabook/mma24p1olscluster.do)
SHA256 `74672f14199ad5925594132e223887db61eb35752624a567818f433e4b38b2d3`
and [log](https://cameron.econ.ucdavis.edu/mmabook/mma24p1olscluster.txt)
SHA256 `5e74c8de58eaf833b0070d68b289d61fad43831f3eb8b57c06818a6bf549837e`
use the DTA's underlying binary floats rather than the rounded ASCII. In a
separate investigation, R `foreign::read.dta` plus the same independent QR and
sandwich formulas matched all 42 displayed OLS coefficient/SE fields within
`5e-8`. All category codes, keys and missingness agree row-by-row; maximum
DTA/ASCII numeric difference is `4.99695e-6`. This explains why original
DTA-based author figures are not the strict oracle for the ASCII workflow;
no native/ASCII tolerance was widened and binary identity is not claimed.
Optional reproduction, in a fresh directory containing `vietnam.csv`:

```sh
Rscript "$repo/examples/mma24p1olscluster/dta_reference.R" /path/to/vietnam_ex1.dta dta.csv
Rscript "$repo/examples/mma24p1olscluster/reference.R" dta.csv dta-reference --csv
python3 "$repo/examples/mma24p1olscluster/check_author.py" /path/to/vietnam_ex1.dta /path/to/mma24p1olscluster.txt dta.csv dta-reference vietnam.csv
```

Reference directories are development outputs and existing reference files
may be overwritten. The DTA converter refuses an existing destination.
