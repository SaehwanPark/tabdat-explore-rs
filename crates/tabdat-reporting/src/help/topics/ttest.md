# ttest

How to invoke:
- `ttest x == 0`: one-sample test of the mean against a numeric null.
- `ttest x == y`: paired test of the mean row-wise difference x minus y against zero.
- `ttest x, by(group)`: independent two-group test, assuming equal variances (pooled).
- `ttest x, by(group) welch`: independent unequal-variance test (Welch).

What it does:
Compare a numeric mean or mean difference using a Student-t reference distribution.
`unequal` is an alias for the flag `welch`; it applies only to the by(group) form.
Load data with `use` first. No prior regression is required. Both successful and
failed tests leave the active data and cached regression unchanged.

Samples and contrasts:
- One-sample tests retain finite, non-NULL observations of x.
- Paired tests retain only rows where both x and y are finite and non-NULL.
- Independent tests retain finite, non-NULL x values with non-NULL group keys.
  Exactly two rendered group labels must remain. Labels are sorted as strings;
  the reported contrast is the first label minus the second, not input-row order
  or numeric key order. Check the displayed names before interpreting the sign.
- Unknown/nonnumeric tested variables, no usable observations, or a group count
  other than two fail. A singleton sample can display summaries with undefined
  inference; do not treat unavailable statistics as evidence for the null.

Results:
Interactive and `--text` output show sample counts, means, standard deviations,
standard errors, fixed 95% Student-t intervals, the t statistic, reference degrees
of freedom, and left-/two-sided/right-tail probabilities. Welch reference degrees
of freedom can be fractional. For one-sample tests the reported mean/interval is
for x itself, not x minus the null; the t statistic tests the stated null mean.
Independent difference-row N is not an observation count and displays as `NA`;
use the separate group counts. Undefined statistics display as `NA`, or JSON null.
Default batch/script success remains quiet; `--json` exposes `TtestResult` for
these same forms, including nested script streams.

Limits:
Only the four forms above are installed. No if qualifier, weights, robust/cluster
options, configurable confidence level, or independent test of two separate
columns is provided. `ttest x == y` is paired, not independent. Use only a numeric
literal or one variable on the right of `==`, not an expression. Variable names
containing spaces can be backtick-quoted.
