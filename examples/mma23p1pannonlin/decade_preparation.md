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

Implementation and verification are pending. This contract precedes the code.
