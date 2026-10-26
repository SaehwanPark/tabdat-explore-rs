# Independent closed/profiled solutions and analytic derivatives, not an expression
# interpreter or the native GN solver. Strict stats::nls can stall near this loss's
# floating-point floor; a failed nls iterate is deliberately not a reference.
options(digits = 17, warn = 2)
args <- commandArgs(trailingOnly = TRUE)
if (length(args) < 1 || length(args) > 2) {
  stop("usage: Rscript reference.R NEW_OUTPUT_DIR [mma05data.asc]")
}
out <- args[1]
if (file.exists(out)) stop("reference output directory must not already exist")
dir.create(out, recursive = TRUE)

write_exact <- function(frame, name) {
  frame[] <- lapply(frame, function(x) {
    if (is.numeric(x)) ifelse(is.na(x), NA_character_, sprintf("%.17g", x)) else x
  })
  write.table(frame, file.path(out, name), sep = ",", row.names = FALSE,
              col.names = TRUE, quote = TRUE, na = "NA")
}
case <- function(name, expression, x, y, start, evaluate, solution = start,
                 status = "fit", predictors = "x") {
  list(name = name, expression = expression, x = x, y = y, start = start,
       evaluate = evaluate, solution = solution, status = status,
       predictors = predictors)
}
linear <- function(theta, x) {
  p <- length(theta)
  j <- if (p == 1) matrix(x, ncol = 1) else cbind(1, x)
  list(mean = as.vector(j %*% theta), j = j, h = array(0, c(length(x), p, p)))
}
squared <- function(theta, x) {
  h <- array(0, c(length(x), 2, 2)); h[, 1, 1] <- 2
  list(mean = theta[1]^2 + theta[2] * x,
       j = cbind(rep(2 * theta[1], length(x)), x), h = h)
}
exp_slope <- function(theta, x) {
  mu <- exp(theta[1] * x)
  list(mean = mu, j = matrix(mu * x, ncol = 1),
       h = array(mu * x^2, c(length(x), 1, 1)))
}
exp_constant <- function(theta, x) {
  mu <- rep(exp(theta[1]), length(x))
  list(mean = mu, j = matrix(mu, ncol = 1), h = array(mu, c(length(x), 1, 1)))
}
rational <- function(theta, x) {
  denominator <- theta[2] + x
  h <- array(0, c(length(x), 2, 2))
  h[, 1, 2] <- h[, 2, 1] <- -1 / denominator^2
  h[, 2, 2] <- 2 * theta[1] / denominator^3
  list(mean = theta[1] / denominator,
       j = cbind(1 / denominator, -theta[1] / denominator^2), h = h)
}
stationary_nonminimum <- function(theta, x) {
  list(mean = theta[1]^2 + theta[1] * x,
       j = matrix(2 * theta[1] + x, ncol = 1),
       h = array(2, c(length(x), 1, 1)))
}
source_exponential <- function(theta, x) {
  z <- cbind(1, x); mu <- exp(-as.vector(z %*% theta))
  h <- array(0, c(length(x), 2, 2))
  for (a in 1:2) for (b in 1:2) h[, a, b] <- mu * z[, a] * z[, b]
  list(mean = mu, j = -z * mu, h = h)
}

x <- c(-2, 1, -1, 4, 0, 3, 2)
y <- c(-3.8, .9, -2.1, 6.4, -.3, 4.4, 3.1)
ols <- as.vector(lm.fit(cbind(1, x), y)$coefficients)
y_squared <- c(2.5, 4.6, 3.1, 7, 4.2, 6, 5.5)
squared_ols <- as.vector(lm.fit(cbind(1, x), y_squared)$coefficients)
stopifnot(squared_ols[1] > 0)
xs <- c(-1, -.5, 0, .5, 1, 1.5); ys <- c(.4, .6, 1, 1.1, 2.4, 2.7)
slope <- uniroot(function(a) sum((ys - exp(a * xs)) * exp(a * xs) * xs),
                  c(0, 2), tol = 1e-14)$root
xr <- c(0, .5, 1, 2, 3, 5); yr <- c(2.2, 1.7, 1.4, 1.1, .8, .6)
profile_amplitude <- function(b) {
  g <- 1 / (b + xr); sum(yr * g) / sum(g^2)
}
br <- uniroot(function(b) {
  a <- profile_amplitude(b)
  sum((yr - a / (b + xr)) * (-a / (b + xr)^2))
}, c(.2, 6), tol = 1e-14)$root
cases <- list(
  case("linear", "a+b*x", x, y, c(a = 1, b = 0), linear, ols),
  case("squared_positive", "a*a+b*x", x, y_squared, c(a = 1, b = 0),
       squared, c(sqrt(squared_ols[1]), squared_ols[2])),
  case("squared_negative", "a*a+b*x", x, y_squared, c(a = -1, b = 0),
       squared, c(-sqrt(squared_ols[1]), squared_ols[2])),
  case("exponential_slope", "exp(a*x)", xs, ys, c(a = .5), exp_slope, slope),
  case("constant_exponential", "exp(a)", rep(0, 5), c(1.2, 1.4, 1.1, 1.8, 1.5),
       c(a = 0), exp_constant, log(mean(c(1.2, 1.4, 1.1, 1.8, 1.5))), predictors = ""),
  case("rational_parameter", "a/(b+x)", xr, yr, c(a = 3, b = 1),
       rational, c(profile_amplitude(br), br)),
  # Exact score/step/rank success cannot replace the observed-curvature test.
  case("stationary_nonminimum", "a*a+a*x", c(-1, 1), c(1, 1), c(a = 0),
       stationary_nonminimum, status = "reject_nonminimum"),
  case("zero_variance", "a*x", c(-1, 1), c(-1, 1), c(a = 1),
       linear, status = "reject_zero_variance"),
  case("rank_deficient", "a+b*x", rep(1, 3), c(1, 2, 3), c(a = 1, b = 1),
       linear, status = "reject_rank"),
  case("infimum", "exp(a)", c(0, 0), c(0, 0), c(a = 0), exp_constant,
       status = "reject_no_finite_minimum", predictors = "")
)
if (length(args) == 2) {
  data <- read.table(args[2], col.names = c("y", "x"))
  stopifnot(nrow(data) == 10000, ncol(data) == 2,
            all(is.finite(as.matrix(data))), all(data$y > 0))
  profile_score <- function(d) {
    w <- exp(d * data$x)
    sum(data$x * data$y * w) / sum(data$y * w) - sum(data$x * w^2) / sum(w^2)
  }
  d <- uniroot(profile_score, c(0, 2), tol = 1e-14)$root
  w <- exp(d * data$x)
  a <- -log(sum(data$y * w) / sum(w^2))
  stopifnot(abs(profile_score(d)) < 1e-13)
  cases <- append(cases, list(case("original", "exp(-a-b*x)", data$x, data$y,
                                   c(a = 1, b = 0), source_exponential, c(a, -d))))
}

metadata <- inputs <- parameters <- covariance <- summaries <- projections <- derivatives <- list()
for (spec in cases) {
  n <- length(spec$y); p <- length(spec$start); param_names <- names(spec$start)
  theta <- as.vector(spec$solution); evaluation <- spec$evaluate(theta, spec$x)
  j <- evaluation$j; r <- spec$y - evaluation$mean
  rss <- sum(r^2); tss <- sum((spec$y - mean(spec$y))^2)
  observed <- crossprod(j)
  for (a in seq_len(p)) for (b in seq_len(p)) {
    observed[a, b] <- observed[a, b] - sum(r * evaluation$h[, a, b])
  }
  curvature <- min(eigen(observed, symmetric = TRUE, only.values = TRUE)$values)
  score <- if (rss == 0) NA_real_ else
    max(abs(as.vector(crossprod(j, r))) / (sqrt(rss) * sqrt(colSums(j^2))))
  qrj <- qr(j, tol = 1e-12)
  full_step <- if (qrj$rank == p) {
    max(abs(qr.coef(qrj, r)) / (1 + abs(theta)))
  } else NA_real_
  metadata[[length(metadata) + 1]] <- data.frame(
    case = spec$name, expression = spec$expression, predictors = spec$predictors,
    parameters = paste(param_names, collapse = ";"),
    start = paste(sprintf("%.17g", spec$start), collapse = ";"), status = spec$status)
  inputs[[length(inputs) + 1]] <- data.frame(case = spec$name, row = seq_len(n),
                                            x = spec$x, y = spec$y)
  # Freeze value/gradient/Hessian independently at starts AND solutions. Original
  # source derivatives are certified by scores/curvature, not a huge frozen tape.
  if (spec$name != "original") for (point in c("start", "solution")) {
    at <- if (point == "start") spec$start else theta
    ev <- spec$evaluate(at, spec$x)
    record <- function(quantity, parameter, column_parameter, value) {
      derivatives[[length(derivatives) + 1]] <<- data.frame(
        case = spec$name, point = point, row = seq_len(n), quantity = quantity,
        parameter = parameter, column_parameter = column_parameter, value = value)
    }
    record("mean", "", "", ev$mean)
    for (a in seq_len(p)) {
      record("gradient", param_names[a], "", ev$j[, a])
      for (b in seq_len(p)) record("hessian", param_names[a], param_names[b], ev$h[, a, b])
    }
  }
  if (spec$status == "fit") {
    stopifnot(n > p, rss > 0, qrj$rank == p, curvature > 0,
              score < 1e-12, full_step < 1e-12)
    projections[[length(projections) + 1]] <- data.frame(
      case = spec$name, row = seq_len(n), mean = evaluation$mean, residual = r)
    bread <- chol2inv(qr.R(qrj))
    for (mode in c("classical", "hc1")) {
      v <- if (mode == "classical") rss / (n - p) * bread else
        n / (n - p) * bread %*% crossprod(j * r) %*% bread
      se <- sqrt(diag(v)); statistic <- theta / se; critical <- qt(.975, n - p)
      parameters[[length(parameters) + 1]] <- data.frame(
        case = spec$name, mode = mode, parameter = param_names, estimate = theta,
        standard_error = se, t = statistic, p_value = 2 * pt(-abs(statistic), n - p),
        ci_lower = theta - critical * se, ci_upper = theta + critical * se)
      for (a in seq_len(p)) for (b in seq_len(p)) {
        covariance[[length(covariance) + 1]] <- data.frame(
          case = spec$name, mode = mode, row_parameter = param_names[a],
          column_parameter = param_names[b], value = v[a, b])
      }
      summaries[[length(summaries) + 1]] <- data.frame(
        case = spec$name, mode = mode, n = n, p = p, df = n - p, rss = rss,
        rmse = sqrt(rss / (n - p)), r_squared = if (tss == 0) NA_real_ else 1 - rss / tss,
        min_observed_hessian = curvature, normalized_score = score,
        relative_full_step = full_step)
    }
  } else {
    if (spec$status == "reject_nonminimum") stopifnot(curvature == -2, score == 0,
                                                      full_step <= 4 * .Machine$double.eps, qrj$rank == 1)
    if (spec$status == "reject_zero_variance") stopifnot(rss == 0)
    if (spec$status == "reject_rank") stopifnot(qrj$rank < p)
    if (spec$status == "reject_no_finite_minimum") {
      stopifnot(abs(score - 1) <= 4 * .Machine$double.eps,
                abs(full_step - 1) <= 4 * .Machine$double.eps)
    }
    summaries[[length(summaries) + 1]] <- data.frame(
      case = spec$name, mode = "initial_reject", n = n, p = p, df = n - p, rss = rss,
      rmse = sqrt(rss / (n - p)), r_squared = if (tss == 0) NA_real_ else 1 - rss / tss,
      min_observed_hessian = curvature, normalized_score = score,
      relative_full_step = full_step)
  }
}
for (name in c("metadata", "inputs", "parameters", "covariance", "summaries",
               "projections", "derivatives")) {
  write_exact(do.call(rbind, get(name)), paste0(name, ".csv"))
}
writeLines(c(R.version.string, paste("stats", packageVersion("stats")),
             "Independent analytic/OLS/profiled-score references; no nls convergence claim.",
             "GN covariance is under correct conditional mean, not observed-loss bread."),
           file.path(out, "versions.txt"))
cat("Certified", sum(vapply(cases, function(x) x$status == "fit", logical(1))),
    "fitted cases;", sum(vapply(cases, function(x) x$status != "fit", logical(1))),
    "explicit rejected controls. Wrote", normalizePath(out), "\n")
