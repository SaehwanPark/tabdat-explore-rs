#!/usr/bin/env Rscript
# Dense Base-R reference for the bounded balanced unstructured Gaussian GEE.
args <- commandArgs(trailingOnly = TRUE)
if (length(args) != 2L) stop("usage: Rscript unstructured_gee_reference.R MOM.dat fresh_reference_directory")
if (dir.exists(args[[2L]])) stop("refusing existing reference directory")

d <- read.table(args[[1L]], col.names = c("lnhr", "lnwg", "kids", "ageh", "agesq", "disab", "id", "year"))
stopifnot(nrow(d) > 0L, nrow(d) <= 5320L, all(is.finite(as.matrix(d))))
stopifnot(!anyDuplicated(d[c("id", "year")]), all(d$id == floor(d$id)), all(d$year == floor(d$year)))
d$source_row <- seq_len(nrow(d))
time_values <- sort(unique(d$year))
ids <- sort(unique(d$id))
G <- length(ids)
T <- length(time_values)
N <- nrow(d)
stopifnot(G >= 3L, G <= 532L, T >= 2L, T <= 10L, all(table(d$id) == T))
sorted <- d[order(d$id, d$year), ]
X <- cbind(`_cons` = 1, lnwg = sorted$lnwg)
y <- sorted$lnhr
groups <- lapply(ids, function(id) which(sorted$id == id))

residual_at <- function(beta) drop(y - X %*% beta)
correlation_at <- function(residual) {
  E <- matrix(residual, nrow = G, ncol = T, byrow = TRUE)
  # The empirical Pearson covariance is standardized by its time-specific
  # diagonal moments. No nearest-PD repair or correlation clipping is applied.
  covariance <- crossprod(E) / G
  R <- cov2cor(covariance)
  chol(R)
  R
}
gls_at <- function(R) {
  H <- matrix(0, ncol(X), ncol(X))
  q <- numeric(ncol(X))
  Ri <- solve(R)
  for (ix in groups) {
    Z <- X[ix, , drop = FALSE]
    H <- H + crossprod(Z, Ri %*% Z)
    q <- q + drop(crossprod(Z, Ri %*% y[ix]))
  }
  list(beta = drop(solve(H, q)), bread = solve(H))
}

beta <- unname(coef(lm(lnhr ~ lnwg, data = sorted)))
converged <- FALSE
for (iterations in seq_len(100L)) {
  R <- correlation_at(residual_at(beta))
  next_fit <- gls_at(R)
  change <- max(abs(next_fit$beta - beta) / (1 + abs(beta)))
  beta <- next_fit$beta
  if (change <= 1e-6) {
    converged <- TRUE
    break
  }
}
stopifnot(converged, iterations < 100L)

residual <- residual_at(beta)
R <- correlation_at(residual)
final_fit <- gls_at(R)
max_abs_scaled_score <- max(abs(final_fit$beta - beta) / (1 + abs(beta)))
stopifnot(is.finite(max_abs_scaled_score), max_abs_scaled_score <= 1e-6)
bread <- final_fit$bread
phi <- sum(residual^2) / (N - ncol(X))
working <- phi * bread
scores <- do.call(rbind, lapply(groups, function(ix) {
  Z <- X[ix, , drop = FALSE]
  drop(crossprod(Z, solve(R, residual[ix])))
}))
sandwich <- bread %*% crossprod(scores) %*% bread

inference <- function(kind, covariance) {
  standard_error <- sqrt(diag(covariance))
  statistic <- beta / standard_error
  data.frame(
    covariance = kind,
    parameter = c("_cons", "lnwg"),
    estimate = beta,
    standard_error = standard_error,
    statistic = statistic,
    p_value = 2 * pnorm(-abs(statistic)),
    confidence_lower = beta - qnorm(.975) * standard_error,
    confidence_upper = beta + qnorm(.975) * standard_error,
    wald_statistic = c(NA_real_, beta[2L]^2 / covariance[2L, 2L]),
    wald_p_value = c(NA_real_, pchisq(beta[2L]^2 / covariance[2L, 2L], 1L, lower.tail = FALSE))
  )
}

dir.create(args[[2L]])
write_precise <- function(frame, name) {
  text <- as.data.frame(lapply(frame, function(value) {
    if (is.numeric(value)) ifelse(is.na(value), "", sprintf("%.17g", value)) else value
  }))
  write.table(text, file.path(args[[2L]], name), sep = ",", quote = FALSE,
              row.names = FALSE, col.names = TRUE, na = "")
}
fit <- data.frame(
  observations = N,
  entities = G,
  periods = T,
  scale = phi,
  intercept = beta[1L],
  slope = beta[2L],
  iterations = iterations,
  max_abs_scaled_score = max_abs_scaled_score
)
correlation <- expand.grid(period_row = seq_len(T), period_column = seq_len(T))
correlation$value <- as.vector(R)
covariance <- do.call(rbind, lapply(list(working = working, sandwich = sandwich), function(matrix) {
  expand.grid(row = c("_cons", "lnwg"), column = c("_cons", "lnwg")) |>
    transform(value = as.vector(matrix))
}))
covariance$kind <- rep(c("working", "sandwich"), each = ncol(X)^2)
coefs <- rbind(inference("working", working), inference("sandwich", sandwich))
fitted <- drop(X %*% beta)
predictions <- data.frame(
  source_row = sorted$source_row,
  lnhr = sorted$lnhr,
  lnwg = sorted$lnwg,
  entity = sorted$id,
  time = sorted$year,
  fitted_mean = fitted,
  residual = residual
)
predictions <- predictions[order(predictions$source_row), ]
write_precise(fit, "fit.csv")
write_precise(correlation, "correlation.csv")
write_precise(covariance[c("kind", "row", "column", "value")], "covariance.csv")
write_precise(coefs, "coefficients.csv")
write_precise(predictions, "predictions.csv")
writeLines(as.character(tools::md5sum(args[[1L]])), file.path(args[[2L]], "MOM.md5"))
cat(R.version.string, "N", N, "G", G, "T", T, "iterations", iterations,
    "slope", sprintf("%.12f", beta[2L]), "working-SE", sprintf("%.12f", sqrt(working[2L, 2L])),
    "sandwich-SE", sprintf("%.12f", sqrt(sandwich[2L, 2L])), "\n")
