# join

Terminal success now shows `Joined: N rows, K columns` using the owned post-join
metadata. Inspect actual values with `head`/`describe`; no key, direction, matched-row,
cardinality or affected-row counters are inferred or rescanned.

How to invoke:
`join table_name on keyvars [, how=inner|left suffix(_right)]`

What it does:
Enrich the active dataset using an existing session-local named table with the installed inner/left join forms.

What problem it answers:
How do I attach lookup fields before a supported summary/model/persistence workflow?

Examples:
- `sql select id,region from active order by id into lookup`
- `use primary.csv`
- `join lookup on id`
- `join lookup on firm_id year, how=left suffix(_lookup)`

Notes:
- Inner is the default and omits unmatched active rows. Left keeps one row with NULL right-side fields for each unmatched active row. One/multiple shared keys and backtick-quoted column names are supported; referenced keys must exist on both sides.
- Key equality uses native SQL semantics: NULL keys do not match each other, including partially NULL composite keys. Native type coercion applies; no lossless wide cross-type equality or pandas missing-key behavior is implied.
- Duplicate matches multiply rows. Output groups follow prior active-row order; within each active row, matches retain stored right-side order. Sort/query lookup relations deliberately before joining when that order matters. Empty and same-table joins are supported.
- Left columns remain first. Right key columns are omitted; right nonkeys append in right schema order. Existing name collisions use the requested/default suffix and incremental unique names where needed. Final native names/types are reported; no additional cross-case naming guarantee is installed.
- CLI-v1 JSON returns `JoinResult` with owned `data.dataset` (source, total rows, ordered schema), not key/matched/unmatched/affected-row diagnostics. Dataset label/panel fields remain null placeholders, not snapshots.
- Existing left variable labels, label definitions and attachments for surviving columns remain. Right labels are not propagated. The original cached fit/sample is retained, not refit or rewritten for duplicated rows; explicitly fit for enriched-data estimation. Prediction uses current rows. An active named relation is updated under existing synchronization rules.
- Unknown tables/keys and tested native conversion/name failures preserve prior relation/session/named state and stop later files. Completed effects/files are not rolled back on later output failure (MIG-0008). No m:1/1:1 cardinality validation, outer/right join, generalized merge diagnostics, by/lazy/remote execution or whole-command parity is implied.
