# `xtdes`: explicit-key panel structure

## Objective and reference

Provide a reusable, read-only panel-structure diagnostic after `xtsum`. The
command describes the observed panel/time key structure and participation
patterns; it does not set or retain panel state and does not estimate a model.
The definitions are informed by Stata's [`xtdescribe` manual](https://www.stata.com/manuals/xtxtdescribe.pdf),
which describes the panel and time ranges, distribution of observations per
panel, duplicate panel-time keys, and common participation patterns.

The command is also intended to replace the source-specific balanced-panel
`xtdes` facts in the [Chapter 21 data summary](../../examples/mma21p1panel/data_summary.md)
without changing that workflow's source guards or `xtsum` results.

## Bounded syntax

```text
xtdes, i(panelvar) t(timevar) [patterns(#)]
```

The panel and time keys are explicit because no persistent `xtset` declaration
is available. The `xtdescribe` spelling is an alias for `xtdes`. Both keys are
required exactly once. `patterns(#)` defaults to 9 and accepts integers from 1
through 100; this bounds structured and terminal output. The command reports
the most frequent patterns and the number omitted. Ties are ordered by the
lexicographic numeric count vector, ascending.

The panel identifier must be a nonmissing finite numeric value or a nonmissing
UTF-8 string. The time key must be a nonmissing finite numeric value. All rows
in the active dataset are considered. Unknown keys, invalid types, NULL keys,
nonfinite numeric keys, or more than 512 distinct observed time values fail the
whole command before a result is published. The time-period cap bounds the
returned time vector and pattern rows.

Repeated panel-time keys are allowed and reported. The command does not require
unique keys or a balanced panel. It does not mutate data, labels, estimates,
session context, exports, or files on success or failure.

## Structural definitions

- The period grid is the sorted set of distinct finite time values observed
  anywhere in the active dataset. No calendar spacing or `delta` is inferred.
- `observation_count` counts every input row; `panel_count` counts distinct
  identifiers; `period_count` counts distinct values in the observed period
  grid. Empty data returns zero counts, NULL time extrema and an undefined
  (`null`) balance flag.
- Per-panel `T_i` is the number of input rows for that identifier. A second
  distribution reports the number of distinct observed time values for each
  identifier. Each distribution reports min, p5, p25, p50, p75, p95 and max;
  percentiles use R type 7 linear interpolation. An empty panel set has NULL
  distribution fields.
- `duplicate_key_pairs` counts panel-time groups containing more than one row;
  `extra_pair_observations` sums each such group's excess rows (`n - 1`).
- `missing_panel_periods` sums `period_count - distinct_periods_i` over panels.
  It measures absences on the observed global period grid only; it cannot infer
  calendar periods absent from every panel.
- `balanced` is true only when every panel has exactly one observation at every
  observed global time value. An empty dataset has no balance classification.
- A participation pattern is a vector over the sorted global period grid. Each
  element is the row count for that panel-time pair, with zero for an absent
  pair. Thus repeated keys appear as counts greater than one rather than being
  hidden as simple presence. Pattern frequencies count panels, not rows.

## Result and acceptance evidence

Emit a stable `XtDesResult` containing the key names, observation/panel/period
counts, typed minimum and maximum panel and time values, duplicate and missing-period
counts, balance flag, the two panel-size distributions, the full sorted period
vector, total distinct-pattern count, requested/omitted pattern counts, and up
to the requested number of pattern rows. Each pattern row includes its count
vector, number of panels, percent and cumulative percent of panels, number of
observed periods, and row count per panel. Undefined values are JSON `null`;
counts are integers. Text output names the key variables and reports the same
counts, distributions and ordered pattern rows.

Before release, compare all returned fields on a small uneven synthetic panel
against the independent Base R control in
[`reference.R`](../../tests/fixtures/xtdes/reference.R) on the
[uneven fixture](../../tests/fixtures/xtdes/uneven.csv). Include numeric and
string panel IDs,
numeric nonconsecutive time values, an absent global-grid cell, duplicate
panel-time rows, tied pattern frequencies, truncation, an empty dataset, and
invalid/missing/nonfinite keys. Compare Chapter 21's complete checksum-pinned
5,320-row MOM source against Base R and the balanced structure printed in the
author log. Test text and JSON, both command spellings, deterministic ties,
failure-state preservation, and that existing `xtsum`, panel model and data
results remain unchanged.

The command does not include `if`/`in`, weights, a declared time increment,
format-aware date labels, persistent panel state, generalized `xtset` checks,
or inferential output. It does not claim literal Stata text-layout parity.
