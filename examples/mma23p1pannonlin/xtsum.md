# Full-decade patents panel summaries

## Bounded objective

Reproduce the March 2005 author program's `xtsum, i(id)` block (program line
152) on the full 1970–1979 transformed long data, before selecting 1975–1979
for the fitted models. Report overall, between and within summaries for all 21
numeric fields retained by the existing [full-decade preparation](decade_preparation.md).
This closes P-XTSUM only; it does not add a general `xtsum` command, persistent
panel declaration, wide-data summaries or Stata DTA restoration.

## Input and missingness contract

Use the checksum-pinned `patr7079.asc` and the existing strict full-decade
converter/preparation. Retain 3,460 ordered firm-years, firms 1–346 and years
1970–1979. `id` is the raw source-record ordinal; CUSIP is not the panel key.
Do not apply the 1975 model restriction before summarizing.

Match the source's variable-specific nonmissing patterns:

- `ARDSSIC` is missing for all ten years of ten firms, giving 3,360 observations
  across 336 contributing firms.
- `LOGRL1`–`LOGRL5` are missing for the first one through five years of every
  firm, giving `346 * (10 - j)` observations for lag `j` and all 346 firms.
- The remaining variables are finite for all 3,460 rows.

Each variable's observed panel records must have the same count across all
contributing firms. Reject partial within-firm missingness or any changed
cohort/pattern: on this bounded source, it would change the weighting and
centering conventions and needs a separate contract.

## Statistical definitions

For each variable, calculate overall mean, sample SD, minimum and maximum from
its finite rows. Between statistics are the sample SD, minimum and maximum of
the unweighted means for firms with at least one observed value. Within values
are `x_it - xbar_i + xbar_panel`, where `xbar_panel` is the unweighted mean of
the contributing firms' observed-value means; report their sample SD, minimum
and maximum. The mean of panel means equals the pooled mean here because each
variable is balanced across its contributing firms. Preserve separate overall
N, contributing-panel count `n`, and mean observed periods `T` for every
variable. Do not substitute regression samples or variance components.

## Acceptance evidence

- Compare all 21 rows, including variable-specific N/n/T, all reported moments
  and extrema, against Base R calculations made directly from the raw 25-token
  source rather than prepared native output.
- Check rounded author-log values with the Stata float32 storage path and the
  exact printed-decimal half-unit tolerance; native raw-double values remain
  separately visible.
- Exercise the full source and synthetic controls for a valid balanced
  variable-specific pattern. Reject changed row/key cohorts, finite-forged
  `ARDSSIC`, and partial within-firm lag missingness without changing the
  prepared input.
- Preserve existing five-year conversion, preparation exports and downstream
  panel workflows.

Source hashes, reproduction commands and unresolved DTA/label boundaries remain
in the Chapter 23 inventory and evidence notes.
