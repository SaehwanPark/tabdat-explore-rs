# Individual descriptive source controls (WIP)

Synthetic development-only inputs/references for the
[individual variable-wise contract](../../../examples/mma24p2poiscluster/individual_descriptives.md).
No publisher data are bundled. Base R4.6.2 independently selects finite values
per variable and computes N/mean/sample SD (N−1)/min/max. Undefined quantities
are empty CSV fields, corresponding to native null, not zero.

- `missing`: eight unsorted source keys; one wholly missing record (key6), plus
  partial missingness. PHARVIS N5 includes zero; LNHHEXP/ILLDAYS/commune N7,
  AGE/SEX/MARRIED/INSURANCE N6, ILLNESS/ACTDAYS N5, INJURY N1 and EDUC N0.
  A joint twelve-variable cohort would be empty, but descriptive data are valid.
  AGE is an uninterpreted numeric source column, not asserted years.
- `blank`: one source key17 and all twelve fields NULL. All twelve summary
  counts are0; every mean/SD/extremum is undefined. The source row is retained;
  no positive-count/rank/group-estimation constraint applies.

Input SHA256:

- missing: `c7bdb9ab0f0c5cd54e3d79f28cee539c48c5c23adf1ca2777c9fe5a27070111d`
- blank: `d346356c4b1debfd02c74a995d84210927abeadd937a023ecdd641bc3cabe16a`

Artifacts: inspectable `input.csv`, independent17-digit `source.csv`, sorted
`prepared_source.csv`, twelve-row `summary.csv` (60 fields), every available
variable/source-key pair in `members.csv` (header only for blank) and `version.txt`.
No log or level transformation is applied. Counts/source/commune keys stay
BIGINT; the other ten measurements are explicitly DOUBLE in native preparation.

Regenerate into fresh directories without changing input:

```sh
work=$(mktemp -d)
for case in missing blank; do
  Rscript examples/mma24p2poiscluster/individual_descriptives_reference.R \
    "tests/fixtures/mma24_individual_descriptives/$case/input.csv" "$work/$case" --csv
  cp "$work/$case/"*.csv "$work/$case/version.txt" \
    "tests/fixtures/mma24_individual_descriptives/$case/"
done
```

Checkpoint: actual native JSON runs and the auditor pass60 R fields plus all
source/prepared fields/keys/order/membership on both controls. Both pinned
original modes also pass on all27766 source records (including the wholly missing
27734), each with333180 available variable/key pairs; all60 precise-DTA printed
companion-log fields pass separate half-printed-digit gates. Fixed absolute1e-9
statistics tolerance; exact counts, undefined patterns and source fields.

This is source feasibility evidence, **not slice completion**: automated actual-
script/state/error/export/forgery tests, JSON/text export comparison, broader
validation and final public status reconciliation remain. Printed-book LNHHEXP
and absent MEDEXP differences remain required; no whole-table/chapter claim.
