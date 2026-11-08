# Independent stable-increment controls

`python3 reference.py` regenerates the CSVs using only Python's standard-library
Decimal arithmetic at400 digits. Inputs are converted with `Decimal.from_float`
so the oracle describes the actual binary64 inputs, not nearby decimal ideals.
No native coefficients, posterior moments or numerical-helper outputs are inputs.
At x=1e-150, subtracting first-order terms still leaves about100 significant
reference digits in the order1e-300 remainder.

- `remainders.csv`:44 expm1(x)-x/log1p(x)-x cases, both signs, zero, tiny inputs,
  both series/direct transition boundaries and large/domain-edge finite inputs.
  Rust must agree within4*EPS times the expected magnitude (MIN_POSITIVE floor).
- `gaussian.csv`:12 closed-form log E exp(a*Z+b*Z²+c), Z standard normal,b<.5,
  including positive/negative~1e-24 increments, cancellation and moderate changes.
  Rust uses the existing96-node normal-weight rule, not Decimal integration;
  tolerance1e-12*abs(reference)+1e-36 allows the existing node/moment roundoff.
  Signs are mandatory above that explicit noise floor. Near-exact cancellation
  below it is not represented as a trustworthy sign decision.

The new helper is currently **test-only and not wired into the fitter**. It
computes only the nonlinear correction given a separately stabilized E[h].
These controls validate remainder/cumulant arithmetic, not the Poisson posterior,
Stein first-moment construction, fit convergence or all500 bootstrap replicates.
Those require direct-R/posterior/source replay before production integration.
Invalid/non-normalized/negative/nonfinite inputs and log-domain/overflow cases
must reject rather than clip or silently renormalize. No statistical fit gate
has changed and no public bootstrap command is implemented.
