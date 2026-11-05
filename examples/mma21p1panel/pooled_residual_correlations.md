# Table 21.3 pooled residual correlations — WIP contract

**Inventory entry / bounded outcome:** Cameron–Trivedi (2005) §21.3.4 p714,
Table21.3 pooled-OLS portion, author March2005
[`mma21p3panresiduals.do`](https://cameron.econ.ucdavis.edu/mmabook/mma21p3panresiduals.do)
lines75–108 and [output](https://cameron.econ.ucdavis.edu/mmabook/mma21p3panresiduals.txt)
lines321–343. Publisher source SHA256 `6c4ccfa560eb8f818412e87895e4c205f51c1e00865346f3b0c532902b7a43e8`,
output `925aff11176cb78bbacf71abc69c2bef4876bdcc0e5544c001c2df4f5f8f56b5`.
MOM.dat eight fields, original532 people x10 years1979–1988. Fit intercept+lnwg
pooled OLS on the same finite sample, predict source-row residuals, correlate
cross-person residuals for each ordered year pair (100 cells); report/export
ordered source/prediction rows and year-pair matrix, with sample counts. This is
descriptive correlation, not a standard-error correction or formal test.

**Bounded input:** 3..532 complete ten-year people, exact integer unique bounded
source_row/id/year; finite lnhr/lnwg within [-20,20], OLS rank and strictly positive
cross-person residual variation in every year. No missing/unbalanced sample,
weights, extra regressors or imputation; checks precede output. Source order is
preserved for prediction export, matrix ordered year_a/year_b. Failure never
overwrites an existing export or source; the two-export script is not a whole-
session filesystem transaction.

**Acceptance before implementation (high risk: data/statistical semantics):**
1. Independent base R original raw-source QR and matrix `cor` checks fitted OLS
   coefficients/full covariance and all5,320 raw/predicted rows and all100 ordered
   correlations/counts with abs1e-9, exact source keys/raw values. Separately compare
   all45 author-log off-diagonal 4-decimal numbers, investigate differences (no
   widened numerical tolerances); output precision is not historical storage parity.
2. Small interleaved deterministic full-year control validates all100 cells, exact
   membership/order, fit, both text/JSON and post-correlation retained-model inference.
3. Duplicate/fractional keys, missing/incomplete years, rank failure, constant-year
   residual variance and existing-export collisions fail without corrupting source or
   existing targets. Explicitly test partial-output behavior if second export collides.
4. Runnable reproduction with R reference only; update ledger/SPEC/roadmap and a
   source inventory distinguishing this completed sub-block from remaining levels,
   FE/RE, FD, individual correlations and full program/book coverage.

Use existing SQL/OLS/predict/export only; no production API/dependency or published
data redistribution. Start focused tests, run root package suite, review final diff
and CI before merge. If source behavior or output conflicts with the plan, report
rather than silently substitute. Report files, commands, discrepancies and limitations.
