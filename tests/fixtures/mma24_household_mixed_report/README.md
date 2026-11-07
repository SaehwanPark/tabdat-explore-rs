# Household mixed-fit comparison controls

Development-only independent R references for the WIP
[contract](../../../examples/mma24p1olscluster/household_mixed_report.md).
These establish existing-fit feasibility, **not yet installed named-report
behavior**. No publisher data are bundled.

- `balanced`:192 selected/source,32 groups, copied exactly from the joint
  equal-group Hausman balanced input. Admissible negative GEE correlation;
  Gaussian ML selects zero unit variance.
- `unbalanced`:199 selected/200 source,32 groups, singleton and missing outcome,
  copied exactly from the joint equal-group Hausman unbalanced input. Negative
  GEE correlation; ML selects zero unit variance. Do not force GEE into an RE
  positive-variance covariance or claim a missing parameter is estimated.
- `interior`:179 selected/180 source,16 groups, positive GEE correlation and ML
  unit variance. Copy existing household RE interior input, rounding every
  nonmissing non-key cell to the nearest multiple of1/64, ties to even. Keys,
  missingness and row order are unchanged. Short exact dyadics avoid the known
  R17-digit decimal-token parsing ambiguity without widening source tolerances.

Input SHA256:

```text
balanced   8a9bc1e55a3ddf24af2f48756602b635897b8f9dc8206352dc116fa4b749ed09
unbalanced 1ea87219c60e10800a06ac314018b6fb5a6e87c43e5973f93da09096c312a8ef
interior   c2a785816b0beeeeecce44874c7e0d31dd3f2b6693686cb936df30d236384130
```

Recreate the interior input from the repository root (write a fresh destination):

```python
import csv
from decimal import Decimal, ROUND_HALF_EVEN
with open('tests/fixtures/mma24_household_re/interior_input.csv') as source:
  rows = csv.DictReader(source)
  with open('/tmp/mixed-interior-input.csv', 'x') as target:
    writer = csv.DictWriter(target, fieldnames=rows.fieldnames, lineterminator='\n')
    writer.writeheader()
    for row in rows:
      for key, value in row.items():
        if key not in ('source_row', 'commune') and value:
          row[key] = format((Decimal(value)*64).to_integral_value(rounding=ROUND_HALF_EVEN)/64, 'f')
      writer.writerow(row)
```

R4.6.2, geeM0.10.1, Matrix1.7.6, foreign0.8.92. Set `R_LIBS` to a dedicated
validation library if optional packages are not installed globally. For each
case, use a fresh `$out` parent and existing independent generators:

```sh
out=$(mktemp -d)
case=balanced
input="$PWD/tests/fixtures/mma24_household_mixed_report/$case/input.csv"
Rscript examples/mma24p1olscluster/household_gee_reference.R "$input" "$out/gee" --csv
Rscript examples/mma24p1olscluster/household_fe_reference.R "$input" "$out/fe" --csv
Rscript examples/mma24p1olscluster/household_re_gls_reference.R "$input" "$out/re_gls" --csv --equal-group-between
Rscript examples/mma24p1olscluster/household_re_ml_reference.R "$input" "$out/re_ml" --csv
```

Freeze each `reference.csv` as `<fit>_reference.csv` and ML's `predictions.csv`
as `ml_predictions.csv`. R CSV formatting is preserved. GEE references contain
both working and uncorrected unit-sandwich inference. FE includes the normalized
common-intercept/full covariance and separate restriction rows; this comparison
needs the fit/common rows, not those additional tests. Raw-source reference
counts are not interchangeable with native candidate counts after explicit
complete-case preparation. Preserve sample semantics when consuming them.

Existing native scripts versus R passed496 numerical gates per control:
GEE198, FE fit/common103, equal-group GLS97, ML98 including marginal likelihood.
Absolute1e-9 and nonzero-p relative1e-8; exact-zero underflow. All192/199/179 ML
prediction rows also match R, with exact selected source fields/keys/order and
absolute1e-9 prediction gates. These are ad hoc feasibility checks; permanent
new-command/wire/state and actual unified-script checks remain to implement.
