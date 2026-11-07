# Individual variable-wise source summaries

**Implemented and independently validated bounded native workflow.** I-DESC,
companion program lines71–74, Table24.3 individual portion p.850. This validates
the companion source summary, not literal printed-book parity or survey inference.
No statistical kernel, model-fitting or runtime API change.

## Source, sample and interpretation

The [script](individual_descriptives.td) provides load/audit → typed preparation →
per-variable summary/interpretation → report/export/reproduce for source order:
`pharvis lnhhexp age sex married educ illness injury illdays actdays insurance commune`.

Use the [individual README](README.md)'s pinned numeric-code CSV
`vietnam_individual.csv`. Original27766 records include source key27734 with
**all twelve fields NULL**. Keep that row: per-variable N27765 is not source
N27766. Partially missing synthetic rows are also retained with independently
available samples, not a joint/listwise filter. Zeros remain; sample SD uses N−1,
N0/N1 SD is undefined. No fit, weight, survey/causal inference, mean uncertainty
or imputation is supplied. All-missing variables have N0 and null statistics,
not zeros or a failed positive-count/rank/group-estimation condition.

The author renames `lnhhinc` to LNHHEXP: preserve the numeric column without
asserting income versus expenditure or generating an undocumented level measure.
AGE values resemble logged ages; do not transform or interpret as years. EDUC/
ILLNESS and other codes are not factor-expanded; ILLNESS is not assumed binary
(its original values range0..9). Commune-code moments are not location or
cluster-size statistics. These are unweighted available-observation summaries,
not national population estimates.

## Preparation and outputs

Raw input1..40000 rows. Unique positive BIGINT source keys <=2^53−1 are required.
PHARVIS is nullable nonnegative BIGINT <=2^53−1; commune is nullable positive
BIGINT <=2^53−1. All-NULL CSV columns are allowed despite import type inference.
The remaining ten measurements may be NULL or must cast to finite DOUBLE.
Prepare explicit BIGINT counts/keys and DOUBLE measurements, sorting by source
key while preserving every record. Malformed/nonfinite data fail rather than
silently changing memberships. Native finite-arithmetic aggregate overflow may
still fail; no universal numerical-stability claim.

Native `SummarizeResult` exposes twelve ordered rows with
`variable,count,mean,std_dev,minimum,maximum`. Undefined values are JSON null.
Extrema retain their native BIGINT/DOUBLE scalar types. Text uses the ordinary
readable summary renderer; this is not exact Stata formatting or a configurable
publication layout.

- `individual_descriptive_source.csv`: every prepared record, including wholly
  or partly missing records, with exactly the original thirteen column names.
- `individual_descriptives.csv`: twelve ordered report rows and the same six
  fields as above. UNION report extrema are DOUBLE; integer code/count values
  stay exact within declared bounds, with nulls empty in CSV.

Finally `use individual_descriptive_source` reactivates the full prepared source
for later explicit modeling. The whole script is **ordered, not transactional**:
load/preparation changes current data and clears a prior fit under existing load
semantics. Ordinary summaries are read-only, including with labels/a cached
Poisson fit, and do not substitute its selected cohort. Existing file owners are
never overwritten; if the second export fails, the first successful file remains.
Use a fresh directory/session, not an overwrite flag, to rerun.

## Run and independently reproduce

Graduate-level users can run native CSV analysis without R/Python. Build using
pinned Rust1.99.1 (`export PATH="$HOME/.cargo/bin:$PATH"`):

```sh
cargo build --locked --bin tabdat-explore-rs
repo="$PWD"
work=/path/to/fresh-working-directory  # Contains prepared vietnam_individual.csv.
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma24p2poiscluster/individual_descriptives.td" > result.jsonl)
```

Use `--text` in a separate fresh directory for readable output. Both modes
export byte-identical source/report files on the original modes and controls.
For optional development-only independent R/Python audit of precise DTA:

```sh
ref=$(mktemp -d)
Rscript examples/mma24p2poiscluster/individual_descriptives_reference.R /path/to/vietnam_ex2.dta "$ref/dta" --dta
python3 examples/mma24p2poiscluster/check_individual_descriptives.py "$work" "$ref/dta" /path/to/vietnam_ex2.dta --dta /path/to/mma24p2poiscluster.txt
```

Prepare native precise DTA input using the README's existing
`dta_reference.R ... --precise`; do not relabel rounded ASCII as precise DTA.
For ASCII, use the README's standard-library `prepare.py`, omit `--dta` from R,
use `--ascii` in Python and omit the author log. DTA decoding uses
`foreign::read.dta(...,convert.factors=FALSE)`; reference source artifacts use17
significant digits. Base R4.6.2 independently computes each available-variable
sample/mean/sample-SD/extremum without reading native output.

## Evidence and limits

Both pinned original modes pass **60 independent R fields**, all27766
source/prepared fields/keys/order and333180 available variable/key pairs.
Record27734 remains wholly missing. Precise DTA additionally passes **60/60
printed companion-log facts**, using separate half-printed-digit gates.
Statistics absolute1e-9; exact counts/undefined patterns/source fields; no
widened tolerances or manufactured agreement.

[Two frozen synthetic controls](../../tests/fixtures/mma24_individual_descriptives/README.md)
include unsorted keys, partial/whole missingness, retained zero, N0/N1 variables,
and a one-row entirely missing record. Actual-script tests pass5/5: full typed
reports, all prepared fields/order, JSON/text byte-identical exports, permutation,
source reactivation, labels/cached-Poisson read-only state and differing sample
counts,21 invalid inputs, and both export failure boundaries. Invalid-source
checks require failure **before summaries/exports**, so an existing output cannot
mask a failed validation guard.

`python3 examples/mma24p2poiscluster/test_check_individual_descriptives.py` passes
both real-binary controls/permutation and rejects20 targeted forgeries, including
shared native/R wrong means, dropped missing records, false counts/membership,
undefined statistics changed to zero and corrupt metadata. Scoped Clippy/fmt/
diff checks pass. The PR records impact-scoped use of prior unchanged root-base
verification rather than repeating passing baselines; no workspace/doctest,
release or platform claim follows.

The author acknowledges a printed-book LNHHEXP difference and absent MEDEXP.
Those reconciliations remain required, not counted complete. I-DETAIL/I-FREQ/
I-HIST, named comparisons, historical ML SE differences and all other unresolved
inventory entries remain required. This is not whole-table/chapter/book coverage.
