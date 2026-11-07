# Individual variable-wise source summaries — bounded contract

**In progress, not yet validated.** I-DESC, companion program lines71–74,
Table24.3 individual portion p.850. Independent of the selected Poisson/model
cohort, household summaries and unresolved historical ML inference.

## Outcome / sample / interpretation

Native load/audit → typed preparation → per-variable summary/interpretation →
report/export/reproduce for source order:
`pharvis lnhhexp age sex married educ illness injury illdays actdays insurance commune`.

Use the individual README's pinned numeric-code CSV `vietnam_individual.csv`.
Original27766 source rows include source key27734 with **all twelve fields NULL**.
Keep that row: original per-variable N27765 is not source N27766. Partially missing
synthetic rows are also retained, with independently available samples, not a
joint/listwise filter. Zeros remain; sample SD uses N−1, N0/N1 SD is undefined.
No fit, sampling weight, survey/causal inference, confidence interval or imputation.

The source renames `lnhhinc` to LNHHEXP; preserve the numeric column without
asserting income versus expenditure or generating an undocumented level measure.
AGE values resemble logged ages; do not transform or interpret as years. EDUC/
ILLNESS and other numeric codes are not factor-expanded; ILLNESS is not assumed
binary. Commune numeric-code moments are not location or cluster-size statistics.

Raw input1..40000 rows. Require unique positive BIGINT source keys <=2^53−1.
PHARVIS is nullable nonnegative BIGINT <=2^53−1; commune is nullable positive
BIGINT <=2^53−1. All-NULL CSV columns are allowed despite import type inference.
The remaining ten measurements may be NULL or must cast to finite DOUBLE.
Prepare explicit BIGINT counts/keys and DOUBLE measurements, sorting by source
key while preserving every record. Reject malformed/nonfinite data rather than
silently changing memberships. Native finite-arithmetic aggregate overflow may
still fail; no universal numerical-stability claim.

Export all prepared source records and a twelve-row
`variable,count,mean,std_dev,minimum,maximum` report; reactivate the prepared source
for later explicit modeling. Native summarize is read-only; the whole load/
prepare/report script is ordered, not transactional. Existing file owners are
never overwritten; earlier successful effects/files need not roll back.

## Acceptance / important failures before implementation

1. Both original modes:60 independent base-R fields, all27766 source/prepared
   records and exact per-variable membership, including retained blank row.
   Separate precise-DTA author-log half-printed-digit gates for all60 fields;
   investigate disagreements, never widen fixed absolute1e-9 tolerances.
2. Small frozen controls with partially/wholly missing rows, different variable
   Ns, retained zero, N0/N1 variables and a one-row wholly missing source. No
   Poisson rank/two-cluster/positive-count requirements on descriptive data.
3. Actual script JSON/text typed ordered summaries and byte-identical exports,
   sorted-source reactivation, source bytes intact, read-only labels/model state
   separately from deliberate load/preparation; successful export and both
   collision boundaries preserve existing file owners.
4. Raw row permutations preserve summaries/prepared-key order. Targeted source,
   report, count, membership and shared native/R statistic forgeries must fail
   independent audit; no production R/Python dependency or new framework/API.
5. Empty/duplicate/invalid or inexact keys/counts, nonnumeric/nonfinite measurements
   and oversized input fail before summaries/exports, not merely because an
   output collision hides a validation miss. Disclose partial state effects.

## Required limits / verification

The author acknowledges a printed-book LNHHEXP difference and missing MEDEXP.
This slice validates the companion source summary, not literal whole-Table24.3
parity. Keep those gaps, I-DETAIL/I-FREQ/I-HIST and named comparisons required;
no chapter/book completion or user-scope exclusion.

Follow existing household descriptive patterns without changing its supported
behavior. New script/control/forgery tests first, then affected root crate and
relevant integration checks, scoped Clippy/fmt/diff and final PR review. No
Rust documentation examples or statistical-kernel changes intended. If essential
source evidence cannot be obtained, record it and request a decision rather
than pretending that an unresolved required entry is complete.
