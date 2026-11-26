# Independent Base R reference for one-endogenous, twenty-three-instrument robust AR sets.
# Rscript robust_ar_twenty_three_ci_reference.R OUTPUT_DIRECTORY > reference.csv
options(digits = 17)
args <- commandArgs(trailingOnly = TRUE)
stopifnot(length(args) == 1L)
dir.create(args[[1]], recursive = TRUE, showWarnings = FALSE)

make_data <- function(seed, n = 552L) {
  set.seed(seed)
  w <- rnorm(n)
  z <- matrix(rnorm(n * 23L), n, 23L)
  colnames(z) <- paste0("z", seq_len(ncol(z)))
  groups <- rep(seq_len(n / 6L), each = 6L)
  group_shock <- rep(rnorm(n / 6L, sd = 0.3), each = 6L)
  weights <- c(0.3, -0.35, 0.4, -0.45, 0.5, -0.55, 0.6, -0.65, 0.7, -0.75, 0.8, -0.85, 0.9, -0.95, 1.0, -1.05, 1.1, -1.15, 1.2, -1.25, 1.3, -1.35, 0.1)
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
polynomial_determinant <- function(polynomial_matrix, degree, variable_scale) {
  # Chebyshev interpolation keeps degree-46 coefficient construction tractable;
  # the independent-node check compares the converted coefficients with direct
  # base R determinants before they enter the boundary calculation.
  count <- degree + 1L
  angles <- pi * (2 * seq_len(count) - 1) / (2 * count)
  nodes <- cos(angles)
  values <- vapply(nodes, function(point)
    det(evaluate_polynomial_matrix(polynomial_matrix, point * variable_scale)), numeric(1))
  chebyshev <- numeric(count)
  for (order in 0L:degree) {
    scale <- if (order == 0L) 1 / count else 2 / count
    chebyshev[order + 1L] <- scale * sum(values * cos(order * angles))
  }
  coefficients <- chebyshev[1L]
  previous <- 1
  current <- c(0, 1)
  coefficients <- padd(coefficients, current, chebyshev[2L])
  if (degree >= 2L) for (order in 2L:degree) {
    following <- padd(c(0, 2 * current), previous, -1)
    coefficients <- padd(coefficients, following, chebyshev[order + 1L])
    previous <- current
    current <- following
  }
  coefficients
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
                                         variable_scale, multiplier = 1) {
  # Compare coefficient expansion with base R's numeric LU determinant on
  # independent Chebyshev nodes to guard the q=23 determinant construction.
  count <- degree + 1L
  nodes <- cos(pi * seq_len(count) / (count + 1L))
  direct <- multiplier * vapply(nodes, function(point)
    det(evaluate_polynomial_matrix(polynomial_matrix, point * variable_scale)), numeric(1))
  evaluated <- vapply(nodes, function(point)
    evaluate_polynomial(polynomial, point), numeric(1))
  scale <- max(abs(direct))
  stopifnot(all(is.finite(direct)), all(is.finite(evaluated)), scale > 0,
            max(abs(evaluated - direct)) <= 5e-8 * scale)
}

fit_reference <- function(data, fixture, covariance) {
  n <- nrow(data)
  W <- cbind(intercept = 1, data$w)
  q <- 23L
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
  # Match the broad scan range while keeping the high-degree interpolation scaled.
  variable_scale <- 32
  determinant <- polynomial_determinant(S, 2L * q, variable_scale)
  check_determinant_polynomial(determinant, S, 2L * q, variable_scale)
  determinant_roots <- polyroot(determinant)
  stopifnot(all(is.finite(Re(determinant_roots))), all(is.finite(Im(determinant_roots))))
  if (covariance == "cluster") {
    near_real <- determinant_roots[abs(Im(determinant_roots)) < 1e-7]
    diagonal_ratios <- vapply(Re(near_real), function(point) {
      score <- evaluate_polynomial_matrix(S, point * variable_scale)
      det(score) / prod(diag(score))
    }, numeric(1))
    stopifnot(all(is.finite(diagonal_ratios)), all(diagonal_ratios > 1e-12))
  }

  block <- matrix(vector("list", (q + 1L)^2L), q + 1L, q + 1L)
  for (i in seq_len(q)) for (j in seq_len(q)) block[[i, j]] <- S[[i, j]]
  for (i in seq_len(q)) {
    block[[i, q + 1L]] <- c(center_g[i], -gx[i])
    block[[q + 1L, i]] <- c(center_g[i], -gx[i])
  }
  block[[q + 1L, q + 1L]] <- 0
  numerator <- -polynomial_determinant(block, 2L * q, variable_scale)
  check_determinant_polynomial(numerator, block, 2L * q, variable_scale, multiplier = -1)
  critical <- qchisq(0.95, df = q)
  boundary <- padd(numerator, determinant, -critical * correction)
  while (length(boundary) > 1L && tail(boundary, 1L) == 0) boundary <- head(boundary, -1L)
  boundary_degree <- length(boundary) - 1L
  stopifnot(boundary_degree == 46L)
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
  # The broad scan finds separated regions; the dense local scan resolves narrow
  # crossings near the center before uniroot refines each direct-Wald endpoint.
  grid <- sort(unique(c(
    seq(center - 32, center + 32, length.out = 2049L),
    seq(center - 1, center + 1, length.out = 4097L),
    center
  )))
  grid_statistic <- vapply(grid, direct_statistic, numeric(1))
  included <- grid_statistic <= critical
  crossings <- which(included[-length(included)] != included[-1L])
  roots <- vapply(crossings, function(index)
    uniroot(function(beta) direct_statistic(beta) - critical,
            c(grid[index], grid[index + 1L]), tol = 1e-12)$root, numeric(1))
  intervals <- list()
  lower <- if (included[1L]) -Inf else NA_real_
  for (i in seq_along(roots)) {
    if (!included[crossings[i]]) {
      lower <- roots[i]
    } else {
      intervals[[length(intervals) + 1L]] <- c(lower, roots[i])
      lower <- NA_real_
    }
  }
  if (included[length(included)])
    intervals[[length(intervals) + 1L]] <- c(lower, Inf)
  shape <- if (length(intervals) == 0L) "empty" else
    if (length(intervals) == 1L && all(is.infinite(intervals[[1L]]))) "all_real" else
    if (length(intervals) > 2L) "multiple" else
    if (length(intervals) == 2L) "disjoint" else
    if (is.infinite(intervals[[1L]][1L])) "lower_ray" else
    if (is.infinite(intervals[[1L]][2L])) "upper_ray" else "bounded"
  stopifnot(length(roots) <= 2L * q, length(roots) %% 2L == 0L)
  root_statistics <- vapply(roots, direct_statistic, numeric(1))
  root_error <- abs(root_statistics - critical)
  stopifnot(all(is.finite(root_statistics)),
            all(root_error < 5e-6 * max(1, critical)))

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

multiple <- make_data(29L)
secondary <- make_data(11L)
for (entry in list(list(name = "multiple", data = multiple),
                   list(name = "secondary", data = secondary))) {
  write.csv(entry$data, file.path(args[[1]], paste0("robust_ar_twenty_three_ci_", entry$name, "_data.csv")),
            row.names = FALSE, quote = FALSE)
}
reference <- rbind(
  fit_reference(multiple, "multiple", "hc1"),
  fit_reference(multiple, "multiple", "cluster"),
  fit_reference(secondary, "secondary", "hc1"),
  fit_reference(secondary, "secondary", "cluster")
)
write.csv(reference, stdout(), row.names = FALSE, quote = FALSE)
