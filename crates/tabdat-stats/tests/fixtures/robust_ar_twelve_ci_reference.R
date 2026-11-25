# Independent Base R reference for one-endogenous, twelve-instrument robust AR sets.
# Rscript robust_ar_twelve_ci_reference.R OUTPUT_DIRECTORY > reference.csv
options(digits = 17)
args <- commandArgs(trailingOnly = TRUE)
stopifnot(length(args) == 1L)
dir.create(args[[1]], recursive = TRUE, showWarnings = FALSE)

make_data <- function(seed, n = 288L) {
  set.seed(seed)
  w <- rnorm(n)
  z <- matrix(rnorm(n * 12L), n, 12L)
  colnames(z) <- paste0("z", seq_len(ncol(z)))
  groups <- rep(seq_len(n / 12L), each = 12L)
  group_shock <- rep(rnorm(n / 12L, sd = 0.3), each = 12L)
  weights <- c(0.3, -0.35, 0.4, -0.45, 0.5, -0.55, 0.6, -0.65, 0.7, -0.75, 0.8, -0.85)
  x <- drop(z %*% weights) + 0.5 * w + group_shock + rnorm(n)
  y <- 1.2 * x + 0.4 * w + group_shock + rnorm(n) * (0.4 + abs(w))
  data.frame(row = seq_len(n), y, w, z, x, group = groups)
}

padd <- function(a, b, scale = 1) {
  out <- numeric(max(length(a), length(b)))
  out[seq_along(a)] <- out[seq_along(a)] + a
  out[seq_along(b)] <- out[seq_along(b)] + scale * b
  out
}
pmul <- function(a, b) {
  out <- numeric(length(a) + length(b) - 1L)
  for (i in seq_along(a)) for (j in seq_along(b))
    out[i + j - 1L] <- out[i + j - 1L] + a[i] * b[j]
  out
}
polynomial_determinant <- function(matrix) {
  n <- nrow(matrix)
  state_count <- bitwShiftL(1L, n)
  partial <- vector("list", state_count)
  partial[[1L]] <- 1
  # Subset dynamic programming avoids enumerating all (q + 1)! terms.
  for (mask in 0L:(state_count - 1L)) {
    product <- partial[[mask + 1L]]
    if (is.null(product)) next
    columns <- which(as.logical(intToBits(mask)[seq_len(n)])) - 1L
    row <- length(columns)
    if (row >= n) next
    for (column in setdiff(seq_len(n) - 1L, columns)) {
      inversions <- sum(columns > column)
      next_mask <- bitwOr(mask, bitwShiftL(1L, column))
      term <- pmul(product, matrix[[row + 1L, column + 1L]])
      sign <- if (inversions %% 2L == 0L) 1 else -1
      index <- next_mask + 1L
      partial[[index]] <- if (is.null(partial[[index]])) {
        term * sign
      } else {
        padd(partial[[index]], term, sign)
      }
    }
  }
  partial[[state_count]]
}
evaluate_polynomial <- function(polynomial, point) {
  sum(polynomial * point^(seq_along(polynomial) - 1L))
}
evaluate_polynomial_matrix <- function(polynomial_matrix, point) {
  out <- base::matrix(0, nrow(polynomial_matrix), ncol(polynomial_matrix))
  for (i in seq_len(nrow(polynomial_matrix))) for (j in seq_len(ncol(polynomial_matrix)))
    out[i, j] <- evaluate_polynomial(polynomial_matrix[[i, j]], point)
  out
}
check_determinant_polynomial <- function(polynomial, polynomial_matrix, degree,
                                         multiplier = 1) {
  # Compare coefficient expansion with base R's numeric LU determinant on
  # independent Chebyshev nodes to guard the q=12 determinant construction.
  count <- degree + 1L
  nodes <- cos(pi * (2 * seq_len(count) - 1) / (2 * count))
  direct <- multiplier * vapply(nodes, function(point)
    det(evaluate_polynomial_matrix(polynomial_matrix, point)), numeric(1))
  evaluated <- vapply(nodes, function(point)
    evaluate_polynomial(polynomial, point), numeric(1))
  scale <- max(abs(direct))
  stopifnot(all(is.finite(direct)), all(is.finite(evaluated)), scale > 0,
            max(abs(evaluated - direct)) <= 5e-8 * scale)
}

fit_reference <- function(data, fixture, covariance) {
  n <- nrow(data)
  W <- cbind(intercept = 1, data$w)
  q <- 12L
  Z <- as.matrix(data[, paste0("z", seq_len(q))])
  X <- cbind(W, Z)
  x <- data$x
  y <- data$y
  groups <- data$group

  residualized_z <- qr.resid(qr(W), Z)
  Q <- qr.Q(qr(residualized_z))[, seq_len(q), drop = FALSE]
  y_w <- qr.resid(qr(W), y)
  x_w <- qr.resid(qr(W), x)
  y_auxiliary <- qr.resid(qr(X), y)
  x_auxiliary <- qr.resid(qr(X), x)
  gy <- drop(crossprod(Q, y_w))
  gx <- drop(crossprod(Q, x_w))
  score_y <- Q * y_auxiliary
  score_x <- Q * x_auxiliary
  if (covariance == "cluster") {
    score_y <- rowsum(score_y, groups)
    score_x <- rowsum(score_x, groups)
    cluster_count <- length(unique(groups))
    correction <- cluster_count / (cluster_count - 1) * (n - 1) / (n - ncol(X))
  } else {
    cluster_count <- NA_integer_
    correction <- n / (n - ncol(X))
  }

  center <- sum(gy * gx) / sum(gx^2)
  center_g <- gy - center * gx
  score_center <- score_y - center * score_x
  S <- matrix(vector("list", q * q), q, q)
  for (i in seq_len(q)) for (j in seq_len(q)) {
    S[[i, j]] <- c(sum(score_center[, i] * score_center[, j]),
                   -sum(score_center[, i] * score_x[, j] + score_x[, i] * score_center[, j]),
                   sum(score_x[, i] * score_x[, j]))
  }
  determinant <- polynomial_determinant(S)
  check_determinant_polynomial(determinant, S, 2L * q)
  determinant_roots <- polyroot(determinant)
  stopifnot(all(abs(Im(determinant_roots)) > 1e-7 * (1 + abs(Re(determinant_roots)))))

  block <- matrix(vector("list", (q + 1L)^2L), q + 1L, q + 1L)
  for (i in seq_len(q)) for (j in seq_len(q)) block[[i, j]] <- S[[i, j]]
  for (i in seq_len(q)) {
    block[[i, q + 1L]] <- c(center_g[i], -gx[i])
    block[[q + 1L, i]] <- c(center_g[i], -gx[i])
  }
  block[[q + 1L, q + 1L]] <- 0
  numerator <- -polynomial_determinant(block)
  check_determinant_polynomial(numerator, block, 2L * q, multiplier = -1)
  critical <- qchisq(0.95, df = q)
  boundary <- padd(numerator, determinant, -critical * correction)
  while (length(boundary) > 1L && tail(boundary, 1L) == 0) boundary <- head(boundary, -1L)
  boundary_degree <- length(boundary) - 1L
  stopifnot(boundary_degree == 24L)
  complex_roots <- polyroot(boundary)
  real_mask <- abs(Im(complex_roots)) < 1e-7 * (1 + abs(Re(complex_roots)))
  roots <- sort(Re(complex_roots[real_mask]) + center)
  stopifnot(all(is.finite(roots)), !anyDuplicated(round(roots, 8L)))

  direct_statistic <- function(beta) {
    auxiliary <- lm.fit(X, y - beta * x)
    scores <- X * auxiliary$residuals
    if (covariance == "cluster") scores <- rowsum(scores, groups)
    bread <- solve(crossprod(X))
    vcov <- correction * bread %*% crossprod(scores) %*% bread
    gamma <- auxiliary$coefficients[(ncol(W) + 1L):ncol(X)]
    gamma_vcov <- vcov[(ncol(W) + 1L):ncol(X),
                       (ncol(W) + 1L):ncol(X), drop = FALSE]
    as.numeric(t(gamma) %*% solve(gamma_vcov, gamma))
  }
  endpoints <- c(-Inf, roots, Inf)
  root_bound <- max(10, 10 * max(abs(roots), 0))
  probes <- numeric(length(endpoints) - 1L)
  for (i in seq_along(probes)) {
    probes[i] <- if (is.infinite(endpoints[i])) -root_bound else
      if (is.infinite(endpoints[i + 1L])) root_bound else
      endpoints[i] / 2 + endpoints[i + 1L] / 2
  }
  included <- vapply(probes, function(beta) direct_statistic(beta) <= critical, logical(1))
  intervals <- list()
  for (i in seq_along(included)) if (included[i])
    intervals[[length(intervals) + 1L]] <- c(endpoints[i], endpoints[i + 1L])
  shape <- if (length(intervals) == 0L) "empty" else
    if (length(intervals) == 1L && all(is.infinite(intervals[[1L]]))) "all_real" else
    if (length(intervals) > 2L) "multiple" else
    if (length(intervals) == 2L) "disjoint" else
    if (is.infinite(intervals[[1L]][1L])) "lower_ray" else
    if (is.infinite(intervals[[1L]][2L])) "upper_ray" else "bounded"
  stopifnot(length(roots) <= 2L * q, length(roots) %% 2L == 0L)
  for (root in roots) {
    value <- direct_statistic(root)
    stopifnot(is.finite(value), abs(value - critical) < 5e-6 * max(1, critical))
  }

  if (length(intervals) == 0L) {
    common <- data.frame(
      fixture = fixture, covariance = covariance, shape = shape,
      confidence_level = 95, observation_count = n,
      auxiliary_parameter_count = ncol(X), auxiliary_residual_df = n - ncol(X),
      covariance_correction = correction, cluster_group_count = cluster_count,
      degrees_of_freedom = q, critical_chi_square = critical,
      root_count = length(roots), boundary_degree = boundary_degree,
      record = "meta", interval_index = NA_integer_,
      lower = NA_real_, upper = NA_real_, beta = NA_real_, statistic = NA_real_
    )
  } else {
    common <- data.frame(
      fixture = fixture, covariance = covariance, shape = shape,
      confidence_level = 95, observation_count = n,
      auxiliary_parameter_count = ncol(X), auxiliary_residual_df = n - ncol(X),
      covariance_correction = correction, cluster_group_count = cluster_count,
      degrees_of_freedom = q, critical_chi_square = critical,
      root_count = length(roots), boundary_degree = boundary_degree, record = "interval",
      interval_index = seq_along(intervals),
      lower = vapply(intervals, `[[`, numeric(1), 1L),
      upper = vapply(intervals, `[[`, numeric(1), 2L),
      beta = NA_real_, statistic = NA_real_
    )
  }
  fixed_probes <- c(-10, -2, 0, 0.5, 2, 10)
  probe_rows <- data.frame(
    fixture = fixture, covariance = covariance, shape = shape,
    confidence_level = 95, observation_count = n,
    auxiliary_parameter_count = ncol(X), auxiliary_residual_df = n - ncol(X),
    covariance_correction = correction, cluster_group_count = cluster_count,
    degrees_of_freedom = q, critical_chi_square = critical,
    root_count = length(roots), boundary_degree = boundary_degree,
    record = "probe", interval_index = NA_integer_,
    lower = NA_real_, upper = NA_real_, beta = fixed_probes,
    statistic = vapply(fixed_probes, direct_statistic, numeric(1))
  )
  rbind(common, probe_rows)
}

multiple <- make_data(313L)
secondary <- make_data(419L)
for (entry in list(list(name = "multiple", data = multiple),
                   list(name = "secondary", data = secondary))) {
  write.csv(entry$data, file.path(args[[1]], paste0("robust_ar_twelve_ci_", entry$name, "_data.csv")),
            row.names = FALSE, quote = FALSE)
}
reference <- rbind(
  fit_reference(multiple, "multiple", "hc1"),
  fit_reference(multiple, "multiple", "cluster"),
  fit_reference(secondary, "secondary", "hc1"),
  fit_reference(secondary, "secondary", "cluster")
)
write.csv(reference, stdout(), row.names = FALSE, quote = FALSE)
