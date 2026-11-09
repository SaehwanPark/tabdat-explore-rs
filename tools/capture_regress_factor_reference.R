# Development-only independent reference for the bounded `regress` factor design.
#
# Scope: one-way categorical factors on a small synthetic sample that is not
# book data. Cases cover the automatic and explicit reference, a text factor, an
# integer code, a boolean flag, `noconstant`, and current-data prediction where a
# cell is missing or holds a level absent from the estimation sample.
#
# The reference is a flat JSON object of parallel arrays so that this control
# needs no extra R packages.
#
# Usage: Rscript capture_regress_factor_reference.R OUTPUT_DIRECTORY
args <- commandArgs(trailingOnly = TRUE)
stopifnot(length(args) == 1)
out <- args[1]
dir.create(out, showWarnings = FALSE, recursive = TRUE)

data <- data.frame(
  x = as.numeric(1:12),
  region = c(
    "West", "East", "North", "West", "East", "North",
    "West", "East", "North", "East", NA, "West"
  ),
  code = c(10L, 2L, 10L, 2L, 10L, 2L, 2L, 10L, 2L, 10L, 2L, 10L),
  flag = c(TRUE, FALSE, TRUE, FALSE, TRUE, FALSE, FALSE, TRUE, FALSE, TRUE, FALSE, TRUE)
)
data$y <- 1.0 + 0.5 * data$x +
  c(0.2, -0.1, 0.3, -0.2, 0.1, -0.3, 0.25, -0.15, 0.05, -0.25, 0.15, -0.05)
write.csv(data, file.path(out, "input.csv"), row.names = FALSE, na = "")

# R orders character factors alphabetically and integer codes numerically. Set
# the levels explicitly so this control does not depend on session defaults.
data$region <- factor(data$region, levels = sort(unique(na.omit(data$region))))
data$flag <- as.logical(data$flag)

# Parallel vectors describing every fitted parameter in command order.
cases <- character(0)
parameters <- character(0)
coefficients <- numeric(0)
standard_errors <- numeric(0)
p_values <- numeric(0)
residual_degrees_of_freedom <- integer(0)
r_squared <- numeric(0)
observations <- integer(0)

record <- function(case, fit, terms) {
  matrix <- summary(fit)$coefficients
  cases <<- c(cases, rep(case, length(terms)))
  parameters <<- c(parameters, terms)
  coefficients <<- c(coefficients, as.numeric(coef(fit)[terms]))
  standard_errors <<- c(standard_errors, as.numeric(matrix[terms, "Std. Error"]))
  p_values <<- c(p_values, as.numeric(matrix[terms, "Pr(>|t|)"]))
  residual_degrees_of_freedom <<- c(
    residual_degrees_of_freedom, rep(fit$df.residual, length(terms))
  )
  r_squared <<- c(r_squared, rep(as.numeric(summary(fit)$r.squared), length(terms)))
  observations <<- c(observations, rep(nobs(fit), length(terms)))
}

reference_fit <- lm(y ~ x + region, data)
record("i.region", reference_fit, c("(Intercept)", "x", "regionNorth", "regionWest"))
record(
  "ib(West).region",
  lm(y ~ x + region, transform(data, region = relevel(region, ref = "West"))),
  c("(Intercept)", "x", "regionEast", "regionNorth")
)
record("i.code", lm(y ~ x + factor(code), data), c("(Intercept)", "x", "factor(code)10"))
record("i.flag", lm(y ~ x + flag, data), c("(Intercept)", "x", "flagTRUE"))
record(
  "noconstant",
  lm(y ~ x + region - 1, data),
  c("x", "regionEast", "regionNorth", "regionWest")
)

# Current-data prediction. A missing cell or a level absent from the fit has no
# retained contrast, which the installed command reports as NULL.
predict_region <- as.character(data$region)
predict_region[11] <- "South"
retained <- levels(data$region)
raw <- coef(reference_fit)
predictions <- vapply(seq_len(nrow(data)), function(row) {
  level <- predict_region[row]
  if (is.na(level) || !(level %in% retained)) {
    return(NA_real_)
  }
  value <- raw[["(Intercept)"]] + raw[["x"]] * data$x[row]
  for (other in retained[-1]) {
    value <- value + raw[[paste0("region", other)]] * as.numeric(level == other)
  }
  value
}, numeric(1))

json_string <- function(text) {
  escaped <- gsub("\\", "\\\\", text, fixed = TRUE)
  escaped <- gsub("\"", "\\\"", escaped, fixed = TRUE)
  paste0("\"", escaped, "\"")
}

json_array <- function(values) {
  if (is.character(values)) {
    items <- vapply(values, json_string, character(1))
  } else {
    items <- vapply(values, function(value) {
      if (is.na(value)) "null" else format(value, digits = 17, scientific = FALSE, trim = TRUE)
    }, character(1))
  }
  paste0("[", paste0(items, collapse = ", "), "]")
}

lines <- c(
  "{",
  paste0("  \"method\": ", json_string(
    "R lm treatment contrasts, classical covariance, t inference"
  ), ","),
  paste0("  \"sample\": ", json_string(
    "complete cases of y, x and the declared factor; original row order"
  ), ","),
  paste0("  \"tolerance\": ", json_string(
    "max(1e-10 absolute, 1e-9 * abs(expected)); no display rounding"
  ), ","),
  "  \"cases\": [",
  paste0(vapply(cases, json_string, character(1)), collapse = ",\n"),
  "  ],",
  paste0("  \"parameters\": ", json_array(parameters), ","),
  paste0("  \"coefficients\": ", json_array(coefficients), ","),
  paste0("  \"standard_errors\": ", json_array(standard_errors), ","),
  paste0("  \"p_values\": ", json_array(p_values), ","),
  paste0(
    "  \"residual_degrees_of_freedom\": ",
    json_array(residual_degrees_of_freedom), ","
  ),
  paste0("  \"r_squared\": ", json_array(r_squared), ","),
  paste0("  \"observations\": ", json_array(observations), ","),
  paste0("  \"predictions\": ", json_array(predictions)),
  "}"
)
con <- file(file.path(out, "regress_factor_reference.json"), open = "wt")
writeLines(lines, con)
close(con)
