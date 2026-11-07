# Household descriptive reference controls

Small synthetic development-only controls for the
[household variable-wise workflow](../../../examples/mma24p1olscluster/household_descriptives.md).
No publisher data are bundled. Reference: base R4.6.2, `mean`, `sd` (N−1),
`min`/`max` on independently selected finite values. Undefined quantities are
empty CSV fields, corresponding to native JSON null, never zero.

- `missing`: eight deliberately unsorted source rows, three communes; outcome
  N4 (including zero and a negative log outcome), age/size/log expenditure N7,
  sex N5, FARM N1 and EDUC N0. Every variable has an explicit independent
  membership list. Source keys are not a time series. This demonstrates why
  neither source N8 nor a joint model cohort is every variable's sample.
- `singleton`: one raw source row, missing outcome, all other samples N1.
  All sample SDs are undefined. Describing data does not require the eight-
  commune/slope-rank bounds of the separate panel estimation workflows.

Input SHA256:

- missing: `d506b8c91a0f6ac3ddd5372cbcf0eb93bf0799d1884644f128cc89f59316d479`
- singleton: `7a437b7d0e34cdaa0d397ae1e506ed2af1ba0af2eb5204109545388e5a5b4bf2`

Artifacts per case:

- `input.csv`: native input, with its unsorted original row order.
- `source.csv`: independent R source decoding,17-digit binary-value roundtrip.
- `prepared_source.csv`: sorted source with explicitly DOUBLE HHEXP=exp(LNHHEXP).
- `summary.csv`: nine ordered rows/count/mean/sample-SD/min/max (45 fields).
- `members.csv`: every variable/source-key pair, ordered by variable then key.
- `legacy_float_summary.csv`: **separate diagnostic**, casting generated HHEXP
  to binary32 as the original default Stata `gen` would; never silently substituted
  for the native double oracle.
- `version.txt`: generating R version.

Regenerate into fresh directories, retaining the inspectable input:

```sh
repo="$PWD"
work=$(mktemp -d)
for case in missing singleton; do
  Rscript examples/mma24p1olscluster/household_descriptives_reference.R \
    "tests/fixtures/mma24_household_descriptives/$case/input.csv" "$work/$case" --csv
  cp "$work/$case/"*.csv "$work/$case/version.txt" \
    "tests/fixtures/mma24_household_descriptives/$case/"
done
```

Fixed absolute1e-9 statistics tolerance; exact counts, undefined patterns and
source fields/keys/membership. Source-row order normalization is intentional;
raw files and prior export owners are preserved. The actual-script integration
checks JSON/text byte-identical exports, typed summaries, final source ownership,
read-only summaries with labels/retained OLS,16 invalid inputs and both export
collision boundaries. Python audit regressions run both controls and a permutation
and reject20 forgeries, including mutually changed native/R means and levels.

Original ASCII/precise DTA are separately checked on all5999 source/prepared rows,
45 R fields and45 printed DTA author-log facts. These controls do not establish
literal Table24.3 book parity: the author notes differences and FARM substitutes
for the printed URBAN variable, which is not present in the supplied source.
