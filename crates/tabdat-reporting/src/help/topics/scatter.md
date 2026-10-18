# scatter

Installed syntax:
`scatter yvar xvar, saving("file.svg") noopen [lines(var1 var2 var3) jitter(number) title("text")]`

Save an SVG scatter plot of current numeric columns, optionally overlaying up to
three numeric line columns (for example, previously generated fitted values).
Both saving() and noopen are required. Use a new .svg path: there is no replace
option, implicit output filename or automatic opening. Open the resulting SVG
in an SVG-capable viewer/browser yourself. PNG and other plot families remain
uninstalled; accepting older bare/saving-only syntax does not make it executable.

Examples (after loading data with outcome and score):
- `scatter outcome score, saving("scatter.svg") noopen`
- `regress outcome score`
- `predict fitted`
- `scatter outcome score, lines(fitted) saving("fit.svg") noopen`
- `scatter outcome score, lines(fitted) jitter(0.03) title("Observed and fitted values") saving("jittered.svg") noopen`

Selection and ordering:
- Numeric coordinates use float64 conversions, not lossless wide-integer or
  decimal identities. The dataset itself is not converted or modified.
- Scatter marks require finite x and y. Each line independently requires finite
  x and its own y; these are display samples, not a new estimation sample.
- Missing/nonfinite x is omitted. At a finite x, a missing/nonfinite line value
  breaks that line. Line vertices sort internally by ascending x, with ties in
  original candidate-row order. Scatter marks keep original row order.
- At least one finite scatter pair and, for each requested line, at least one
  adjacent finite pair after sorting are required. A requested series never
  silently disappears. Lines must name one to three unique columns.
- At most 10,000 current candidate rows; select an explicit plotting cohort
  first for larger datasets. Labels/title must have 1 to 80 characters and valid
  XML text. Variable labels supply axis/legend labels, falling back to names.
  The fixed layout has no general long-label/font-metric layout guarantee.
- Axes include all displayed values with nominal 5% padding, including OLS
  fitted values outside [0,1]. Constant ranges expand; unrepresentable finite
  ranges fail rather than produce invalid geometry. Tick text is rounded;
  embedded coordinates retain float64 precision.

jitter() is an optional finite nonnegative y-only display amplitude in outcome
units; default zero. Candidate row i (zero-based, before exclusions/sorting)
gets amplitude * ((73*i mod 251)/125 - 1). This deterministic ordinal pattern
is not Stata's percentage/RNG jitter, does not use set seed, does not change raw
outcomes/predictions and supplies no simulation, inference or anonymization.
Changing current row order changes the display pattern.

The SVG embeds raw float64 coordinates, display coordinates and original
candidate-row indices. Jitter does NOT hide the underlying data: review an
artifact before sharing it. It contains generated markup only, with no external
assets, bundled fonts, scripts or arbitrary SVG input. Fonts/layout depend on
the viewer; there is no pixel/byte parity claim with Stata/Python.

The command preserves data, labels, row order and the retained model on success
and failure. It renders completely before filesystem effects, then publishes a
complete sibling stage with a no-clobber hard link. Existing files/directories/
symlinks are never replaced. This requires a local hard-link-capable filesystem;
there is no crash-durability guarantee, and filesystem/crash failures can leave
an owned hidden stage. Parent directories may be created only after rendering.

JSON/text report the artifact path/format/dimensions, candidate rows, finite
scatter/line counts and per-series exclusions, coordinate precision and jitter
convention. No estimator is fitted, modified or inferred from arbitrary line
columns. The native Figure14.1 workflow and independent development-only checks
are in examples/mma14p1binary/figure.td and its README.
