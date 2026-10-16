# Independent scalar AR inversion reference: R 4.6.1 stats::lm.fit/qf/polyroot.
# No production R dependency. Four inspectable, orthogonal synthetic designs.
# Invert RSS_restricted(b) - (1 + q*Fcrit/df)*RSS_full(b) <= 0.
n <- 40L
i <- seq_len(n)
basis <- function(k) cos(2 * pi * k * i / n)
w <- basis(5)
z <- cbind(1, w, basis(1), basis(2))
restricted <- cbind(1, w)
reference <- function(strength, direct_z1, direct_z2, intercept = TRUE, level = 95) {
  x <- strength * basis(1) + basis(3)
  y <- 1.5 * x + 0.5 * w + direct_z1 * basis(1) + direct_z2 * basis(2) + basis(4)
  full_design <- if (intercept) z else z[, -1]
  restricted_design <- if (intercept) restricted else restricted[, -1, drop = FALSE]
  r <- lm.fit(restricted_design, cbind(y, x))$residuals
  u <- lm.fit(full_design, cbind(y, x))$residuals
  df <- n - ncol(full_design)
  critical <- qf(level / 100, 2, df)
  k <- 1 + 2 * critical / df
  coefficients <- c(a = sum(r[, 2]^2) - k * sum(u[, 2]^2),
    b = -2 * (sum(r[, 1] * r[, 2]) - k * sum(u[, 1] * u[, 2])),
    c = sum(r[, 1]^2) - k * sum(u[, 1]^2))
  roots <- polyroot(rev(coefficients))
  list(critical = critical, df = df, coefficients = coefficients, roots = roots)
}
print(list(bounded = reference(2, 0, 0), all_real = reference(.01, 0, 0),
  disjoint = reference(.1, 2, 0), empty = reference(2, 0, 10),
  no_intercept_90 = reference(2, 0, 0, FALSE, 90)), digits = 16)
