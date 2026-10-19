# Independent development-only Poisson/NB2 count probabilities.
# Scalar fixture: Rscript count_probability_reference.R > count_probability_reference.csv
# Empirical source: Rscript count_probability_reference.R mma20p1count.asc \
#   R_predictions.csv > R_reference.csv
# Probabilities come directly from dpois/ppois/dnbinom/pnbinom, never a recurrence.
# Empirical mode follows the author-program law; the printed NB2 row remains unresolved.
options(digits = 17)
args <- commandArgs(trailingOnly = TRUE)

scalar_cases <- function() {
  alpha_low <- exp(-11.9)
  alpha_high <- exp(11.9)
  alpha_extreme <- exp(11.99)
  data.frame(
    case = c(
      "poisson_tiny_1e300_cut1", "poisson_tiny_1e300_cut10",
      "poisson_tiny_1e300_cut100", "poisson_tiny_1e8_cut1",
      "poisson_tiny_1e8_cut10", "poisson_tiny_1e8_cut100",
      "poisson_mid_0_306_cut10", "poisson_mid_1_cut10",
      "poisson_mid_3_cut10", "poisson_mid_23_5_cut10",
      "poisson_large_750_cut100", "poisson_large_1000_cut100",
      "poisson_large_1e6_cut100",
      "nb2_tiny_1e300_alpha_0_69_cut1", "nb2_tiny_1e300_alpha_0_69_cut10",
      "nb2_tiny_1e8_alpha_0_01_cut10", "nb2_mid_0_306_alpha_0_01_cut10",
      "nb2_mid_3_alpha_0_69_cut10", "nb2_mid_23_5_alpha_1_182251683_cut10",
      "nb2_mid_3_alpha_3_cut100", "nb2_mid_0_306_alpha_low_cut10",
      "nb2_mid_3_alpha_low_cut10", "nb2_large_750_alpha_low_cut100",
      "nb2_large_1000_alpha_low_cut100", "nb2_large_1e6_alpha_low_cut100",
      "nb2_mid_0_306_alpha_high_cut10", "nb2_mid_3_alpha_high_cut10",
      "nb2_large_750_alpha_high_cut100", "nb2_large_1000_alpha_high_cut100",
      "nb2_large_1e6_alpha_high_cut100", "nb2_large_750_alpha_0_01_cut100",
      "nb2_large_1000_alpha_0_69_cut100", "nb2_large_1e6_alpha_3_cut100",
      "nb2_large_1e6_alpha_0_0125_cut100", "nb2_small_0_001_alpha_extreme_cut100",
      "nb2_large_750_alpha_extreme_cut100", "nb2_small_0_001_alpha_high_cut100"
    ),
    family = c(rep("poisson", 13L), rep("nb2", 24L)),
    mu = c(
      1e-300, 1e-300, 1e-300, 1e-8, 1e-8, 1e-8,
      0.306, 1, 3, 23.5, 750, 1000, 1e6,
      1e-300, 1e-300, 1e-8, 0.306, 3, 23.5, 3, 0.306, 3,
      750, 1000, 1e6, 0.306, 3, 750, 1000, 1e6, 750, 1000, 1e6, 1e6,
      1e-3, 750, 1e-3
    ),
    alpha = c(
      rep(NA_real_, 13L), 0.69, 0.69, 0.01, 0.01, 0.69,
      1.1822516830333165, 3, alpha_low, alpha_low, alpha_low, alpha_low,
      alpha_low, alpha_high, alpha_high, alpha_high, alpha_high, alpha_high,
      0.01, 0.69, 3, 0.0125, alpha_extreme, alpha_extreme, alpha_high
    ),
    tail_from = c(
      1, 10, 100, 1, 10, 100, 10, 10, 10, 10, 100, 100, 100,
      1, 10, 10, 10, 10, 10, 100, 10, 10, 100, 100, 100,
      10, 10, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100
    ),
    stringsAsFactors = FALSE
  )
}

probability_bins <- function(family, mu, alpha, tail_from) {
  exact_bins <- 0:(tail_from - 1L)
  if (family == "poisson") {
    exact <- dpois(exact_bins, lambda = mu)
    tail <- ppois(tail_from - 1L, lambda = mu, lower.tail = FALSE)
  } else {
    size <- 1 / alpha
    exact <- dnbinom(exact_bins, size = size, mu = mu)
    tail <- pnbinom(tail_from - 1L, size = size, mu = mu,
      lower.tail = FALSE)
  }
  c(exact, tail)
}

validate_probabilities <- function(probabilities, context) {
  if (!all(is.finite(probabilities)) || any(probabilities < 0) ||
      any(probabilities > 1) || abs(sum(probabilities) - 1) > 5e-13) {
    stop(sprintf("invalid partition for %s: sum=%.17g", context,
      sum(probabilities)), call. = FALSE)
  }
  invisible(context)
}

write_scalar_fixture <- function() {
  cases <- scalar_cases()
  stopifnot(nrow(cases) == 37L, !anyDuplicated(cases$case),
    all(is.finite(cases$mu)), all(cases$mu > 0),
    all(cases$tail_from %in% c(1, 10, 100)),
    all(cases$family == "poisson" | (is.finite(cases$alpha) & cases$alpha > 0)),
    all(abs(log(cases$alpha[cases$family == "nb2"])) < 12))
  partition_errors <- numeric(nrow(cases))
  rows <- lapply(seq_len(nrow(cases)), function(index) {
    current <- cases[index, ]
    probabilities <- probability_bins(current$family, current$mu,
      current$alpha, current$tail_from)
    validate_probabilities(probabilities, current$case)
    partition_errors[index] <<- abs(sum(probabilities) - 1)
    data.frame(case = current$case, family = current$family, mu = current$mu,
      alpha = current$alpha, tail_from = current$tail_from,
      bin = c(0:(current$tail_from - 1L), current$tail_from),
      probability = probabilities, stringsAsFactors = FALSE)
  })
  result <- do.call(rbind, rows)
  high_mean_nb2 <- probability_bins("nb2", 1e6, 0.0125, 100L)
  stopifnot(nrow(result) == 2090L, high_mean_nb2[1L] == 0,
    high_mean_nb2[100L] > 0,
    probability_bins("poisson", 1e-300, NA_real_, 1L)[2L] > 0,
    probability_bins("poisson", 1e-8, NA_real_, 10L)[11L] > 0,
    probability_bins("nb2", 1e-300, 0.69, 1L)[2L] > 0,
    probability_bins("nb2", 1e-8, 0.01, 10L)[11L] > 0)
  write.csv(result, stdout(), row.names = FALSE, quote = FALSE, na = "NA")
  worst_partition <- which.max(partition_errors)
  cat("Scalar cases: 37; records: 2090; cutoffs: 1, 10, 100; maximum ",
    "partition residual=", format(partition_errors[worst_partition], digits = 17),
    " for ", cases$case[worst_partition], ".\n", sep = "", file = stderr())
}

read_author_ascii <- function(path) {
  expected_sha256 <- "c8b1eee8ae1f637f4d4e4c6aa6b2dcbf2b8aa98f276b9369adcb4f02a528c58a"
  stopifnot(file.exists(path))
  checksum <- system2("shasum", c("-a", "256", shQuote(path)),
    stdout = TRUE, stderr = TRUE)
  stopifnot(length(checksum) >= 1L,
    identical(strsplit(checksum[1L], "[[:space:]]+")[[1L]][1L], expected_sha256))
  values <- scan(path, quiet = TRUE)
  stopifnot(length(values) == 20186L * 18L, all(is.finite(values)))
  data <- as.data.frame(matrix(values, ncol = 18L, byrow = TRUE))
  names(data) <- c("MDU", "LC", "IDP", "LPI", "FMDE", "PHYSLIM",
    "NDISEASE", "HLTHG", "HLTHF", "HLTHP", "LINC", "LFAM", "EDUCDEC",
    "AGE", "FEMALE", "CHILD", "FEMCHILD", "BLACK")
  stopifnot(nrow(data) == 20186L, sum(data$MDU) == 57746,
    sum(data$MDU == 0) == 6308, max(data$MDU) == 77,
    all(data$MDU >= 0 & data$MDU == floor(data$MDU)))
  cat("Source rows: 20186; columns: 18; SHA-256: ", expected_sha256,
    "; source checksum and count invariants passed.\n", sep = "", file = stderr())
  data
}

poisson_fit <- function(X, y) {
  initial <- glm.fit(X, y, family = poisson(), intercept = TRUE,
    control = glm.control(epsilon = 1e-13, maxit = 100L))
  stopifnot(isTRUE(initial$converged), all(is.finite(initial$coefficients)))
  beta <- unname(initial$coefficients)
  converged <- FALSE
  for (iteration in seq_len(100L)) {
    mu <- exp(drop(X %*% beta))
    information <- crossprod(X, X * mu)
    step <- drop(solve(information, crossprod(X, y - mu)))
    step_norm <- max(abs(step))
    if (step_norm < 1e-14) {
      converged <- TRUE
      break
    }
    beta <- beta + step
  }
  stopifnot(converged)
  eta <- drop(X %*% beta)
  mu <- exp(eta)
  score_norm <- max(abs(crossprod(X, y - mu)))
  stopifnot(all(is.finite(mu)), all(mu > 0), is.finite(score_norm))
  list(beta = beta, eta = eta, mu = mu, iterations = iteration,
    step_norm = step_norm, score_norm = score_norm,
    log_likelihood = sum(dpois(y, lambda = mu, log = TRUE)))
}

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
  list(log_likelihood = sum(dnbinom(y, size = r, mu = mu, log = TRUE)),
    score = colSums(scores), information = information, mu = mu, eta = eta)
}

nb2_fit <- function(X, y, predictors) {
  frame <- data.frame(MDU = y, predictors)
  initial <- MASS::glm.nb(MDU ~ ., data = frame,
    control = glm.control(epsilon = 1e-13, maxit = 100L))
  stopifnot(isTRUE(initial$converged))
  par <- c(unname(coef(initial)), lnalpha = -log(initial$theta))
  converged <- FALSE
  for (iteration in seq_len(50L)) {
    evaluated <- nb2_evaluate(par, X, y)
    step <- drop(solve(evaluated$information, evaluated$score))
    step_norm <- max(abs(step))
    if (step_norm < 1e-14) {
      converged <- TRUE
      break
    }
    par <- unname(par + step)
  }
  stopifnot(converged, abs(par[length(par)]) < 12)
  evaluated <- nb2_evaluate(par, X, y)
  alpha <- exp(par[length(par)])
  score_norm <- max(abs(evaluated$score))
  stopifnot(all(is.finite(evaluated$mu)), all(evaluated$mu > 0),
    is.finite(alpha), alpha > 0, is.finite(score_norm))
  list(beta = par[-length(par)], lnalpha = par[length(par)], alpha = alpha,
    eta = evaluated$eta, mu = evaluated$mu, iterations = iteration,
    step_norm = step_norm, score_norm = score_norm,
    log_likelihood = evaluated$log_likelihood)
}

fit_probabilities <- function(family, mu, alpha = NA_real_, tail_from = 10L) {
  exact <- vapply(0:(tail_from - 1L), function(count) {
    if (family == "poisson") dpois(count, lambda = mu) else
      dnbinom(count, size = 1 / alpha, mu = mu)
  }, numeric(length(mu)))
  tail <- if (family == "poisson") {
    ppois(tail_from - 1L, lambda = mu, lower.tail = FALSE)
  } else {
    pnbinom(tail_from - 1L, size = 1 / alpha, mu = mu,
      lower.tail = FALSE)
  }
  result <- cbind(exact, tail)
  colnames(result) <- c(paste0("p", 0:(tail_from - 1L)), paste0("ge", tail_from))
  partition_error <- max(abs(rowSums(result) - 1))
  stopifnot(all(is.finite(result)), all(result >= 0), all(result <= 1),
    partition_error <= 5e-14)
  attr(result, "partition_error") <- partition_error
  result
}

emit_aggregate <- function(rows, case, quantity, i, j, value) {
  rows[[length(rows) + 1L]] <- data.frame(case = case, quantity = quantity,
    i = i, j = j, value = unname(value), stringsAsFactors = FALSE)
  rows
}

empirical_reference <- function(source_path, predictions_path) {
  stopifnot(requireNamespace("MASS", quietly = TRUE))
  data <- read_author_ascii(source_path)
  y <- data$MDU
  predictors <- data[names(data) != "MDU"]
  X <- cbind(intercept = 1, as.matrix(predictors))
  stopifnot(nrow(X) == length(y), ncol(X) == 18L,
    all(is.finite(X)), qr(X)$rank == ncol(X))

  poisson <- poisson_fit(X, y)
  nb2 <- nb2_fit(X, y, predictors)
  poisson_probabilities <- fit_probabilities("poisson", poisson$mu)
  nb2_probabilities <- fit_probabilities("nb2", nb2$mu, nb2$alpha)
  empirical_partition_error <- max(attr(poisson_probabilities, "partition_error"),
    attr(nb2_probabilities, "partition_error"))
  predictions <- data.frame(prepared_row = seq_along(y), MDU = y, predictors,
    poisson_mu = poisson$mu, poisson_xb = poisson$eta,
    poisson_probabilities, nb2_mu = nb2$mu, nb2_xb = nb2$eta,
    nb2_probabilities, check.names = FALSE)
  names(predictions) <- c("prepared_row", "MDU", names(predictors),
    "poisson_mu", "poisson_xb", paste0("poisson_p", 0:9), "poisson_ge10",
    "nb2_mu", "nb2_xb", paste0("nb2_p", 0:9), "nb2_ge10")
  stopifnot(ncol(predictions) == 45L, nrow(predictions) == 20186L)
  write.csv(predictions, predictions_path, row.names = FALSE, quote = FALSE)

  aggregate <- list()
  append_case <- function(case, probabilities, log_likelihood, alpha = NA_real_) {
    observed_bin <- pmin(y, 10L)
    for (bin in 0:10) {
      observed_count <- sum(observed_bin == bin)
      fitted_fraction <- mean(probabilities[, bin + 1L])
      for (entry in list(
        c(quantity = "observed_count", value = observed_count),
        c(quantity = "observed_fraction", value = observed_count / length(y)),
        c(quantity = "fitted_fraction", value = fitted_fraction),
        c(quantity = "expected_count", value = length(y) * fitted_fraction)
      )) {
        aggregate <<- emit_aggregate(aggregate, case, entry[["quantity"]],
          bin, 0, as.numeric(entry[["value"]]))
      }
    }
    aggregate <<- emit_aggregate(aggregate, case, "N", 0, 0, length(y))
    aggregate <<- emit_aggregate(aggregate, case, "tail_from", 0, 0, 10)
    aggregate <<- emit_aggregate(aggregate, case, "log_likelihood", 0, 0,
      log_likelihood)
    if (case == "nb2") {
      aggregate <<- emit_aggregate(aggregate, case, "alpha", 0, 0, alpha)
    }
  }
  append_case("poisson", poisson_probabilities, poisson$log_likelihood)
  append_case("nb2", nb2_probabilities, nb2$log_likelihood, nb2$alpha)
  aggregate <- do.call(rbind, aggregate)
  stopifnot(nrow(aggregate) == 95L,
    identical(names(aggregate), c("case", "quantity", "i", "j", "value")))
  poisson_observed <- aggregate$value[aggregate$case == "poisson" &
    aggregate$quantity == "observed_count"]
  nb2_observed <- aggregate$value[aggregate$case == "nb2" &
    aggregate$quantity == "observed_count"]
  poisson_fitted <- aggregate$value[aggregate$case == "poisson" &
    aggregate$quantity == "fitted_fraction"]
  nb2_fitted <- aggregate$value[aggregate$case == "nb2" &
    aggregate$quantity == "fitted_fraction"]
  poisson_expected <- aggregate$value[aggregate$case == "poisson" &
    aggregate$quantity == "expected_count"]
  nb2_expected <- aggregate$value[aggregate$case == "nb2" &
    aggregate$quantity == "expected_count"]
  stopifnot(sum(poisson_observed) == length(y),
    identical(poisson_observed, nb2_observed),
    abs(sum(poisson_fitted) - 1) <= 5e-14,
    abs(sum(nb2_fitted) - 1) <= 5e-14,
    abs(sum(poisson_expected) - length(y)) <= length(y) * 5e-14,
    abs(sum(nb2_expected) - length(y)) <= length(y) * 5e-14)
  write.csv(aggregate, stdout(), row.names = FALSE, quote = FALSE)

  poisson_means <- colMeans(poisson_probabilities)
  nb2_means <- colMeans(nb2_probabilities)
  author_poisson <- c(0.1067641, 0.1922452, 0.2092483, 0.1763299,
    0.1258341, 0.0799562, 0.0469841, 0.0264166, 0.0146732, 0.0082771,
    0.0132712)
  author_nb2 <- c(0.318268, 0.1907193, 0.1275572, 0.0892949, 0.0643067,
    0.0472688, 0.0353048, 0.0267168, 0.0204443, 0.0157971, 0.064322)
  stopifnot(max(abs(poisson_means - author_poisson)) < 5e-6,
    max(abs(nb2_means - author_nb2)) < 5e-6)
  author_comparison <- rbind(
    data.frame(family = "poisson", bin = 0:10,
      author_2006_mean = author_poisson, reference_mean = poisson_means,
      difference = poisson_means - author_poisson),
    data.frame(family = "nb2", bin = 0:10, author_2006_mean = author_nb2,
      reference_mean = nb2_means, difference = nb2_means - author_nb2)
  )
  book_nb2_percent <- c(30.9, 19.6, 13.6, 9.67, 6.97, 5.07, 3.70, 2.72, 2.0, 1.47)
  printed_comparison <- data.frame(bin = 0:9,
    book_printed_percent = book_nb2_percent,
    reference_percent = 100 * nb2_means[1:10],
    difference_percentage_points = 100 * nb2_means[1:10] - book_nb2_percent)
  cat("R ", as.character(getRversion()), "; MASS ",
    as.character(packageVersion("MASS")), ".\n", sep = "", file = stderr())
  cat("Convergence: Poisson iterations=", poisson$iterations,
    ", max_abs_step=", format(poisson$step_norm, digits = 17),
    ", max_abs_score=", format(poisson$score_norm, digits = 17),
    "; NB2 iterations=", nb2$iterations, ", max_abs_step=",
    format(nb2$step_norm, digits = 17), ", max_abs_joint_score=",
    format(nb2$score_norm, digits = 17), ".\n", sep = "", file = stderr())
  cat("Converged independent Poisson and joint NB2 score polishing; rank=18; ",
    "N=20186; cutoff=10; maximum row partition error=",
    format(empirical_partition_error, digits = 17), ".\n",
    sep = "", file = stderr())
  cat("Aggregate schema: case,quantity,i,j,value; rows=95. Prediction schema: ",
    "prepared_row,MDU,17 predictors,poisson_mu,poisson_xb,poisson_p0..p9,",
    "poisson_ge10,nb2_mu,nb2_xb,nb2_p0..p9,nb2_ge10; rows=20186, columns=45.\n",
    sep = "", file = stderr())
  cat("Poisson and NB2 aggregates agree with the rounded 2006 author-program means ",
    "within 5e-6 probability units. Rounded author-program comparison ",
    "(difference=reference minus program mean):\n", file = stderr())
  write.table(author_comparison, file = stderr(), sep = ",", row.names = FALSE,
    quote = FALSE)
  cat("The printed NB2 row differs; its discrepancy remains unresolved ",
    "(no rounding explanation is asserted).\n", file = stderr())
  cat("NB2 printed Table 20.6 comparison (difference is reference minus printed, ",
    "percentage points):\n", file = stderr())
  write.table(printed_comparison, file = stderr(), sep = ",", row.names = FALSE,
    quote = FALSE)
}

if (length(args) == 0L) {
  write_scalar_fixture()
} else if (length(args) == 2L) {
  empirical_reference(args[1L], args[2L])
} else {
  stop("usage: count_probability_reference.R [author_ASC predictions_csv]")
}
