# Robust scalar AR confidence set with two excluded instruments

## Contract

Extend `estat ar, ci [level(95)]` after a robust or one-way clustered 2SLS fit
when the retained design has exactly one endogenous regressor and exactly two
excluded instruments. Included exogenous coefficients remain unrestricted.
For candidate coefficient `b`, regress `y - b*x` on the retained included
exogenous regressors and both excluded instruments, then test the two
instrument coefficients jointly with the selected HC1 or CR1 covariance. The
confidence set is the non-rejection set for the asymptotic chi-square(2) test.

Preserve the existing single-instrument result type and JSON shape. The
two-instrument result identifies both excluded instruments, reports two degrees
of freedom and carries every non-rejection interval. Add a `multiple` shape when
there are more than two intervals. Do not use a finite search grid. If the
quartic root classification or auxiliary covariance rank is numerically
ambiguous, fail without changing the retained fit or data.

HC1 assumes independent rows. CR1 assumes independent groups and many-group
asymptotics, without a few-cluster correction or coverage guarantee. Classical
AR output, one-instrument robust AR output, and robust point-null tests remain
unchanged. Designs with more than two excluded instruments or multiple
endogenous regressors stay unsupported in this slice.

## Verification plan

- Derive all roots of the quartic non-rejection inequality from the auxiliary
  robust Wald test; classify each intervening interval from the test statistic.
- Compare endpoints and interval membership with an independent Base R
  `lm.fit`/HC1/CR1 reference for two-instrument synthetic designs, including a
  bounded/disconnected set and an all-real or empty set.
- Exercise polynomial degree reduction, near-coincident roots, singular
  auxiliary covariance, no-intercept designs, retained sample/state,
  instrument rescaling, JSON/text reporting and unsupported dimensions.
- Confirm the existing single-instrument robust confidence-set tests and CLI
  contracts remain unchanged.

This is a bounded weak-IV inference method extension, not a new textbook
illustration or a claim of instrument validity. It does not add multiple
endogenous regressors, more than two excluded instruments, robust projection,
HAC/survey covariance or few-cluster inference.
