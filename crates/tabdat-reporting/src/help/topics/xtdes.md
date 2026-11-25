# xtdes

`xtdes, i(panelvar) t(timevar) [patterns(#)]`

The `xtdescribe` spelling is an alias. Both panel and numeric time keys must be
explicit because no panel declaration is retained. The command is read-only,
uses all active rows, permits repeated panel-time keys, and reports those
duplicates. It does not infer a calendar step or declared delta.

- `xtdes, i(firm) t(year)` reports the default nine most common participation
  patterns.
- `xtdescribe, i(person) t(period) patterns(20)` reports up to twenty patterns.

Patterns are row counts over the sorted distinct time values observed anywhere
in the data; zero means a panel has no row at that time. The time grid is capped
at 512 distinct values, and `patterns(#)` accepts 1 through 100. Structured
output uses `XtDesResult`. See the [full contract](../../../../../docs/commands/xtdes.md).
