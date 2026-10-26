# Independent profiled-amplitude solutions; no native solver or stats::nls iterate.
options(digits = 17, warn = 2)
args <- commandArgs(trailingOnly = TRUE)
if (length(args) < 1 || length(args) > 2) {
  stop("usage: Rscript weighted_reference.R NEW_OUTPUT_DIR [mma05data.asc]")
}
out <- args[1]
if (file.exists(out)) stop("reference output directory must not already exist")

fit <- function(x, y, weights) {
  n <- length(y)
  stopifnot(length(x) == n, length(weights) == n, n > 2,
            all(is.finite(x)), all(is.finite(y)), all(y > 0),
            all(is.finite(weights)), all(weights > 0))
  score <- function(d) {
    e <- exp(d * x)
    sum(weights * x * y * e) / sum(weights * y * e) -
      sum(weights * x * e^2) / sum(weights * e^2)
  }
  d <- uniroot(score, c(-2, 2), tol = 1e-14)$root
  e <- exp(d * x)
  amplitude <- sum(weights * y * e) / sum(weights * e^2)
  theta <- c(a = -log(amplitude), b = -d)
  mu <- amplitude * e
  scale <- 1 / sqrt(weights)
  z <- y / scale
  m <- mu / scale
  r <- z - m
  j <- cbind(-m, -x * m)
  rss <- sum(r^2)
  qrj <- qr(j, tol = 1e-12)
  observed <- crossprod(j) - crossprod(cbind(1, x), cbind(1, x) * (r * m))
  curvature <- min(eigen(observed, symmetric = TRUE, only.values = TRUE)$values)
  normalized_score <- max(abs(crossprod(j, r)) / (sqrt(colSums(j^2)) * sqrt(rss)))
  full_step <- max(abs(qr.coef(qrj, r)) / (1 + abs(theta)))
  stopifnot(is.finite(rss), rss > 0, qrj$rank == 2, curvature > 0,
            abs(score(d)) < 1e-13, normalized_score < 1e-12, full_step < 1e-12)
  bread <- chol2inv(qr.R(qrj))
  list(theta = theta, mu = mu, scale = scale, z = z, m = m, residual = r,
       classical = rss / (n - 2) * bread,
       hc1 = n / (n - 2) * bread %*% crossprod(j * r) %*% bread,
       diagnostics = c(scaled_rss = rss, scaled_rmse = sqrt(rss / (n - 2)),
         scaled_r_squared = 1 - rss / sum((z - mean(z))^2),
         level_rss = sum((y - mu)^2),
         level_r_squared = 1 - sum((y - mu)^2) / sum((y - mean(y))^2),
         profile_score = score(d), normalized_score = normalized_score,
         relative_full_step = full_step, min_observed_hessian = curvature))
}

cases <- list(control = data.frame(x = c(-1, 0, 1), y = c(2.2, 1.6, 2.2)))
if (length(args) == 2) {
  hash <- system2("shasum", c("-a", "256", shQuote(args[2])), stdout = TRUE)
  stopifnot(length(hash) == 1,
            substr(hash, 1, 64) == "460f5d731f1848c8695d78a1fbf0db348d0daced4b8b5b0c2734bffa20b3e31c")
  source <- read.table(args[2], col.names = c("y", "x"))
  stopifnot(nrow(source) == 10000, ncol(source) == 2)
  cases$original <- source
}

parameters <- covariance <- summaries <- projections <- list()
for (name in names(cases)) {
  data <- cases[[name]]; n <- nrow(data)
  preliminary <- fit(data$x, data$y, rep(1, n))
  mu0 <- preliminary$mu
  if (name == "control") stopifnot(max(abs(preliminary$theta - c(-log(2), 0))) < 1e-12)
  for (method in c("wnls", "fgnls")) {
    w <- if (method == "wnls") 1 / mu0 else 1 / mu0^2
    result <- fit(data$x, data$y, w)
    normalized <- fit(data$x, data$y, w * n / sum(w))
    stopifnot(max(abs(result$theta - normalized$theta)) < 1e-12,
              max(abs(result$classical - normalized$classical)) < 1e-12,
              max(abs(result$hc1 - normalized$hc1)) < 1e-12)
    if (name == "control") {
      expected_rss <- if (method == "wnls") .12 else .06
      stopifnot(max(abs(result$theta - c(-log(2), 0))) < 1e-12,
                max(abs(result$classical - diag(c(.02, .03)))) < 1e-12,
                max(abs(result$hc1 - diag(c(.02, .015)))) < 1e-12,
                abs(result$diagnostics["scaled_rss"] - expected_rss) < 1e-12,
                abs(result$diagnostics["level_rss"] - .24) < 1e-12)
    } else {
      # Printed Table 5.7 quantities are separate, coarser checks than the oracle.
      expected <- if (method == "wnls") c(1.9906, -.9961, .0225, .0098, .0359, .0224) else
        c(1.9840, -.9907, .0148, .0100, .0146, .0101)
      actual <- c(result$theta, sqrt(diag(result$classical)), sqrt(diag(result$hc1)))
      stopifnot(all(abs(actual - expected) < .00005))
    }
    projections[[length(projections) + 1]] <- data.frame(
      case = name, method = method, row = seq_len(n), x = data$x, y = data$y,
      preliminary_mean = mu0, weight = w, scale = result$scale,
      transformed_response = result$z, transformed_mean = result$m,
      transformed_residual = result$residual, level_mean = result$mu,
      level_residual = data$y - result$mu, rate = 1 / result$mu)
    for (mode in c("classical", "hc1")) {
      v <- result[[mode]]; se <- sqrt(diag(v)); t <- result$theta / se
      critical <- qt(.975, n - 2)
      parameters[[length(parameters) + 1]] <- data.frame(
        case = name, method = method, mode = mode, parameter = names(result$theta),
        estimate = result$theta, standard_error = se, t = t,
        p_value = 2 * pt(-abs(t), n - 2),
        ci_lower = result$theta - critical * se, ci_upper = result$theta + critical * se)
      for (i in 1:2) for (j in 1:2) covariance[[length(covariance) + 1]] <- data.frame(
        case = name, method = method, mode = mode, row_parameter = names(result$theta)[i],
        column_parameter = names(result$theta)[j], value = v[i, j])
      summaries[[length(summaries) + 1]] <- data.frame(
        case = name, method = method, mode = mode, n = n, df = n - 2,
        as.list(result$diagnostics))
    }
  }
}
dir.create(out, recursive = TRUE)
for (name in c("parameters", "covariance", "summaries", "projections")) {
  frame <- do.call(rbind, get(name))
  frame[] <- lapply(frame, function(x) if (is.numeric(x)) sprintf("%.17g", x) else x)
  write.table(frame, file.path(out, paste0(name, ".csv")), sep = ",",
              row.names = FALSE, col.names = TRUE, quote = TRUE)
}
writeLines(c(R.version.string, paste("stats", packageVersion("stats")),
             "Profiled fixed-first-fit weights; GN working and HC1 covariance."),
           file.path(out, "versions.txt"))
cat("Certified", length(cases), "cases with fixed preliminary weights.\n")
