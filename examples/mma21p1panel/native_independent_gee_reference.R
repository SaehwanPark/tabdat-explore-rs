#!/usr/bin/env Rscript
# Independent Base-R QR and raw group-score reference for native `corr(independent)`.
args <- commandArgs(trailingOnly = TRUE)
if (length(args) != 2L) stop("usage: Rscript native_independent_gee_reference.R MOM.dat fresh_reference_directory")
if (dir.exists(args[[2L]])) stop("refusing existing reference directory")

d <- read.table(args[[1L]], col.names = c("lnhr", "lnwg", "kids", "ageh", "agesq", "disab", "id", "year"))
stopifnot(nrow(d) == 5320L, all(is.finite(as.matrix(d))))
stopifnot(!anyDuplicated(d[c("id", "year")]), all(d$id == floor(d$id)), all(d$year == floor(d$year)))
d$source_row <- seq_len(nrow(d))
X <- cbind(`_cons` = 1, lnwg = d$lnwg)
y <- d$lnhr
fit <- lm.fit(X, y)
stopifnot(fit$rank == ncol(X), all(is.finite(fit$coefficients)))
beta <- unname(fit$coefficients)
residual <- drop(y - X %*% beta)
rss <- sum(residual^2)
N <- nrow(d)
ids <- unique(d$id)
G <- length(ids)
phi <- rss / N
bread <- chol2inv(qr.R(qr(X)))
working <- phi * bread
scores <- do.call(rbind, lapply(ids, function(id) {
  rows <- which(d$id == id)
  drop(crossprod(X[rows, , drop = FALSE], residual[rows]))
}))
sandwich <- bread %*% crossprod(scores) %*% bread
stopifnot(G == 532L, all(is.finite(c(phi, working, sandwich))), all(diag(working) > 0), all(diag(sandwich) > 0))

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
fit_summary <- data.frame(
  observations = N,
  groups = G,
  scale = phi,
  intercept = beta[1L],
  slope = beta[2L],
  working_correlation = 0,
  inverse_bread_scaled_score = max(abs(drop(bread %*% colSums(X * residual))) / (1 + abs(beta)))
)
covariance <- do.call(rbind, lapply(list(working = working, sandwich = sandwich), function(matrix) {
  grid <- expand.grid(column = c("_cons", "lnwg"), row = c("_cons", "lnwg"))
  data.frame(row = grid$row, column = grid$column,
             value = matrix[cbind(match(grid$row, c("_cons", "lnwg")),
                                  match(grid$column, c("_cons", "lnwg")))])
}))
covariance$kind <- rep(c("working", "sandwich"), each = nrow(covariance) / 2L)
coefficients <- rbind(inference("working", working), inference("sandwich", sandwich))
predictions <- data.frame(
  source_row = d$source_row,
  lnhr = d$lnhr,
  lnwg = d$lnwg,
  id = d$id,
  year = d$year,
  fitted_mean = drop(X %*% beta),
  residual = residual
)
write_precise(fit_summary, "fit.csv")
write_precise(covariance[c("kind", "row", "column", "value")], "covariance.csv")
write_precise(coefficients, "coefficients.csv")
write_precise(predictions, "predictions.csv")
writeLines(as.character(tools::md5sum(args[[1L]])), file.path(args[[2L]], "MOM.md5"))
cat(R.version.string, "N", N, "G", G, "slope", sprintf("%.12f", beta[2L]),
    "working-SE", sprintf("%.12f", sqrt(working[2L, 2L])),
    "raw-sandwich-SE", sprintf("%.12f", sqrt(sandwich[2L, 2L])), "\n")
