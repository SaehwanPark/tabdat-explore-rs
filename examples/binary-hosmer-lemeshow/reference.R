#!/usr/bin/env Rscript

script_arg <- grep("^--file=", commandArgs(trailingOnly = FALSE), value = TRUE)
script_dir <- dirname(normalizePath(sub("^--file=", "", script_arg[[1]])))
data_path <- file.path(script_dir, "binary_gof_data.csv")
reference_path <- file.path(script_dir, "binary_gof_reference.csv")
options(digits = 17)

# A four-pattern, 160-row unweighted Bernoulli design. Repeated rows create
# exact fitted-probability ties so quantile boundaries exercise tie handling.
patterns <- expand.grid(x1 = 0:1, x2 = 0:1)
events <- c(5L, 13L, 22L, 32L)
rows <- do.call(rbind, lapply(seq_len(nrow(patterns)), function(pattern) {
  repeat_index <- 0:39
  data.frame(
    id = (pattern - 1L) * 40L + repeat_index + 1L,
    y = as.integer(((repeat_index * 7L + pattern * 11L) %% 40L) < events[[pattern]]),
    x1 = patterns$x1[[pattern]],
    x2 = patterns$x2[[pattern]],
    cluster = (repeat_index + pattern) %% 10L + 1L
  )
}))
rows$x1[[1]] <- NA_real_
write.csv(rows, data_path, row.names = FALSE, quote = FALSE, na = "")

grouped_gof <- function(y, probability, requested_groups) {
  unique_probability <- sort(unique(probability))
  pattern_count <- tabulate(match(probability, unique_probability),
                            nbins = length(unique_probability))
  cumulative_count <- cumsum(pattern_count)
  targets <- vapply(seq_len(requested_groups - 1L), function(i) {
    as.integer(ceiling(i * length(y) / requested_groups))
  }, integer(1))
  candidate_boundaries <- unique(vapply(targets, function(target) {
    unique_probability[[which(cumulative_count >= target)[[1]]]]
  }, numeric(1)))
  boundaries <- candidate_boundaries[candidate_boundaries < max(unique_probability)]
  group_id <- 1L + vapply(probability, function(value) {
    sum(boundaries < value)
  }, integer(1))
  group_count <- max(group_id)
  groups <- do.call(rbind, lapply(seq_len(group_count), function(group) {
    selected <- group_id == group
    n <- sum(selected)
    expected_events <- sum(probability[selected])
    expected_nonevents <- sum(1 - probability[selected])
    data.frame(
      group = group,
      probability_upper_bound = max(probability[selected]),
      observation_count = n,
      observed_events = sum(y[selected]),
      expected_events = expected_events,
      observed_nonevents = n - sum(y[selected]),
      expected_nonevents = expected_nonevents
    )
  }))
  groups$pearson_component <-
    (groups$observed_events - groups$expected_events)^2 /
    (groups$expected_events * groups$expected_nonevents / groups$observation_count)
  statistic <- sum(groups$pearson_component)
  degrees_of_freedom <- group_count - 2L
  groups$model <- NA_character_
  groups$requested_group_count <- requested_groups
  groups$group_count <- group_count
  groups$degrees_of_freedom <- degrees_of_freedom
  groups$statistic <- statistic
  groups$p_value <- pchisq(statistic, degrees_of_freedom, lower.tail = FALSE)
  groups
}

reference <- do.call(rbind, lapply(c("logit", "probit"), function(link) {
  fit <- glm(y ~ x1 + x2, family = binomial(link = link), data = rows)
  fit_rows <- model.frame(fit)
  output <- grouped_gof(model.response(fit_rows), fitted(fit), requested_groups = 10L)
  output$model <- link
  output
}))
reference <- reference[c(
  "model", "requested_group_count", "group_count", "degrees_of_freedom",
  "statistic", "p_value", "group", "probability_upper_bound",
  "observation_count", "observed_events", "expected_events",
  "observed_nonevents", "expected_nonevents", "pearson_component"
)]
write.csv(reference, reference_path, row.names = FALSE, quote = FALSE)

cat(sprintf("R %s; wrote %d fixture rows and %d grouped reference rows\n",
            getRversion(), nrow(rows), nrow(reference)))
