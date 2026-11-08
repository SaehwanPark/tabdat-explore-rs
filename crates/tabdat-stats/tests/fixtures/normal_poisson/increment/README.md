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

`Rscript posterior_reference.R NEW_OUTPUT_DIR` separately generates18 direct
improper-integral Poisson group controls: six old posterior shapes (zero totals,
singletons/3/5 rows, total100000 and mean1e-5) times positive/negative1e-3 and
positive1e-12 parameter shifts. Exact17-digit row inputs preserve physical
binary64 y/mu/delta values. R integrates complete-data directional scores and
expm1(log density ratio) inside the old posterior; it does not use GH, native
moments or the Stein first-moment identity. Centered curvature coordinates avoid
large marginal-level subtraction. R4.6.2 reports maximum quadrature abs.error
1.69e-12 (across normalized mass/score/ratio integrals, not a certified ratio
error bound); requested relative tolerance2e-12. Fixed Rust directional-score/
log-ratio gate1e-9*abs(reference)+1e-24 allows quadrature/moment propagation while
retaining meaningful relative tests for the~1e-12 changes. All18 signs must match.

The helper is currently **test-only and not wired into the fitter**. Cumulant
arithmetic takes a separately stabilized E[h]; the group helper constructs that
mean using the Poisson/Stein/remainder identities and the existing96-node rule.
These controls validate that group construction against independent integration,
not fit convergence or all500 bootstrap replicates. Source replay, ordinary/
refined-rule regressions and no-progress rejection remain before production
integration.
Invalid/non-normalized/negative/nonfinite inputs and log-domain/overflow cases
must reject rather than clip or silently renormalize. No statistical fit gate
has changed and no public bootstrap command is implemented.
