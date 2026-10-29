# WIP: Chapter 23 Figure 23.1 pooled log-patents/R&D source-data adaptation

## Contract before implementation

Deliver a reproducible load/prepare → specify → fit → diagnose → report/export
workflow for Figure 23.1 of Cameron–Trivedi (2005) §23.3: all 346 original
source-order firms and 1,730 1975–79 rows, LOGR on the horizontal axis and
LPAT on the vertical, with the author's zero-count substitution ln(0.5), raw
observations, the nonparametric local fit, and the pooled linear fit. The
March 2005 author program labels this graph "Figure 21.1" in a comment but
identifies it as Chapter 23 Figure 23.1 and says its axes correct the book.
Follow the executed program's axes. Use existing native `lowess` with explicitly
specified span 0.8, native `regress`/`predict`, and `scatter ... lines(...)`
SVG, not a new general graphics or estimation API. Use source `(id,year)`
order before smoothing, and publish one source/fit CSV and one SVG in fresh
paths. Report native clustered pooled slope and no smoothing uncertainty.

Native centred-rank nonrobust LOWESS is an ordered-row smoother. Exact equal X
can have different fitted values depending on input order. R `stats::lowess`
is **not** the reference for its rank-window rule. The default Stata span is
0.8 but historical Stata tie ordering and generated-float precision are not
established; do not claim literal original graph/curve/pixel reproduction or
use a wider tolerance to obscure differences. This is a validated source-data
adaptation under the native method, not complete Table 23.1 or chapter closure.

Independent development-only R QR solves the declared local weighted design,
with a Python direct normal-equation audit and independent pooled OLS check.
Original-source evidence must check exact source/design membership, all 1,730
fitted rows, all raw SVG markers and both connected fitted paths/geometry;
small inspectable fixture covers ties and deterministic ordering. Numerical
thresholds follow the existing independently validated LOWESS workflow and
must be fixed before validation.

Acceptance and failure cases: (1) pinned original input and a balanced fixture
produce text/JSON-equivalent CSV/SVG and all numerical fits agree; (2) input
row reversal gives the same source-key ordering and fits; (3) actual SVG paths
and marker coordinates match the exported data; (4) malformed/duplicate keys,
missing/nonfinite/unsupported model values and existing derived-column names
fail before fitting; (5) preexisting destination is not overwritten and source
is unchanged; document nontransactional earlier effects and retained native
OLS state. No historical graph parity, robust smoothing, new-X prediction,
uncertainty bands, causal conclusions, arbitrary data dimensions, or general
new graphical commands are included.
