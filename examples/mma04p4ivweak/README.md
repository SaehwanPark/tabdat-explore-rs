# Cameron–Trivedi §4.9.6: schooling returns by 2SLS

The author's `DATA66.dat` is a fixed-width file and is not redistributed here.
After obtaining it from the [author's data page](https://cameron.econ.ucdavis.edu/mmabook/mmadata.html), run this from the repository root:

```sh
awk -f examples/mma04p4ivweak/data66_to_csv.awk /path/to/DATA66.dat \
  > examples/mma04p4ivweak/DATA66.csv
(cd examples/mma04p4ivweak && cargo run --manifest-path ../../Cargo.toml -- \\
  --text -f schooling_iv.td)
```

The converter selects only the variables used by the published program, uses
positions and widths from `DATA66.dct`, maps source `.` values to empty CSV
cells, and does not preserve Stata labels. The `.td` script retains
`nlsflt == 1`; `ivregress` then uses complete cases across the model variables.
The source output reports N=3,010, `grade76` coefficient 0.1324, robust SE 0.0488,
and excluded-instrument first-stage F(3, 2980)=8.07. The app reports ordinary
partial R²=0.00805582 separately from Shea's partial R²=0.00640757 for `grade76`.
Neither relevance diagnostic supplies weak-IV-robust inference.

To obtain a schooling-specific weak-IV-robust interval while leaving the
experience coefficients as endogenous nuisance parameters, run the nonrobust
projection workflow:

```sh
(cd examples/mma04p4ivweak && cargo run --manifest-path ../../Cargo.toml -- \
  --text -f schooling_iv_ar_ci.td)
```

On DATA66 this retains N=3,010 and reports the 95% `grade76` projection
[-0.014387552440543, 0.502853235901166]. This is an analytic projection of the
joint homoskedastic AR region, so coverage is at least 95% and can be conservative.
Exact finite-sample coverage also requires conditional normality. The published
program reports conventional robust 2SLS standard errors; this method currently
requires a separate nonrobust fit and does not inherit that robust covariance
claim.

For a development-only independent R reference (not a runtime dependency):

```sh
Rscript examples/mma04p4ivweak/reference_shea.R /path/to/DATA66.csv
Rscript examples/mma04p4ivweak/reference_ar_projected.R /path/to/DATA66.csv
```

This uses the same selected complete-case sample, residualizes each original and
instrument-projected design column on the remaining columns, and compares their
residual sums of squares. It checks N and the author's rounded Shea statistic,
prints all three Shea values and the retained original CSV row indices. The
synthetic R reference and Rust tests additionally cover covariance invariance,
no-intercept designs and outcome-independent diagnostics.

The projected-AR reference independently profiles `exp76` and `expsq76` with
base-R `optim`, obtains each auxiliary residual sum of squares with `lm.fit`, and
brackets the two grade endpoints with `uniroot`. Source files are downloaded
temporarily for validation and are not included in the repository.
