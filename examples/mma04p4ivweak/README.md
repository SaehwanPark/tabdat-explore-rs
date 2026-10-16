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
and excluded-instrument first-stage F(3, 2980)=8.07. The app also reports
conventional first-stage partial R², **not** Shea's statistic or weak-IV-robust
inference.
