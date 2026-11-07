# Household variable-wise source summaries — bounded contract

**In progress, not yet validated.** H-DESC, companion program lines69–72,
Table24.3 household portion p.850. This is independent of the unresolved
historical ML SE investigation; it does not change estimation or close that gap.

## Outcome and conventions

Deliver a native load/audit → prepare → summarize/interpret → export/reproduce
workflow for the nine variables in source order:
`lnexp12m age sex hhsize farm educ hhexp lnhhexp commune`.

Use the pinned household README's ASCII or precise numeric-code DTA input,
`vietnam.csv`, with its original nine source columns. Original5999 source rows;
outcome has5006 observations, other source variables have5999. **Do not select
the joint estimation cohort before describing the data.** Each variable has its
own finite available-row sample. Zeros remain; sample SD uses N−1 and is undefined
below2 observations. All-missing variables report N0/NA, not empty-data failure.
There is no model, sampling weight, survey/causal inference or confidence interval.

Preparation explicitly normalizes the seven measurement columns to DOUBLE,
retains positive BIGINT source/commune keys (<=2^53−1 for exact independent R
identity checks), and orders rows by unique source-row key. Nullable measurements
are allowed, including entirely missing CSV columns; nonnumeric text/nonfinite
nonmissing measurements fail. Raw input must contain1..40000 rows. Generate
`hhexp=exp(lnhhexp)` as **DOUBLE**, not a purported original level expenditure
column. A nonmissing generated level must remain positive/finite; exponential
underflow/overflow fails. This deliberately uses native double precision instead
of the source's default Stata float storage for `gen HHEXP`; independent R uses
the same declared double convention. No input file is overwritten.

Use the existing native ordered `summarize` result and a nine-row CSV report
with `variable,count,mean,std_dev,minimum,maximum`. Also export the complete
prepared source, including every generated HHEXP value. Reactivate that source
at the end so subsequent modeling can explicitly choose its own cohort. The
whole script is ordered, not transactional; an existing export owner is never
overwritten and earlier successful effects/files are not rolled back.

## Acceptance and failure cases (before implementation)

1. Both original modes: validate all5999 source/prepared rows, exact fields/keys/
   order/membership, and45 independent R count/mean/sample-SD/min/max quantities.
   Keep author-log rounding gates separate; audit all45 printed facts on precise
   DTA without widening tolerances or asserting literal printed-book parity.
2. Small independently frozen controls: variable-wise missingness (different
   membership/counts), retained zeros, all-missing and singleton-variable SD/NA;
   a one-row source must work without unnecessary panel-model sample bounds.
3. Actual script JSON/text: typed ordered summaries, both exports byte-identical,
   final prepared-source ownership/order, source bytes unchanged. Inspect-read-only
   summary behavior separately from the script's deliberate load/preparation.
4. Permuted raw rows still yield source-key-ordered prepared records and the same
   summaries. Forge source/generated values, report counts/statistics/order and
   cohort membership; independent audit must reject targeted corruptions.
5. Empty/duplicate/wrong-type/nonpositive/inexact keys, nonnumeric/nonfinite
   measurements, failed EXP and export collisions fail without overwriting source
   or pre-existing output owners. Describe partial effects honestly.

Independent development-only base-R reference; no production dependency or
new framework/API. Fixed absolute1e-9 statistics tolerance, exact counts/source
fields and separate half-printed-digit author gates. Native finite-arithmetic
aggregate overflow may still fail rather than claim general numerical stability.

## Scope / stop conditions

The author explicitly notes differences from printed Table24.3 and substitutes
FARM for URBAN. This slice validates the **companion household summary workflow**,
not missing URBAN or literal full-table/chapter/book coverage. Keep those source/
book distinctions, individual summaries and all other inventory gaps required.
If an author gate differs under declared precision, investigate and document it;
do not replace values or widen tolerance to manufacture agreement. Reassess
scope if an unavailable essential source or new public contract is required.

## Verification plan

Inspect the diff before testing. Run the new script/control and forgery tests
first; expand to root integration scope and shared source-auditor regressions.
Use existing frozen small fixtures, R original-source audits, scoped Clippy,
fmt/diff checks, then final review/PR evidence before merge and branch cleanup.
No Rust documentation examples or estimator formula changes are intended.
