# Chapter 27 supplied-imputation marginal pooling

## Meaning and scope

This example pools **already-fitted per-imputation estimates and marginal standard
errors** for `intercept`, `x1missing`, `x2missing` using the large-complete-sample
Rubin rules in the Chapter27.8.1 `mma27p2milinear.sas` / `.lst` case1 analysis.
The author performed10 normal-MCMC imputations and separate linear regressions.
The public log gives the individual estimates/SEs only to five decimal places;
the actual completed data and unrounded regression fits are not available here.
The inputs are simulation results, not empirical subjects, and the author states
its seed differs from the book. No author data or log is redistributed.

**This does not create imputations, fit their regressions, validate MAR, or
supply a general MI command.** Validity requires suitable proper multiple
imputations, congenial analysis models and appropriate missingness assumptions.
Marginal SEs cannot recover cross-parameter covariances, joint Wald tests or
prediction uncertainty. Complete-data DF corrections (including Barnard–Rubin),
imputation diagnostics and the full Chapter27 illustration remain required.
The separate [single-mean-fill comparison](README.md) is not MI.

## Formula and admissible inputs

For m estimates Q_j and within-imputation variances U_j=SE_j², use
`Qbar=mean(Q_j)`, `W=mean(U_j)`, `B=sum((Q_j-Qbar)^2)/(m-1)` and
`T=W+(1+1/m)*B`. For B>0, `r=(1+1/m)*B/W` and the **original Rubin**
large-complete-sample `df=(m-1)*(1+1/r)^2`; report `Qbar/sqrt(T)`, a two-sided
Student-t p, and t95 interval. The missing-information estimate is
`(r+2/(df+3))/(1+r)` and relative efficiency is `1/(1+FMI/m)`. Do not confuse
the large-complete-DF reference with a finite-complete-DF correction.

For exactly zero B, use the normal infinite-DF limit, FMI0 and efficiency1;
DF is NULL, explicitly labeled `normal_infinite_df_limit`. For positive B, reject
finite DF above1,000,000 rather than silently normal-approximate. The script
accepts2–100 contiguous integer imputation keys `1..m`, exactly the same three
parameter names and one row for every key/parameter; unique key pairs are
required. The CSV key column must load as BIGINT, so use integer lexical forms
(not decimal/exponent forms); this rejects fractional values that would otherwise
round to an integer when parsed as binary floating point. Estimates must be finite with magnitude at most1,000,000; supplied SEs
must be finite and within `1e-12..1e6`. Total variance and the resulting statistic
must be finite; `|statistic|<=1e6`. Missing/invalid values or unresolved inference
fail before export; no imputations or failed results are silently dropped.

Two native DuckDB scalar functions support the fractional DF required here:
`tabdat_t_pvalue(t,df)` is the **two-sided** Student-t tail;
`tabdat_t_quantile(probability,df)` is its quantile. Both use the existing Rust
stats implementation. They return SQL NULL for NULL/nonfinite/out-of-bound inputs
or nonfinite results; arguments are DOUBLE-coercible, `1<=df<=1e6`, `|t|<=1e6`,
and `0.025<=probability<=0.975`. These deliberate SQL boundaries are not a
claim about every extreme-tail or arbitrary-DF use. A normal limit for B=0 uses
`tabdat_gauss_phi`, not a made-up finite DF.

## Reproduction, output and state

Download `mma27p2milinear.lst` from the [author index](https://cameron.econ.ucdavis.edu/mmabook/mmaprograms.html)
and use a fresh output directory. The source log SHA256 is
`0d0633bf3758205423cd909fb20a6b05c6c19ed729eca35e05ad7ce379ef7074`;
the author SAS program SHA256 is
`56f85ecf888cf2c878d9df4f2015fcdb98d7df19d4e2799b087027ceeb637f8b`.

```sh
root="$PWD"; work=/path/to/fresh/work; log=/path/to/mma27p2milinear.lst
sh "$root/examples/mma27p1milinear/prepare_pooling.sh" "$log" "$work/estimates.csv"
(cd "$work" && "$root/target/debug/tabdat-explore-rs" --json -f "$root/examples/mma27p1milinear/rubin_pooling.td" > out.jsonl)
Rscript "$root/examples/mma27p1milinear/rubin_reference.R" "$work/estimates.csv" "$work/reference"
python3 "$root/examples/mma27p1milinear/check_rubin.py" "$work" "$work/reference" --author-log "$log"
```

The extractor validates ten complete blocks and publishes via a temporary file
and no-clobber link; the checker additionally verifies the log hash and all30
ordered printed estimate/SE pairs. Users can instead supply their own rectangular
`estimates.csv` with columns `imputation,parameter,estimate,standard_error` and
use `--synthetic` only for an explicitly unpinned independent control. Use
`--text` in a separate fresh directory for readable results. Both modes output
`rubin_marginal_report.csv` and a typed report with estimates, all variance
components, df/reference, t/normal inference, FMI, efficiency and min/max.

The final active table is `rubin_marginal_report`; pooling creates **no retained
estimation model**. A prior model, if any, remains retained and is not the pooled
result; returning to compatible data before using its `predict` is necessary.
The converter and export do not overwrite owners. Export collisions occur after
SQL/session effects, so the script is not transactional. Some guards surface as
generic `sql failed`; check key rectangularity, variance and DF bounds.

## Evidence and limits

R4.6.2 independently computes Rubin formulas, `pt` and `qt` on the exact rounded
log input. All3 reported marginal rows and48 numeric fields agree at absolute
1e-9, nonzero-p relative1e-8 and exact zero on underflow; maximum absolute native/R
difference is3.35e-10 (large DF). JSON report and CSV match; text/JSON CSV is
byte-identical. The log's three printed **unrounded-fit** pooled estimates differ
by +1e-6,+1e-6 and −2e-6 from pooling its rounded inputs; pooled SE differences
are below9e-7 and DF differences below0.008. These descriptive comparisons are
not the strict R oracle or proof of literal SAS/RNG replication.

Independent R synthetic controls cover positive and zero B, the normal limit,
zero-underflow p and two imputations with small fractional t DF. Native SQL tests
compare R t tails/quantiles at DF1,3,172.3775,33662 and1,000,000, check NULL,
invalid and multichunk paths. Actual CLI tests cover malformed/duplicate log
blocks, extraction/export collisions, unbalanced/duplicate keys and decimal keys
that would round to integers, bad SEs, excessive DF/t, source/JSON/report
forgeries, preservation of a prior OLS model,
and text/JSON parity. R and Python are reference/audit tools only, not production
dependencies. The author MCMC generation, completed-data fits, cross-covariance,
finite-complete-DF inference and the full textbook inventory remain open.
