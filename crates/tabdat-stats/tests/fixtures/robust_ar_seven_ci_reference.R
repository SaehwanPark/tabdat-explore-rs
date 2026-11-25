# Independent Base R reference for one-endogenous, seven-instrument robust AR sets.
# Rscript robust_ar_seven_ci_reference.R OUTPUT_DIRECTORY > reference.csv
options(digits = 17)
args <- commandArgs(trailingOnly = TRUE)
stopifnot(length(args) == 1L)
dir.create(args[[1]], recursive = TRUE, showWarnings = FALSE)

make_data <- function(seed, n = 168L) {
  set.seed(seed)
  w <- rnorm(n)
  z <- matrix(rnorm(n * 7L), n, 7L)
  groups <- rep(seq_len(n / 7L), each = 7L)
  group_shock <- rep(rnorm(n / 7L, sd = 0.3), each = 7L)
  weights <- c(0.3, -0.35, 0.4, -0.45, 0.5, -0.55, 0.6)
  x <- drop(z %*% weights) + 0.5 * w + group_shock + rnorm(n)
  y <- 1.2 * x + 0.4 * w + group_shock + rnorm(n) * (0.4 + abs(w))
  data.frame(row = seq_len(n), y, w, z1 = z[, 1], z2 = z[, 2], z3 = z[, 3],
             z4 = z[, 4], z5 = z[, 5], z6 = z[, 6], z7 = z[, 7], x, group = groups)
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
permutations <- function(values) {
  if (length(values) == 1L) return(list(values))
  unlist(lapply(seq_along(values), function(i)
    lapply(permutations(values[-i]), function(rest) c(values[i], rest))),
    recursive = FALSE)
}
polynomial_determinant <- function(matrix) {
  q <- nrow(matrix)
  out <- 0
  for (permutation in permutations(seq_len(q))) {
    inversions <- sum(outer(seq_len(q), seq_len(q), function(i, j)
      i < j & permutation[i] > permutation[j]))
    term <- 1
    for (i in seq_len(q)) term <- pmul(term, matrix[[i, permutation[i]]])
    out <- padd(out, term, if (inversions %% 2L) -1 else 1)
  }
  out
}

fit_reference <- function(data, fixture, covariance) {
  n <- nrow(data)
  W <- cbind(intercept = 1, data$w)
  q <- 7L
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
  critical <- qchisq(0.95, df = q)
  boundary <- padd(numerator, determinant, -critical * correction)
  while (length(boundary) > 1L && tail(boundary, 1L) == 0) boundary <- head(boundary, -1L)
  boundary_degree <- length(boundary) - 1L
  stopifnot(boundary_degree == 14L)
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

multiple <- make_data(4L)
secondary <- make_data(7L)
for (entry in list(list(name = "multiple", data = multiple),
                   list(name = "secondary", data = secondary))) {
  write.csv(entry$data, file.path(args[[1]], paste0("robust_ar_seven_ci_", entry$name, "_data.csv")),
            row.names = FALSE, quote = FALSE)
}
reference <- rbind(
  fit_reference(multiple, "multiple", "hc1"),
  fit_reference(multiple, "multiple", "cluster"),
  fit_reference(secondary, "secondary", "hc1"),
  fit_reference(secondary, "secondary", "cluster")
)
write.csv(reference, stdout(), row.names = FALSE, quote = FALSE)
