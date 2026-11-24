args <- commandArgs(trailingOnly = TRUE)
data_path <- if (length(args) >= 1) args[[1]] else "examples/nb2-overdispersion-score/score_reference.csv"
reference_path <- if (length(args) >= 2) args[[2]] else "crates/tabdat-stats/tests/fixtures/poisson_nb2_score_reference.csv"

set.seed(20261124)
n <- 180L
row <- seq_len(n)
x <- sin(row * 0.11) + cos(row * 0.037)
log_exposure <- -0.2 + 0.15 * cos(row * 0.19)
mu <- exp(0.15 + 0.24 * x + log_exposure)
alpha <- 0.35
independent_rate <- mu * rgamma(n, shape = 1 / alpha, scale = alpha)
y_independent <- rpois(n, independent_rate)
group <- rep(seq_len(30L), each = 6L)
group_rate <- rgamma(30L, shape = 1 / alpha, scale = alpha)
y_clustered <- rpois(n, mu * group_rate[group])
data <- data.frame(
  source_row = row,
  y_independent = y_independent,
  y_clustered = y_clustered,
  x = x,
  log_exposure = log_exposure,
  group = group
)
write.csv(data, data_path, row.names = FALSE, quote = FALSE)

reference <- function(y, model, covariance, groups = NULL) {
  fit <- glm(y ~ x + offset(log_exposure), family = poisson(), data = data)
  fitted_mean <- fitted(fit)
  score <- 0.5 * ((y - fitted_mean)^2 - y)
  if (covariance == "model") {
    correction <- 1
    score_variance <- 0.5 * sum(fitted_mean^2)
  } else if (covariance == "robust") {
    correction <- length(score) / (length(score) - 1)
    score_variance <- correction * sum(score^2)
  } else {
    group_score <- as.numeric(tapply(score, groups, sum))
    correction <- length(group_score) / (length(group_score) - 1)
    score_variance <- correction * sum(group_score^2)
  }
  total_score <- sum(score)
  statistic <- total_score / sqrt(score_variance)
  data.frame(
    model = model,
    covariance = covariance,
    observation_count = length(y),
    group_count = if (is.null(groups)) NA_integer_ else length(unique(groups)),
    correction = correction,
    score = total_score,
    score_variance = score_variance,
    statistic = statistic,
    p_value = pnorm(-statistic)
  )
}

results <- rbind(
  reference(data$y_independent, "poisson", "model"),
  reference(data$y_independent, "poisson", "robust"),
  reference(data$y_clustered, "poisson", "cluster", data$group)
)
write.csv(results, reference_path, row.names = FALSE, quote = FALSE)
