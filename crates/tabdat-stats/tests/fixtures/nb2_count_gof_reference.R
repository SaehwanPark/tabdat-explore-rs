# Independent Base R/MASS reference for the score-adjusted NB2 grouped CGF test.
# Synthetic fixture: Rscript nb2_count_gof_reference.R > nb2_count_gof_reference.csv
# MMA sample: Rscript nb2_count_gof_reference.R mma20p1count.asc > mma20p1count_gof_reference.csv
options(digits = 17)
args <- commandArgs(trailingOnly = TRUE)
stopifnot(length(args) %in% c(0L, 1L), requireNamespace("MASS", quietly = TRUE))

nb2_evaluate <- function(par, X, y) {
  k <- ncol(X)
  r <- exp(-par[k + 1L])
  eta <- drop(X %*% par[seq_len(k)])
  mu <- exp(eta)
  sr <- digamma(y + r) - digamma(r) + log(r / (r + mu)) + 1 - (r + y) / (r + mu)
  scores <- cbind(X * (r * (y - mu) / (r + mu)), lnalpha = -r * sr)
  cross <- drop(crossprod(X, r * mu * (y - mu) / (r + mu)^2))
  information <- rbind(
    cbind(crossprod(X, X * (r * mu * (r + y) / (r + mu)^2)), cross),
    c(cross, sum(-r * sr - r^2 * (trigamma(y + r) - trigamma(r) + 1 / r -
      1 / (r + mu) + (y - mu) / (r + mu)^2)))
  )
  list(mu = mu, eta = eta, scores = scores,
    score = colSums(scores), information = information,
    log_likelihood = sum(dnbinom(y, size = r, mu = mu, log = TRUE)))
}

polish_nb2 <- function(par, X, y) {
  converged <- FALSE
  for (iteration in seq_len(100L)) {
    evaluated <- nb2_evaluate(par, X, y)
    step <- drop(solve(evaluated$information, evaluated$score))
    if (max(abs(step)) < 1e-14) {
      converged <- TRUE
      break
    }
    par <- unname(par + step)
  }
  stopifnot(converged)
  list(par = unname(par), evaluated = nb2_evaluate(par, X, y), iteration = iteration)
}

grouped_gof <- function(case, y, predictors) {
  X <- cbind(intercept = 1, as.matrix(predictors))
  stopifnot(length(y) > ncol(X) + 1L, all(is.finite(y)),
    all(y >= 0 & y == floor(y)), max(y) <= 100000,
    all(is.finite(X)), qr(X)$rank == ncol(X))
  frame <- data.frame(y = y, predictors)
  initial <- MASS::glm.nb(y ~ ., data = frame,
    control = glm.control(epsilon = 1e-13, maxit = 100L))
  stopifnot(isTRUE(initial$converged))
  fitted <- polish_nb2(c(unname(coef(initial)), lnalpha = -log(initial$theta)), X, y)
  par <- fitted$par
  evaluated <- fitted$evaluated
  alpha <- exp(tail(par, 1L))
  r <- exp(-tail(par, 1L))
  mu <- evaluated$mu
  stopifnot(all(is.finite(mu)), all(mu > 0), is.finite(alpha), alpha > 0,
    max(abs(evaluated$score)) < 1e-8)

  tail_from <- 10L
  probability <- cbind(
    vapply(0:(tail_from - 1L), function(k) {
      dnbinom(k, size = r, mu = mu)
    }, numeric(length(y))),
    pnbinom(tail_from - 1L, size = r, mu = mu, lower.tail = FALSE)
  )
  stopifnot(all(is.finite(probability)), all(probability >= 0),
    max(abs(rowSums(probability) - 1)) < 5e-14)
  observed_cell <- pmin(y, tail_from)
  moments <- vapply(0:(tail_from - 1L), function(j) {
    as.numeric(observed_cell == j) - probability[, j + 1L]
  }, numeric(length(y)))
  auxiliary <- cbind(moments, evaluated$scores)
  norms <- sqrt(colSums(auxiliary^2))
  stopifnot(all(is.finite(norms)), all(norms > 0))
  auxiliary <- sweep(auxiliary, 2L, norms, "/")
  qr_aux <- qr(auxiliary)
  stopifnot(qr_aux$rank == ncol(auxiliary))
  regression <- lm.fit(auxiliary, rep(1, length(y)), tol = 1e-12)
  stopifnot(regression$rank == ncol(auxiliary))
  statistic <- sum(regression$fitted.values^2)
  degrees_of_freedom <- ncol(probability) - 1L
  p_value <- pchisq(statistic, df = degrees_of_freedom, lower.tail = FALSE)

  rows <- list()
  emit <- function(quantity, i = 0L, j = 0L, value) {
    rows[[length(rows) + 1L]] <<- data.frame(case = case,
      quantity = quantity, i = i, j = j, value = unname(value))
  }
  for (bin in 0:tail_from) {
    observed_count <- sum(observed_cell == bin)
    fitted_fraction <- mean(probability[, bin + 1L])
    emit("observed_count", bin, 0, observed_count)
    emit("observed_fraction", bin, 0, observed_count / length(y))
    emit("fitted_fraction", bin, 0, fitted_fraction)
    emit("expected_count", bin, 0, length(y) * fitted_fraction)
  }
  for (parameter in seq_along(par)) emit("coefficient", parameter - 1L, 0L, par[parameter])
  emit("observation_count", value = length(y))
  emit("alpha", value = alpha)
  emit("log_likelihood", value = evaluated$log_likelihood)
  emit("statistic", value = statistic)
  emit("degrees_of_freedom", value = degrees_of_freedom)
  emit("p_value", value = p_value)
  emit("auxiliary_regressor_count", value = ncol(auxiliary))
  result <- do.call(rbind, rows)
  cat("R ", as.character(getRversion()), "; MASS ",
    as.character(packageVersion("MASS")),
    "; case=", case, "; N=", length(y), "; full-rank auxiliary columns=",
    ncol(auxiliary), "; max abs score=", format(max(abs(evaluated$score)), digits = 17),
    "; CGF=", format(statistic, digits = 17), "; df=", degrees_of_freedom,
    "; p=", format(p_value, digits = 17), ".\n", sep = "", file = stderr())
  result
}

if (length(args) == 1L) {
  source_sha256 <- "c8b1eee8ae1f637f4d4e4c6aa6b2dcbf2b8aa98f276b9369adcb4f02a528c58a"
  checksum <- system2("shasum", c("-a", "256", shQuote(args[1L])), stdout = TRUE,
    stderr = TRUE)
  stopifnot(length(checksum) >= 1L,
    identical(strsplit(checksum[1L], "[[:space:]]+")[[1L]][1L], source_sha256))
  values <- scan(args[1L], quiet = TRUE)
  stopifnot(length(values) == 20186L * 18L)
  data <- as.data.frame(matrix(values, ncol = 18L, byrow = TRUE))
  names(data) <- c("MDU", "LC", "IDP", "LPI", "FMDE", "PHYSLIM", "NDISEASE",
    "HLTHG", "HLTHF", "HLTHP", "LINC", "LFAM", "EDUCDEC", "AGE", "FEMALE",
    "CHILD", "FEMCHILD", "BLACK")
  stopifnot(nrow(data) == 20186L, sum(data$MDU) == 57746,
    sum(data$MDU == 0) == 6308, max(data$MDU) == 77)
  result <- grouped_gof("mma20p1count", data$MDU, data[-1L])
  cat("Exact author sample and ASCII checksum passed; data remain external.\n",
    file = stderr())
} else {
  y <- c(0, 0, 2, 0, 0, 3, 0, 3, 1, 3, 8, 3, 0, 0, 0, 2, 1, 2, 0, 0, 2, 1,
    1, 2, 0, 5, 4, 1, 1, 0, 0, 2, 2, 0, 1, 0)
  x <- seq(-2, 2, length.out = length(y))
  z <- rep(c(0, 1, 0.4), length.out = length(y))
  result <- grouped_gof("multiple", y, data.frame(x = x, z = z))
}
write.csv(result, stdout(), row.names = FALSE, quote = FALSE)
