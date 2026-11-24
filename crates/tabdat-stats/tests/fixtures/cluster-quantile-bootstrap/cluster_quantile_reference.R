# Independent Base R quantreg reference for whole-cluster pairs resampling.
# Rscript cluster_quantile_reference.R OUTPUT_DIRECTORY > reference.csv

args <- commandArgs(trailingOnly = TRUE)
if (length(args) != 1L) stop("supply one output directory")
file_arg <- sub("^--file=", "", grep("^--file=", commandArgs(), value = TRUE)[1])
script_dir <- dirname(normalizePath(file_arg))
if (!requireNamespace("quantreg", quietly = TRUE)) stop("install quantreg 6.1")

set.seed(906L)
cluster_count <- 20L
cluster_sizes <- rep(c(4L, 5L, 6L, 5L), 5L)
group <- rep(seq_len(cluster_count), cluster_sizes)
n <- length(group)
x <- rnorm(n)
z <- rnorm(n)
cluster_intercepts <- rnorm(cluster_count, 0, 1.2)
y <- 1.25 + 0.65 * x - 0.35 * z + cluster_intercepts[group] + rexp(n) - 1
data <- data.frame(row = seq_len(n), y = y, x = x, z = z, group = group)
data <- rbind(
  data,
  data.frame(row = 101, y = NA_real_, x = 0, z = 0, group = 21),
  data.frame(row = 102, y = 1, x = NA_real_, z = 0, group = 1),
  data.frame(row = 103, y = 1, x = 0, z = 0, group = NA_real_)
)
dir.create(args[[1]], recursive = TRUE, showWarnings = FALSE)
write.csv(data, file.path(args[[1]], "cluster_quantile_inputs.csv"),
          row.names = FALSE, quote = FALSE, na = "")

complete <- data[complete.cases(data[c("y", "x", "z", "group")]), ]
stopifnot(nrow(complete) == 100L, length(unique(complete$group)) == 20L)
draws <- read.csv(file.path(script_dir, "cluster_draws.csv"), check.names = FALSE)
draw_columns <- grep("^draw_", names(draws), value = TRUE)
stopifnot(nrow(draws) == 80L, length(draw_columns) == cluster_count)
quantiles <- c(0.25, 0.5, 0.75)
parameter_names <- c("intercept", "x", "z")

fit <- function(sample, tau) {
  values <- stats::coef(quantreg::rq(y ~ x + z, tau = tau, data = sample, method = "br"))
  names(values)[names(values) == "(Intercept)"] <- "intercept"
  values[parameter_names]
}

original <- unlist(lapply(quantiles, function(tau) fit(complete, tau)), use.names = FALSE)
estimates <- matrix(NA_real_, nrow = nrow(draws), ncol = length(original))
replicate_observation_counts <- integer(nrow(draws))
for (replicate in seq_len(nrow(draws))) {
  sampled_groups <- as.integer(draws[replicate, draw_columns])
  rows <- unlist(lapply(sampled_groups, function(id) which(complete$group == id)),
                 use.names = FALSE)
  resample <- complete[rows, , drop = FALSE]
  replicate_observation_counts[replicate] <- nrow(resample)
  estimates[replicate, ] <- unlist(
    lapply(quantiles, function(tau) fit(resample, tau)), use.names = FALSE
  )
}
covariance <- stats::cov(estimates)
bootstrap_mean <- colMeans(estimates)
coefficient_rows <- lapply(seq_along(original), function(j) {
  sorted <- sort(estimates[, j])
  interval <- stats::quantile(sorted, c(0.025, 0.975), type = 7, names = FALSE)
  parameter_names_j <- paste0("q", rep(quantiles, each = 3L), ":",
    rep(parameter_names, times = length(quantiles)))
  data.frame(
    record = "coefficient", parameter = parameter_names_j[j],
    field = c("estimate", "bootstrap_mean", "bias", "standard_error", "ci_lower", "ci_upper"),
    value = c(original[j], bootstrap_mean[j], bootstrap_mean[j] - original[j],
      sqrt(covariance[j, j]), interval)
  )
})
covariance_rows <- do.call(rbind, lapply(seq_len(ncol(covariance)), function(i) {
  do.call(rbind, lapply(seq_len(ncol(covariance)), function(j) {
    data.frame(
      record = "covariance",
      parameter = paste0("q", quantiles[(i - 1L) %/% 3L + 1L], ":",
        parameter_names[(i - 1L) %% 3L + 1L]),
      field = paste0("q", quantiles[(j - 1L) %/% 3L + 1L], ":",
        parameter_names[(j - 1L) %% 3L + 1L]),
      value = covariance[i, j]
    )
  }))
}))
metadata <- data.frame(
  record = "meta",
  parameter = c("observation_count", "dropped_observations", "cluster_count",
    "replications", "min_replicate_n", "max_replicate_n"),
  field = "value",
  value = c(nrow(complete), nrow(data) - nrow(complete), cluster_count,
    nrow(draws), min(replicate_observation_counts), max(replicate_observation_counts))
)
contrast_estimate <- original[8L] - original[2L]
contrast_draws <- estimates[, 8L] - estimates[, 2L]
contrast_se <- stats::sd(contrast_draws)
contrast_z <- contrast_estimate / contrast_se
contrast_rows <- data.frame(
  record = "contrast",
  parameter = "x",
  field = c("estimate", "standard_error", "statistic", "p_value", "ci_lower", "ci_upper"),
  value = c(contrast_estimate, contrast_se, contrast_z,
    2 * stats::pnorm(-abs(contrast_z)),
    contrast_estimate - stats::qnorm(.975) * contrast_se,
    contrast_estimate + stats::qnorm(.975) * contrast_se)
)
reference <- rbind(do.call(rbind, coefficient_rows), covariance_rows, metadata, contrast_rows)
write.csv(reference, stdout(), row.names = FALSE, quote = FALSE)
