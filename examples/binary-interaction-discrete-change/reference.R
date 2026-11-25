# Base R only. Independently computes 0-to-1 probability changes when the
# original binary source appears in a continuous product, with and without its
# own main effect. The input fixture is shared with binary-discrete-change.
options(digits = 17)
script_path <- sub("^--file=", "", grep("^--file=", commandArgs(), value = TRUE)[1])
root <- dirname(normalizePath(script_path))
data <- read.csv(
  file.path(root, "../binary-discrete-change/binary_discrete_change.csv"),
  check.names = FALSE
)

fit_one <- function(link, design, covariance) {
  retained <- is.finite(data$y) & is.finite(data$x) & is.finite(data$treated) &
    data$group != 1
  y <- data$y[retained]
  x <- data$x[retained]
  z <- x^2
  treated <- data$treated[retained]
  group <- data$group[retained]
  region <- factor(ceiling(group / 7))
  if (design == "full") {
    X <- cbind(intercept = 1, x = x, z = z, treated = treated,
               treated__interaction__x = treated * x,
               treated__interaction__z = treated * z)
  } else if (design == "full_factor") {
    group_columns <- model.matrix(~ region)[, -1, drop = FALSE]
    X <- cbind(intercept = 1, x = x, z = z, treated = treated,
               treated__interaction__x = treated * x,
               treated__interaction__z = treated * z, group_columns)
  } else {
    X <- cbind(intercept = 1, treated__interaction__x = treated * x,
               treated__interaction__z = treated * z)
  }
  initial <- glm.fit(
    x = X, y = y, family = binomial(link = link),
    control = glm.control(maxit = 100L, epsilon = 1e-13)
  )$coefficients
  beta <- initial
  for (iteration in seq_len(30L)) {
    eta <- drop(X %*% beta)
    probability <- if (link == "logit") plogis(eta) else pnorm(eta)
    density <- dnorm(eta)
    variance <- probability * (1 - probability)
    if (link == "logit") {
      score_slope <- y - probability
      information_weight <- variance
    } else {
      ratio <- density / variance
      score_slope <- (y - probability) * ratio
      information_weight <- density * ratio +
        (y - probability) * ratio * (eta + (1 - 2 * probability) * ratio)
    }
    score <- drop(crossprod(X, score_slope))
    if (max(abs(score)) < 1e-13) break
    information <- crossprod(X, X * information_weight)
    beta <- beta + drop(solve(information, score))
  }

  eta <- drop(X %*% beta)
  probability <- if (link == "logit") plogis(eta) else pnorm(eta)
  density <- dnorm(eta)
  variance <- probability * (1 - probability)
  if (link == "logit") {
    score_slope <- y - probability
    information_weight <- variance
  } else {
    ratio <- density / variance
    score_slope <- (y - probability) * ratio
    information_weight <- density * ratio +
      (y - probability) * ratio * (eta + (1 - 2 * probability) * ratio)
  }
  bread <- solve(crossprod(X, X * information_weight))
  scores <- X * score_slope
  covariance_matrix <- switch(covariance,
    model = bread,
    robust = length(y) / (length(y) - 1) * bread %*% crossprod(scores) %*% bread,
    cluster = {
      grouped_scores <- rowsum(scores, group, reorder = FALSE)
      group_count <- nrow(grouped_scores)
      group_count / (group_count - 1) * bread %*%
        crossprod(grouped_scores) %*% bread
    }
  )

  if (design == "full") {
    X0 <- cbind(intercept = 1, x = x, z = z, treated = 0,
                treated__interaction__x = 0, treated__interaction__z = 0)
    X1 <- cbind(intercept = 1, x = x, z = z, treated = 1,
                treated__interaction__x = x, treated__interaction__z = z)
  } else if (design == "full_factor") {
    X0 <- cbind(intercept = 1, x = x, z = z, treated = 0,
                treated__interaction__x = 0, treated__interaction__z = 0,
                group_columns)
    X1 <- cbind(intercept = 1, x = x, z = z, treated = 1,
                treated__interaction__x = x, treated__interaction__z = z,
                group_columns)
  } else {
    X0 <- cbind(intercept = 1, treated__interaction__x = rep(0, length(x)),
                treated__interaction__z = rep(0, length(x)))
    X1 <- cbind(intercept = 1, treated__interaction__x = x,
                treated__interaction__z = z)
  }
  eta0 <- drop(X0 %*% beta)
  eta1 <- drop(X1 %*% beta)
  probability0 <- if (link == "logit") plogis(eta0) else pnorm(eta0)
  probability1 <- if (link == "logit") plogis(eta1) else pnorm(eta1)
  derivative0 <- if (link == "logit") probability0 * (1 - probability0) else dnorm(eta0)
  derivative1 <- if (link == "logit") probability1 * (1 - probability1) else dnorm(eta1)
  gradient <- colMeans(X1 * derivative1 - X0 * derivative0)
  estimate <- mean(probability1 - probability0)
  standard_error <- sqrt(drop(t(gradient) %*% covariance_matrix %*% gradient))
  statistic <- estimate / standard_error
  data.frame(
    model = link,
    design = design,
    covariance = covariance,
    observation_count = length(y),
    group_count = if (covariance == "cluster") length(unique(group)) else NA_integer_,
    estimate = estimate,
    standard_error = standard_error,
    statistic = statistic,
    p_value = 2 * pnorm(-abs(statistic)),
    confidence_lower = estimate - 1.959963984540054 * standard_error,
    confidence_upper = estimate + 1.959963984540054 * standard_error
  )
}

references <- do.call(rbind, lapply(c("logit", "probit"), function(link) {
  do.call(rbind, lapply(c("full", "full_factor", "product_only"), function(design) {
    do.call(rbind, lapply(c("model", "robust", "cluster"), function(covariance) {
      fit_one(link, design, covariance)
    }))
  }))
}))
write.csv(references, stdout(), row.names = FALSE, quote = FALSE)
