# WIP: year-pair correlations of panel log hours and log wages

## Bounded contract before implementation

Reproduce the Chapter21 author residual program lines105–106 (two descriptive
10x10 matrices of `lnhr` and `lnwg`), using the published MOM.dat panel rather
than treating pooled/FE/RE residual matrices as the observed-level matrices.
Input and finite/key/balanced-sample constraints should match the validated
[pooled residual workflow](pooled_residual_correlations.md):3..532 people, all
10 years1979–1988, one valid unique `(id,year)`, no missing/nonfinite model
values, bounded exact integer keys/source row; original N5320, G532. Pearson
correlations are calculated across the same complete people for each pair of
years, separately for each variable. Report person count, ordered year pair and
correlation; retain/export all source fields/order, both matrices and scope note.
No fit, causal claim, formal serial-correlation test or covariance adjustment is
implied. This is a descriptive subworkflow, not full chapter/book coverage.

Acceptance: (1) original all5,320 source rows and200 matrix/table cells against
independent R `cor`; (2) all90 off-diagonals compared with author output at
printed four decimals, separate from fixed absolute1e-9 R comparison; (3)
interleaved dyadic control with both nonconstant variables and text/JSON export
agreement; (4) common people counts/order, diagonal/symmetry and explicit
interpretation. Failure: duplicate/incomplete/missing/nonfinite/out-of-range
keys/values and an exactly constant year for either variable fail before any
export; first/second export collisions preserve existing files and document any
partial export. Independently verify original raw source hash and author-log hash;
no publisher data or R/Python runtime dependency is added to production.

References: author [source inventory](residual_inventory.md), printed log lines
278–319; independent R `cor` on the exact validated panel. SQL guards are expected
to surface generic `sql failed`, and two exports are not promised transactional.
Do not count either as fixed or claim FD/individual matrices or chapter completion.
