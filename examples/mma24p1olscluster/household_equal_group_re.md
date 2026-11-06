# Household equal-commune between-component RE GLS — in progress

Bounded next slice: an opt-in `xtreg y xvars, re id(group) time(key)
vcmethod(equal_group_between)` for the graduate user reproducing §24.7 household
RE columns. The existing trace-corrected Swamy–Arora default remains unchanged;
`vcmethod(swamy_arora)` will name it explicitly. The alternate estimator uses
within RSS/(N−G−p) for sigma_e², unweighted equal-commune between RSS/(G−K)
minus sigma_e²*mean(1/T_g) for the raw unit variance, nonnegative truncation,
per-size theta and the existing transformed RSS/(N−K) model covariance and
normal/chi-square inference. It is a named formula, not a general Stata mode.

Acceptance cases before merge:

- Same pinned ASCII sample/design: 5,006 cases/194 communes, six numeric slopes;
  independently check complete covariance, components, normal95/Wald inference
  and every ordered population xb/residual row against direct R QR.
- Separately run the pinned DTA through the explicit method and check the 14
  printed Stata-8 coefficient/SE facts at their displayed rounding precision.
- Independent synthetic unbalanced interior/truncated-zero cases with singleton
  and missing outcome, plus balanced equivalence to the existing estimator.
- Terminal/CLI-v1/script output must name the selected method; retained model
  and population predictions survive summary/reporting and export collision;
  omitted/default method preserves supported workflows and numerical results.
- Reject malformed/unknown/duplicate methods, FE/ML combinations and alternate
  robust/cluster covariance; rank/variance/keys failures must not export or
  retain a failed model, and an export collision must preserve its owner/source.

Exclusions: general Stata parity, complex-survey/causal inference, GEE/ML,
conditional unit effects, alternate-method robust/cluster covariance,
variance-component uncertainty and Hausman post-estimation. Full Table 24.4,
individual FE/RE and remaining chapter/roadmap outcomes remain required.

**Status:** contract only. Implementation, references, tests and review pending.
