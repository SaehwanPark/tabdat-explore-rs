# Chapter 23 conditional-FE Poisson firm bootstrap — WIP contract

## Exact illustration

Reproduce the author program's §23.3/Table23.1 **firm-pairs bootstrap method**
for the conditional-FE patent slope, not its historical random stream. Reuse
the pinned `patr7079.asc` and audited 1975–79 design: 346 firms/1,730 rows,
exclude 22 zero-total firms before resampling, retain 324 firms/1,620 rows.
Fit the one-slope conditional Poisson model; then B=500 bootstrap replicates,
each sampling exactly 324 retained firms **with replacement**, preserving all
five within-firm observations and counting sampled firms with multiplicities.
Refit the conditional slope for each draw; no fixed-effect interpolation or
resampling individual rows. Original author output says 324 clusters, 1,620
observations and B=500. Bootstrap assumes independent firms; it is not a
few-cluster guarantee, a model-specification repair or causal identification.

Proposed bounded composition, not a new general native bootstrap command:
SQL implements xoshiro256++ with SplitMix64 expansion, one independent
pseudorandom stream initialized from integer `10001 + replicate − 1` per
replicate, and unbiased fixed-width rejection to draw an index from the 324
firms in ascending original-id order. This explicit per-replicate stream is
**not** the existing single-stream TabDat bootstrap contract, and is not Stata
RNG parity. Reject the whole workflow if any replicate cannot generate all
indices within its bounded work or has a nonfinite, unbracketed, boundary or
numerically unidentified conditional fit; never redraw, discard or alter a
failed replicate. Report original estimate, bootstrap mean, bias, B−1 SE and
R-type7 percentile95 interval, plus sample/seed/RNG/method metadata; export
ordered per-replicate draws and fitted slopes without overwriting an existing
file. Text and CLI-v1 JSON must agree.

## Acceptance, independent evidence and exclusions

1. Before implementing, check SQL integer/bitwise arithmetic reproduces the
   external xoshiro256++ and SplitMix64 reference and unbiased rejection with
   exact Python oracle, including synthetic high-bit and forced rejection cases.
   Independently verify all 162,000 ordered original-source sampled firm IDs,
   not merely bootstrap moments. Numerical bounds and seed expansion are part of
   this workflow's reproducibility contract.
2. R independently solves each conditional fit, compares all 500 replicate
   slopes and B−1/type7 summaries and exact original sample membership, and
   validates source/design values. Illustrate the difference from the historical
   output without claiming identical Stata draws. Synthetic fixtures test
   exclusions, multiplicities, variable draw identities and root boundaries.
3. Preserve row/firm identities and ordering; test malformed/nonfinite data,
   all-zero panels, failed/unidentified bootstrap replicates, work bounds,
   no-clobber and reporting/export failures. No partial successful analysis on
   a bad replicate; source input and pre-existing files stay unchanged.
4. Update SPEC/roadmap/ledger and run focused plus affected regression checks.
   Author data are unbundled; R/Python are development-only.

Non-goals: bootstrap SE for Poisson RE/GEE/NLS, arbitrary estimators or panel
bootstrap commands, analytic cluster-robust FE covariance, histogram/figures,
BCa intervals and whole-table/chapter completion. If SQL arithmetic, execution
cost or numerical behavior cannot meet this contract, stop and reassess rather
than replacing the RNG silently or weakening validation.
