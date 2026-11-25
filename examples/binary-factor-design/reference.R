set.seed(84523)

n <- 240L
id <- seq_len(n)
x <- runif(n, min = -2.5, max = 2.5)
region <- factor(rep(c("East", "North", "West"), length.out = n),
                 levels = c("East", "North", "West"))
group_code <- as.integer(region)
is_west <- region == "West"
eta <- -0.4 + 0.7 * x + 0.65 * (region == "North") - 0.5 * (region == "West")
y <- rbinom(n, size = 1L, prob = plogis(eta))
cluster <- rep(seq_len(n / 10L), each = 10L)

# A missing factor, continuous predictor and outcome exercise joint complete
# cases while preserving rows with missing outcomes for current-data scoring.
region[5L] <- NA
group_code[5L] <- NA_integer_
is_west[5L] <- NA
x[12L] <- NA_real_
y[18L] <- NA_integer_

data <- data.frame(
  id = id,
  y = y,
  x = x,
  region = region,
  group_code = group_code,
  is_west = is_west,
  cluster = cluster
)
write.csv(data, "binary_factor_data.csv", row.names = FALSE, na = "")

fit_one <- function(data, family, design) {
  control <- glm.control(epsilon = 1e-13, maxit = 100L)
  if (design == "default") {
    glm(y ~ x + region, data = data, family = family, na.action = na.exclude,
        control = control)
  } else if (design == "west_reference") {
    glm(y ~ x + relevel(region, ref = "West"), data = data,
        family = family, na.action = na.exclude, control = control)
  } else if (design == "noconstant") {
    glm(y ~ 0 + x + region, data = data, family = family,
        na.action = na.exclude, control = control)
  } else if (design == "numeric_factor") {
    glm(y ~ x + factor(group_code), data = data, family = family,
        na.action = na.exclude, control = control)
  } else if (design == "boolean_factor") {
    glm(y ~ x + factor(is_west), data = data, family = family,
        na.action = na.exclude, control = control)
  } else {
    stop("unknown design")
  }
}

# Base R glm's probit covariance is Fisher/IRLS information, while the
# executable's documented probit contract uses inverse observed information.
# Polish the glm start with R's own pnorm/dnorm and QR/solve operations, then
# independently assemble the converged observed-information covariance.
probit_observed_reference <- function(fit) {
  X <- model.matrix(fit)
  y <- fit$y
  beta <- coef(fit)
  for (iteration in seq_len(30L)) {
    eta <- drop(X %*% beta)
    p <- pnorm(eta)
    pdf <- dnorm(eta)
    variance <- p * (1 - p)
    a <- pdf / variance
    residual <- y - p
    curvature <- pdf * a + residual * a * (eta + (1 - 2 * p) * a)
    score <- drop(crossprod(X, residual * a))
    if (sqrt(sum(score^2)) < 1e-13) break
    beta <- beta + solve(crossprod(X, X * curvature), score)
  }
  eta <- drop(X %*% beta)
  p <- pnorm(eta)
  pdf <- dnorm(eta)
  variance <- p * (1 - p)
  a <- pdf / variance
  residual <- y - p
  curvature <- pdf * a + residual * a * (eta + (1 - 2 * p) * a)
  score <- drop(crossprod(X, residual * a))
  stopifnot(sqrt(sum(score^2)) < 1e-12)
  list(beta = beta, covariance = solve(crossprod(X, X * curvature)))
}

fit_rows <- list()
covariance_rows <- list()
prediction_rows <- list()
fit_summary_rows <- list()
append_covariance <- function(model, design, type, covariance) {
  for (row in seq_len(nrow(covariance))) {
    for (column in seq_len(ncol(covariance))) {
      covariance_rows[[length(covariance_rows) + 1L]] <<- data.frame(
        model = model,
        design = design,
        covariance_type = type,
        row = row - 1L,
        column = column - 1L,
        value = covariance[row, column]
      )
    }
  }
}
for (model in c("logit", "probit")) {
  family <- binomial(link = model)
  for (design in c("default", "west_reference", "noconstant",
                   "numeric_factor", "boolean_factor")) {
    fit <- fit_one(data, family, design)
    if (model == "probit") {
      reference <- probit_observed_reference(fit)
      estimates <- reference$beta
      covariance <- reference$covariance
    } else {
      estimates <- coef(fit)
      covariance <- vcov(fit)
    }
    for (index in seq_along(estimates)) {
      fit_rows[[length(fit_rows) + 1L]] <- data.frame(
        model = model,
        design = design,
        coefficient_index = index - 1L,
        estimate = unname(estimates[index]),
        standard_error = sqrt(covariance[index, index])
      )
    }
    append_covariance(model, design, "model_based", covariance)
    if (design == "default") {
      X <- model.matrix(fit)
      eta_sample <- drop(X %*% estimates)
      probability_sample <- if (model == "logit") plogis(eta_sample) else pnorm(eta_sample)
      score_multiplier <- if (model == "logit") {
        fit$y - probability_sample
      } else {
        (fit$y - probability_sample) * dnorm(eta_sample) /
          (probability_sample * (1 - probability_sample))
      }
      score_matrix <- X * score_multiplier
      robust_covariance <- covariance %*% crossprod(score_matrix) %*% covariance *
        nrow(X) / (nrow(X) - 1L)
      append_covariance(model, design, "robust", robust_covariance)
      retained_rows <- as.integer(rownames(model.frame(fit)))
      cluster_scores <- rowsum(
        score_matrix,
        data$cluster[retained_rows],
        reorder = FALSE
      )
      cluster_count <- nrow(cluster_scores)
      cluster_covariance <- covariance %*% crossprod(cluster_scores) %*% covariance *
        cluster_count / (cluster_count - 1L)
      append_covariance(model, design, "cluster", cluster_covariance)
    }
    eta_hat <- predict(fit, newdata = data, type = "link")
    probability <- predict(fit, newdata = data, type = "response")
    for (row in seq_len(n)) {
      prediction_rows[[length(prediction_rows) + 1L]] <- data.frame(
        model = model,
        design = design,
        id = id[row],
        retained = !is.na(fitted(fit)[row]),
        xb = eta_hat[row],
        probability = probability[row]
      )
    }
    fit_summary_rows[[length(fit_summary_rows) + 1L]] <- data.frame(
      model = model,
      design = design,
      observations = nobs(fit),
      events = sum(fit$y),
      log_likelihood = as.numeric(logLik(fit))
    )
  }
}

write.csv(do.call(rbind, fit_rows), "binary_factor_fit_reference.csv",
          row.names = FALSE)
write.csv(do.call(rbind, covariance_rows), "binary_factor_covariance_reference.csv",
          row.names = FALSE)
write.csv(do.call(rbind, prediction_rows), "binary_factor_prediction_reference.csv",
          row.names = FALSE, na = "")
write.csv(do.call(rbind, fit_summary_rows), "binary_factor_summary_reference.csv",
          row.names = FALSE)
