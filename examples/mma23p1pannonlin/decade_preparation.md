# Patents full-decade preparation — WIP

Bounded outcome: convert all25 raw fields into3460 keyed1970–79 records,
derive LPAT/DPAT/RANDD, five LOGR lags and four year indicators before selecting
the1730-row1975–79 analysis cohort, then report variable-wise summaries and
export both prepared cohorts. Source-record ordinal ID is not CUSIP. Missing
ARDSSIC never drops a row. Default five-year conversion remains byte-compatible.

Acceptance cases:
1. Publisher source (SHA25687525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8):
 all3460 full and1730 selected records/derived values and per-variable N/mean/
 N-1 SD/min/max agree with independent raw-token R calculations; no source data
 committed. Labels are explicit native metadata, not DTA byte parity.
2. Artificial40-firm source:400 full/200 selected records; lag j has40*j missing
 full-decade values and NO selected lag missingness. In particular1975 lag5 uses
 1970, not a lag computed after restriction. Shuffled long CSV gives identical
 ordered exports. Raw firm order still defines ordinal ID.
3. Malformed/incomplete raw conversion and invalid/duplicate/gapped long keys
 fail without modifying input files or publishing invalid exports. Existing
 converter defaults and existing five-year workflows remain unchanged.
4. Both exports are no-clobber. The script is ordered, not a multi-file
 transaction: a later failure may leave an earlier successful export. Conversion
 stages beside its output and publishes only after complete validation.

No estimator/refit or production R/Python dependency, general panel declaration,
`xtsum`, wide-schema summaries, historical float32/DTA/label restoration, named
comparison or chapter completion is claimed. Those inventory portions remain
required. Full years1970..1979 deliberately replace source two-digit70..79;
LPAT uses LN(0.5) only at zero PAT; RANDD uses DOUBLE EXP(LOGR); lagging requires
complete unique ten-year keys so calendar and row offsets coincide.

Independent reference: base R raw25-token records, direct indexing of source
history (not SQL windows), independent log/exp/indicator transforms and variable-
wise finite summaries. Source fields/keys/missingness compare exactly as parsed
binary64; derived values and summaries use fixed1e-10+1e-12*abs(reference), not
historical rounded-output parity. Constant/all-missing SD semantics are explicit.

## Implementation checkpoint (draft PR443; not merge-ready)

`prepare.sh RAW OUT [--full-decade]` now selects explicit ten-year conversion;
two-argument/default AWK remains1975–79. Conversion still stages beside the
output and publishes by no-clobber hard link only after complete validation.
`decade_preparation.td` guards complete unique ten-year keys, derives all21
fields in key order, labels native columns, summarizes/exports full history,
then uses `keep if year>=1975` before selected summaries/export. ARDSSIC is
explicitly cast to DOUBLE so wholly missing nonmodel metadata remains numeric
for N0 summaries; it is never a row-selection condition. CSV does not preserve
native label metadata. No NEWPAT temporary column is exported.

Reproduce in a fresh directory from the repository root:

```sh
mkdir /tmp/my-patents-decade
sh examples/mma23p1pannonlin/prepare.sh /path/patr7079.asc \
  /tmp/my-patents-decade/patents_decade.csv --full-decade
(cd /tmp/my-patents-decade && /absolute/repo/target/debug/tabdat-explore-rs \
  --json -f /absolute/repo/examples/mma23p1pannonlin/decade_preparation.td > result.jsonl)
Rscript examples/mma23p1pannonlin/decade_preparation_reference.R \
  /path/patr7079.asc /tmp/my-patents-decade-reference
python3 examples/mma23p1pannonlin/check_decade_preparation.py \
  /tmp/my-patents-decade /path/patr7079.asc /tmp/my-patents-decade-reference
```

Use `--text` in a separate prepared directory. Artificial source uses the
existing `tests/fixtures/mma23_gamma_re_bootstrap/source.asc` and checker
`--control`. Independent R direct-history indexing/log/exp/summaries plus Python
raw-source/derived-value/empirical-summary guards validate all3460/1730 original
and400/200 control rows,21 fields and42 variable-wise summary rows each.
Original source hash is pinned in the checker. The first original CLI attempt
had a duplicated binary-path typo; corrected invocation passes, not a code fix.
The final guarded/typed script was separately replayed on both sources.

New focused nextest `ec67342f-56a5-4548-99cd-42e9f62cdfea` passes3 tests:
- frozen R full/selected records and all42 summaries, text/JSON equal exports,
  no-clobber and default five-year projection equivalence;
- shuffled source and wholly missing ARDSSIC, including numeric N0 summaries;
- malformed converter input, owned output, missing/duplicate/out-of-range years,
  unchanged input files and no invalid exports.

Evidence is under `/tmp/tabdat-ch23-decade/`: `control-reference/`,
`original-reference/`, final `control-v3/` and `original-v3/` result/export files,
and `focused-nextest.log`. Frozen artificial R files are committed under
`tests/fixtures/mma23_decade_preparation/` and do not require R in Cargo tests.

Still required before merge: independent-checker hostile-artifact tests,
representative existing five-year reverse-consumer regressions, scoped lint/
final author review, specification/roadmap/coverage/inventory reconciliation and
GitHub verification. No affected-crate/workspace/Clippy/CI/platform/release or
whole-inventory completion is claimed at this checkpoint. The source contract
at the top preceded implementation; no tolerance was widened.

Weekly quota reached97% used/3% remaining on2026-11-08. Work is safely saved as
WIP rather than merged prematurely; resumption requires the operator's
`/goal-resume`. No unattended weekly restart is promised.
