args <- commandArgs(trailingOnly = TRUE)
if (length(args) != 2L) {
  stop("usage: reference_maxscore.R <Nldata.csv> <output.csv>", call. = FALSE)
}

raw <- read.csv(args[[1L]], check.names = FALSE)
stopifnot(nrow(raw) == 1182L,
  identical(raw$source_row, seq_len(1182L)),
  all(c("mode", "ppier", "pcharter") %in% names(raw)))
sample <- raw[raw$mode %in% c(2L, 4L), , drop = FALSE]
stopifnot(nrow(sample) == 630L,
  sum(sample$mode == 2L) == 178L,
  sum(sample$mode == 4L) == 452L,
  all(is.finite(sample$pcharter) & sample$pcharter > 0),
  all(is.finite(sample$ppier) & sample$ppier > 0))
y <- as.integer(sample$mode == 4L)
x <- log(sample$pcharter / sample$ppier)
stopifnot(all(is.finite(x)), length(unique(x)) >= 2L)

predict_rule <- function(direction, threshold, x) {
  if (direction == "constant_zero") return(rep(FALSE, length(x)))
  if (direction == "constant_one") return(rep(TRUE, length(x)))
  if (direction == "increasing") return(x > threshold)
  if (direction == "decreasing") return(x < threshold)
  stop("unknown maximum-score direction", call. = FALSE)
}

levels <- sort(unique(x))
candidates <- list(
  list(direction = "constant_zero", lower = NA_real_, upper = NA_real_, threshold = NA_real_),
  list(direction = "constant_one", lower = NA_real_, upper = NA_real_, threshold = NA_real_)
)
for (split in seq_len(length(levels) - 1L)) {
  lower <- levels[[split]]
  upper <- levels[[split + 1L]]
  midpoint <- lower / 2 + upper / 2
  increasing_threshold <- if (lower < midpoint && midpoint < upper) midpoint else lower
  decreasing_threshold <- if (lower < midpoint && midpoint < upper) midpoint else upper
  candidates[[length(candidates) + 1L]] <- list(
    direction = "increasing", lower = lower, upper = upper,
    threshold = increasing_threshold)
  candidates[[length(candidates) + 1L]] <- list(
    direction = "decreasing", lower = lower, upper = upper,
    threshold = decreasing_threshold)
}

results <- lapply(candidates, function(candidate) {
  predicted <- predict_rule(candidate$direction, candidate$threshold, x)
  candidate$predicted <- predicted
  candidate$correct <- sum(predicted == (y == 1L))
  candidate
})
best <- max(vapply(results, `[[`, numeric(1), "correct"))
optimal <- Filter(function(result) result$correct == best, results)
selected <- optimal[[1L]]
predicted <- selected$predicted
tp <- sum(y == 1L & predicted)
fp <- sum(y == 0L & predicted)
fn <- sum(y == 1L & !predicted)
tn <- sum(y == 0L & !predicted)
stopifnot(best == 536L, length(optimal) == 2L,
  tp == 421L, fp == 63L, fn == 31L, tn == 115L)

unit_coefficients <- function(result) {
  if (result$direction == "constant_zero") return(c(intercept = -1, slope = 0))
  if (result$direction == "constant_one") return(c(intercept = 1, slope = 0))
  norm <- sqrt(result$threshold^2 + 1)
  if (result$direction == "increasing") {
    return(c(intercept = -result$threshold / norm, slope = 1 / norm))
  }
  c(intercept = result$threshold / norm, slope = -1 / norm)
}

output <- do.call(rbind, lapply(seq_along(optimal), function(i) {
  result <- optimal[[i]]
  beta <- unit_coefficients(result)
  data.frame(
    region_order = i,
    direction = result$direction,
    lower = result$lower,
    upper = result$upper,
    threshold = result$threshold,
    intercept = beta[["intercept"]],
    slope = beta[["slope"]],
    observation_count = length(y),
    event_count = sum(y),
    correct_count = result$correct,
    empirical_score = result$correct / length(y),
    true_positive = tp,
    false_positive = fp,
    false_negative = fn,
    true_negative = tn,
    stringsAsFactors = FALSE
  )
}))
write.csv(output, args[[2L]], row.names = FALSE, na = "")
