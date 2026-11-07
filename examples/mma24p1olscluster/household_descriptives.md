# Household variable-wise source summaries

**Implemented and independently validated bounded native workflow.** H-DESC,
companion program lines69–72, Table24.3 household portion p.850. This validates
the companion summary, not literal printed-book parity. It is independent of
the unresolved historical ML SE investigation; no estimator changes.

## Outcome and conventions

The [script](household_descriptives.td) provides native load/audit → prepare →
summarize/interpret → export/reproduce for the nine variables in source order:
`lnexp12m age sex hhsize farm educ hhexp lnhhexp commune`.

Use the pinned household README's ASCII or precise numeric-code DTA input,
`vietnam.csv`, with its original nine source columns. Original5999 source rows;
outcome has5006 observations, other source variables have5999. **Do not select
the joint estimation cohort before describing the data.** Each variable has its
own finite available-row sample. Zeros remain; sample SD uses N−1 and is undefined
below2 observations. All-missing variables report N0/NA, not empty-data failure.
There is no model, sampling weight, survey/causal inference or confidence interval.
Commune means/SDs describe arbitrary numeric identifiers, not geographic location
or cluster-size distributions. EDUC is a category code, not years of schooling;
FARM is not a measurement of the book's URBAN variable.

Preparation explicitly normalizes the seven measurement columns to DOUBLE,
retains positive BIGINT source/commune keys (<=2^53−1 for exact independent R
identity checks), and orders rows by unique source-row key. Nullable measurements
are allowed, including entirely missing CSV columns; nonnumeric text/nonfinite
nonmissing measurements fail. Raw input must contain1..40000 rows. Generate
`hhexp=exp(lnhhexp)` as **DOUBLE**, not a purported original level expenditure
column. A nonmissing generated level must remain positive/finite; exponential
underflow/overflow fails. This deliberately uses native double precision instead
of the source's default Stata float storage for `gen HHEXP`; independent R uses
the same declared double convention. No input file is overwritten.

Use the existing native ordered `summarize` result and a nine-row CSV report
with `variable,count,mean,std_dev,minimum,maximum`. Also export the complete
prepared source, including every generated HHEXP value. Reactivate that source
at the end so subsequent modeling can explicitly choose its own cohort. The
whole script is ordered, not transactional; an existing export owner is never
overwritten and earlier successful effects/files are not rolled back.

## Verified behavior

1. Both original modes: validate all5999 source/prepared rows, exact fields/keys/
   order/membership, and45 independent R count/mean/sample-SD/min/max quantities.
   Keep author-log rounding gates separate; audit all45 printed facts on precise
   DTA without widening tolerances or asserting literal printed-book parity.
2. Small independently frozen controls: variable-wise missingness (different
   membership/counts), retained zeros, all-missing and singleton-variable SD/NA;
   a one-row source must work without unnecessary panel-model sample bounds.
3. Actual script JSON/text: typed ordered summaries, both exports byte-identical,
   final prepared-source ownership/order, source bytes unchanged. Inspect-read-only
   summary behavior separately from the script's deliberate load/preparation.
4. Permuted raw rows still yield source-key-ordered prepared records and the same
   summaries. Forge source/generated values, report counts/statistics/order and
   cohort membership; independent audit must reject targeted corruptions.
5. Empty/duplicate/wrong-type/nonpositive/inexact keys, nonnumeric/nonfinite
   measurements, failed EXP and export collisions fail without overwriting source
   or pre-existing output owners. Describe partial effects honestly.

Independent development-only base-R reference; no production dependency or
new framework/API. Fixed absolute1e-9 statistics tolerance, exact counts/source
fields and separate half-printed-digit author gates. Native finite-arithmetic
aggregate overflow may still fail rather than claim general numerical stability.

## Scope / stop conditions

The author explicitly notes differences from printed Table24.3 and substitutes
FARM for URBAN. This slice validates the **companion household summary workflow**,
not missing URBAN or literal full-table/chapter/book coverage. Keep those source/
book distinctions, individual summaries and all other inventory gaps required.
If an author gate differs under declared precision, investigate and document it;
do not replace values or widen tolerance to manufacture agreement. Reassess
scope if an unavailable essential source or new public contract is required.

## Run and independently reproduce

Graduate-level users can run the native workflow without R/Python after the
[household README](README.md)'s pinned source preparation. Build with pinned
Rust1.99.1 (`export PATH="$HOME/.cargo/bin:$PATH"`):

```sh
cargo build --locked --bin tabdat-explore-rs
repo="$PWD"
work=/path/to/fresh-working-directory  # Already contains prepared vietnam.csv.
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma24p1olscluster/household_descriptives.td" > result.jsonl)
```

Use `--text` for readable ordered summaries and run in a separate fresh directory
to compare exports. Existing owners cause an error; choose a fresh directory,
not an overwrite flag. Source reload/preparation deliberately changes the active
data and clears any prior fit under existing load semantics. The summary itself
is read-only, including with labels/retained OLS; this is not model restoration.

For independent precise DTA validation (optional development R/Python):

```sh
ref=$(mktemp -d)
Rscript examples/mma24p1olscluster/household_descriptives_reference.R /path/to/vietnam_ex1.dta "$ref/dta" --dta
python3 examples/mma24p1olscluster/check_household_descriptives.py "$work" "$ref/dta" /path/to/vietnam_ex1.dta --dta /path/to/mma24p1olscluster.txt
```

For ASCII, omit `--dta` from R, use `--ascii` in Python and omit the author log;
use separately prepared ASCII native input, not DTA numbers relabelled ASCII.
DTA uses `foreign::read.dta(...,convert.factors=FALSE)`. Reference artifacts use17
significant digits, base-R mean/sample-SD/extrema and independently enumerated
per-variable source keys. A separate binary32-generation diagnostic never replaces
the declared native DOUBLE oracle.

Both original modes pass45 R fields and all5999 source/prepared row/membership
checks; precise DTA also passes all45 printed companion-log rounding facts.
[Two frozen synthetic controls](../../tests/fixtures/mma24_household_descriptives/README.md)
cover N0/N1/variable-wise samples and source permutations. Actual-script tests
pass5/5, including JSON/text byte-identical exports, final source ownership,
read-only labels/OLS,16 invalid inputs and both export failures. The optional
`python3 examples/mma24p1olscluster/test_check_household_descriptives.py` passes
both controls/permutation and rejects20 forgeries (including shared native/R
wrong means or generated levels). Shared BP/mixed-auditor regressions passed.
Broader root-regression evidence and the corrected prior named-report schema
golden are recorded in [PR#432](https://github.com/SaehwanPark/tabdat-explore-rs/pull/432);
no Rust documentation examples or statistical kernel changed. Whole-book/full-survey/platform/release claims are not implied.
