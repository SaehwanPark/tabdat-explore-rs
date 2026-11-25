# Chapter 21 source-data and panel summaries

## Bounded objective

Reproduce the data-inspection block in the March 2005 `mma21p1panfeandre.do`
program (lines 80–96): year frequencies, the 18-variable inventory and summary,
the balanced panel structure, and overall/between/within summaries for
`lnhr lnwg kids ageh agesq disab`. This adds an executable source workflow; it
uses the reusable [`xtsum` command](../../docs/commands/xtsum.md) for those six
variables, but does not add persistent panel state or claim a general `xtdes`
command.

## Input contract

Use the original eight-column `MOM.dat` source and the repository's strict
`prepare.sh` conversion to `MOM.csv`. Retain all 5,320 rows and require unique
integer `id`/`year` keys: IDs 1–532, years 1979–1988, and ten consecutive years
for every person. All six `xtsum` variables must be finite. The workflow must
stop if its source cohort or balanced-panel structure differs.

Generated `dyear1`–`dyear10` indicators are in ascending year order, matching
the author script. The converter's `source_row` is checked as provenance, then
excluded from the 18-variable inventory. CSV-imported numeric types are not
Stata DTA storage types; this workflow audits variable names and empirical
summaries, not literal DTA metadata or `save mom.dta` behavior.

## Acceptance evidence

- Compare all year counts, source and generated-variable summaries, panel count,
  time span, per-person observation-count distribution and six `xtsum` rows
  against a Base R reference from the pinned original source.
- Require the native rows to use the stable `XtSumResult` fields and preserve
  the requested six-variable order; compare N/n/T and every reported moment or
  extremum against Base R, then compare the float32 path with the author log.
- Compare the rounded values and panel facts with the authors' published May
  2005 log. Keep native float64 values distinct from the author's float32
  display values.
- Exercise the actual script on the complete source and small fixtures for
  duplicate keys, missing years, extra years, nonfinite analysis fields and
  wrong row counts. Fail before exporting partial summary artifacts.
- Preserve the existing panel preparation and model scripts unchanged.

Independent source hashes and current inventory status are in
[`inventory.md`](inventory.md#source-identity-and-status-rules).
