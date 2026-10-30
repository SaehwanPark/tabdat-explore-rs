# WIP: Chapter 21 Figure 21.4 first-differences source-data adaptation

## Bounded contract before implementation

Deliver the author-program Figure21.4 (§21.3 p.713) as native load/guard → raw
within-person consecutive annual differences → jointly finite pairs → person-
clustered OLS and centred-rank span0.8 LOWESS → diagnostics/CSV/SVG workflow.
The pinned `MOM.dat` SHA-256 is
`2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77`;
original source has532 people,5,320 levels and4,788 eligible 1980–88 annual
pairs. X is Δlog hourly wage, Y is Δlog annual hours. The author program plots
those differences but labels axes as levels; label changes explicitly and
record that deliberate correction, not literal axis-label parity.

Construct `LAG` on **raw** sorted levels before filtering missing outcomes,
require consecutive calendar years and both finite differences, and do not
cross people or bridge missing/gapped years. Original source is complete and
balanced, so this matches the author-program sample. Use original-key order
for LOWESS tie semantics, one OLS intercept and cluster CR1 with t(G−1); no
extra covariates, robust iterations, inference bands or causality claim.
Native ordered-row LOWESS is not R nearest-neighbour, historical Stata tie
order/storage or pixel parity. The existing23M work cap admits original work
`4788*3829=18,333,252`; no new smoother or public capacity change.

Independent development-only R keyed raw-source pairing, clustered OLS and
per-window weighted QR plus Python direct 2×2 normal equations and SVG audit:
verify all source/pair/fit rows, covariance/inference, 4,788 raw marks and
9,576 connected path vertices at fixed `1e-11+1e-10*abs(reference)` (nonzero
p relative1e-9, QR score<1e-12). Use reproducible interleaved synthetic gaps,
missing model/nonmodel values and excluded people, with exact membership and
no-bridging checks. Include text/JSON and reversed ordering, invalid/duplicate
key and name collisions, singular fit, no-clobber/source invariants, retained
OLS state and shared-wrong fit/SVG forgeries. Explicitly document ordered
nontransactional file effects and do not infer whole-Chapter21 completion.
