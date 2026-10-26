# Independent centred-rank LOWESS controls

Synthetic data only; no author observations redistributed. `input.csv` has31
unordered, nonlinear/noisy synthetic rows including15 tied X3 rows; Y=.2X²−.8X+2+
((11*zero_based_input_row mod17)−8)/10. `all_tied.csv` has7 signed-zero X rows
with Y[−2,0,6,4,12,8,14]. These exercise asymmetric boundary shrinkage, stable
within-tie input order, exact coincident-window local means and k0 identity.

`expected.csv`/`all_tied_expected.csv` freeze every per-row fit at.8/.4/.1 from
base R weighted Householder QR (`lm.wfit`), not the native weighted moments.
Corresponding metadata records N/k/coincident-window counts and normalized
weighted normal-equation certificates<1e-12. `nearest_window_difference.csv`
shows R stats::lowess(f=span,iter=0,delta=0) is not this centred-window method.
`versions.txt` identifies R/stats4.6.1; no contributed package was used.

`source_control_*` use the existing own14-row fixed-width-source analogue
`../psid_density/input.csv` (8 selected); they freeze independent raw preparation,
source-relative ordinals and all24 QR fits, not an author's numerical results.

Reproduce from repository root (each output directory must not already exist):

```sh
Rscript examples/mma09p1np/lowess_reference.R tests/fixtures/lowess/input.csv /tmp/lowess-main-reference
Rscript examples/mma09p1np/lowess_reference.R tests/fixtures/lowess/all_tied.csv /tmp/lowess-tied-reference
Rscript examples/mma09p1np/lowess_reference.R tests/fixtures/psid_density/input.csv /tmp/lowess-source-reference
```

The R generator independently applies the published centred-rank window/tricube
method and solves normalized weighted designs by QR; exact zero radius uses an
intercept-only design. Radius-normalized tol1e-12 does not drop a nonzero slope,
and isn't regularization. Primary native tolerance is1e-11 absolute+1e-10 relative;
integers/row ordering/fallback counts are exact. The original external4856-record
source was separately checked against its prepared CSV field-by-field and used
for177 rows/531 QR fits; none of that author data is frozen here. Historical
Stata tied-curve sort/pixel parity is not certified by these controls.
