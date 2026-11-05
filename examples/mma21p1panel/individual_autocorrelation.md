# Individual log-hours lag-one correlations

[individual_autocorrelation.td](individual_autocorrelation.td) implements the
bounded individual portion of Cameron–Trivedi (2005) §21.3.4, author
[residual program](residual_inventory.md) lines142–156. For each person it
correlates current `lnhr` in1980–1988 with that same person's previous calendar
year, giving **nine time pairs per person**. This differs from correlating people
across two years. It reproduces the two correlations actually printed for people1
and2 and generalizes the author's “and so on” to all available people. These are
descriptive sample correlations, **not** a formal serial-correlation test,
covariance correction or causal conclusion. The author does not compute wage
correlations in this block, and neither does this workflow; there is no fit or
model state here.

## Sample and outputs

The original MOM.dat has532 complete people x10 years1979–1988. This bounded
workflow requires3..532 complete ten-year people; unique exact integer source_row,
id and year in1..5320,1..532 and1979..1988 respectively; and finite lnhr/lnwg
bounded by abs20. No missing-value deletion, weights, imputation or gaps. SQL
`LAG` partitions by id and orders by year, keeping explicitly consecutive years;
it never bridges people or missing years. The first export contains4,788 original
annual/source pairs in `(id,year)` order, including current source fields and
previous source row/year/lnhr. Source keys preserve original ordering even for
interleaved input. The second export and displayed table contain one ordered row
per person `(id,pairs,correlation)`; each pair count is nine.

For each person the current and lagged nine-value series must each have finite
population SD>1e-12, and correlation must be finite and within the numeric range.
This explicit absolute resolution bound is conservative, not a significance test;
constant and unresolved series fail, not return zero. The query subtracts the
minimum of each person/series before variance and correlation: Pearson r is
translation-invariant, and this avoids cancellation for nearly constant levels.
On dyadic controls with log-hours approximately7 and time steps1e-9 or1e-8,
uncentered DuckDB correlation differed from R by3.70e-9 and2.96e-9; the centered
query matched independent R within8.89e-16 across steps1e-11 through1e-7. A
separate step1e-13 control fails the declared resolution guard. This is not a
promise of numerical accuracy for every possible ill-conditioned dataset.

All people and correlations are preflighted before either export. SQL failures
currently surface generic `sql failed`; the preceding scope note states the
within-person variance requirement. Fix the data or choose another specified
analysis if a person's correlation is undefined. Use a fresh directory: exports
never overwrite existing owners or source, but the two-export script is not a
transaction. A second-target collision leaves the first export; remove or rename
it before retrying. This is not a general unbalanced panel or autocorrelation API.

## Independent and historical evidence

The development-only [R4.6.2 reference](individual_autocorrelation_reference.R)
sorts raw MOM by person and year, explicitly pairs adjacent values, and calls R
`cor` separately for each person. The
[auditor](check_individual_autocorrelation.py) checks all4,788x12 source/annual
cells exactly and every one of532 person/table/export correlations at absolute
1e-9 (observed max6.39e-16), including pair counts/order. The author's printed
person1/person2 correlations match to four decimal places (max absolute
difference0.00002008), separately from the tighter R tolerance. Original source
and log hashes are pinned. Printed agreement does not prove historical internal
precision.

The author program reads MOM.dat, fits pooled/FE/RE residuals and saves `mom3`
**before** the later FD reload; the individual block reloads that saved state
and uses its unchanged lnhr. This differs from the FD block's separately loaded
external `mom` whose binary identity is unknown. The author reshape actually
creates person-suffixed variables despite its misleading year-suffix comment;
the executed `corr lnhr1 L.lnhr1` and `lnhr2` establish the within-person target.

The [interleaved dyadic16-person control](fd_residual_correlations_synthetic.py)
adds144x12 source/annual cells and16 independent R correlations. On this control,
person1's within-time r is0.782392 while the R across-person1979/1980 r is
0.485249; a test explicitly distinguishes them. Four
[actual-script tests](../../tests/mma21_individual_autocorrelation_workflow.rs)
cover text/JSON parity, source/lag identities, current/lag constant and resolution
failures, invalid keys/missing data and both collision paths including partial
first output. Eight forged table/note/export/raw/previous/id/pair-count/correlation
traces were rejected. Original text/JSON exports are byte-identical. Publisher
data/program are not redistributed; R/Python are not production dependencies.

## Reproduce

From repo root with a current binary (`cargo build --locked`), locally obtained
publisher MOM.dat and author output, and R for independent reference only:

```sh
repo=$PWD
raw=/absolute/path/to/MOM.dat
log=/absolute/path/to/mma21p3panresiduals.txt
work=$(mktemp -d)
sh examples/mma21p1panel/prepare.sh "$raw" "$work/MOM.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/mma21p1panel/individual_autocorrelation.td" > individual_autocorrelation.jsonl)
Rscript examples/mma21p1panel/individual_autocorrelation_reference.R "$raw" "$work/reference"
python3 examples/mma21p1panel/check_individual_autocorrelation.py "$work" "$work/reference" "$raw" --author-log "$log"
```

Use a separate fresh directory and `--text` for readable output. Converter/export
refuse existing targets; R requires a fresh reference directory and raw eight-
column MOM.dat, not prepared MOM.csv. Synthetic audit uses `--synthetic`, waiving
original input hash and author-log requirement only. Other source-program blocks,
the companion inventory and complete chapter/book coverage remain open.
