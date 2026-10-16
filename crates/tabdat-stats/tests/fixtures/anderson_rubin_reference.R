# Independent reference: R 4.6.1 base stats::lm/anova/pf.
# Run from the repository root. The synthetic design is shared with the existing
# 2SLS reference; the AR reference is a nested auxiliary-regression F test.
source("crates/tabdat-stats/tests/fixtures/iv_two_stage_least_squares_reference.R")
ar <- function(null, intercept = TRUE, controls = TRUE) {
  adjusted <- y - x1 * null[1] - x2 * null[2]
  if (controls) {
    reduced <- lm(adjusted ~ w)
    full <- lm(adjusted ~ w + z1 + z2)
  } else {
    reduced <- lm(adjusted ~ 1)
    full <- lm(adjusted ~ z1 + z2)
  }
  if (!intercept) {
    reduced <- update(reduced, . ~ . - 1)
    full <- update(full, . ~ . - 1)
  }
  comparison <- anova(reduced, full)
  c(statistic = comparison$F[2], p_value = comparison$`Pr(>F)`[2],
    numerator_df = comparison$Df[2], denominator_df = df.residual(full))
}
print(list(nonzero_null = ar(c(1.7, -0.6)), zero_null = ar(c(0, 0)),
  no_intercept = ar(c(1.7, -0.6), FALSE),
  no_controls_or_intercept = ar(c(1.7, -0.6), FALSE, FALSE)), digits = 15)
