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

fit_discrete_one <- function(data, family, design) {
  control <- glm.control(epsilon = 1e-13, maxit = 100L)
  if (design == "default") {
    glm(y ~ treated + x + region, data = data, family = family,
        na.action = na.exclude, control = control)
  } else if (design == "west_reference") {
    glm(y ~ treated + x + relevel(region, ref = "West"), data = data,
        family = family, na.action = na.exclude, control = control)
  } else if (design == "noconstant") {
    glm(y ~ 0 + treated + x + region, data = data, family = family,
        na.action = na.exclude, control = control)
  } else if (design == "numeric_factor") {
    glm(y ~ treated + x + factor(group_code), data = data, family = family,
        na.action = na.exclude, control = control)
  } else if (design == "boolean_factor") {
    glm(y ~ treated + x + factor(is_west), data = data, family = family,
        na.action = na.exclude, control = control)
  } else {
    stop("unknown discrete-change design")
  }
}

# Polish Base R glm starts with R's own link, matrix and QR/solve operations so
# all references use a score norm below 1e-13 rather than glm's stopping delta.
logit_polished_reference <- function(fit) {
  X <- model.matrix(fit)
  y <- fit$y
  beta <- coef(fit)
  for (iteration in seq_len(30L)) {
    eta <- drop(X %*% beta)
    probability <- plogis(eta)
    variance <- probability * (1 - probability)
    score <- drop(crossprod(X, y - probability))
    if (sqrt(sum(score^2)) < 1e-13) break
    beta <- beta + solve(crossprod(X, X * variance), score)
  }
  eta <- drop(X %*% beta)
  probability <- plogis(eta)
  score <- drop(crossprod(X, y - probability))
  stopifnot(sqrt(sum(score^2)) < 1e-12)
  list(beta = beta, covariance = solve(crossprod(X, X * probability * (1 - probability))))
}

# Base R glm's probit covariance is Fisher/IRLS information, while the
# executable's documented probit contract uses inverse observed information.
# Polish the glm start with R's pnorm/dnorm and matrix operations, then
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
factor_contrast_rows <- list()
continuous_marginal_effect_rows <- list()
discrete_change_rows <- list()
binary_lincom_rows <- list()
binary_classification_rows <- list()
binary_probability_prediction_rows <- list()
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
append_classification <- function(model, fit, estimates, family, cutoff) {
  X <- model.matrix(fit)
  probability <- family$linkinv(drop(X %*% estimates))
  actual <- fit$y == 1
  predicted <- probability >= cutoff
  tp <- sum(actual & predicted)
  fp <- sum(!actual & predicted)
  fn <- sum(actual & !predicted)
  tn <- sum(!actual & !predicted)
  percent <- function(numerator, denominator) {
    if (denominator == 0) NA_real_ else 100 * numerator / denominator
  }
  binary_classification_rows[[length(binary_classification_rows) + 1L]] <<-
    data.frame(
      model = model,
      cutoff = cutoff,
      observation_count = length(actual),
      actual_positive_count = sum(actual),
      actual_negative_count = sum(!actual),
      predicted_positive_count = sum(predicted),
      predicted_negative_count = sum(!predicted),
      true_positive = tp,
      false_positive = fp,
      false_negative = fn,
      true_negative = tn,
      sensitivity = percent(tp, tp + fn),
      specificity = percent(tn, tn + fp),
      positive_predictive_value = percent(tp, tp + fp),
      negative_predictive_value = percent(tn, tn + fn),
      false_positive_rate_given_negative = percent(fp, fp + tn),
      false_negative_rate_given_positive = percent(fn, tp + fn),
      false_positive_rate_given_positive = percent(fp, tp + fp),
      false_negative_rate_given_negative = percent(fn, tn + fn),
      correctly_classified = percent(tp + tn, length(actual))
    )
}
append_probability_predictions <- function(model, fit, estimates, covariance_type,
                                           covariance, family, design_name,
                                           requires_region) {
  for (row in seq_len(nrow(data))) {
    if (is.na(data$x[row]) || (requires_region && is.na(data$region[row]))) {
      eta <- probability <- standard_error <- NA_real_
      lower <- upper <- rep(NA_real_, 2L)
    } else {
      design <- model.matrix(delete.response(terms(fit)), data = data[row, , drop = FALSE],
                             contrasts.arg = fit$contrasts)
      eta <- drop(design %*% estimates)
      probability <- family$linkinv(eta)
      standard_error_eta <- sqrt(drop(design %*% covariance %*% t(design)))
      standard_error <- family$mu.eta(eta) * standard_error_eta
      levels <- c(95, 90)
      lower <- family$linkinv(eta - qnorm(0.5 + levels / 200) * standard_error_eta)
      upper <- family$linkinv(eta + qnorm(0.5 + levels / 200) * standard_error_eta)
    }
    for (level_index in seq_along(c(95, 90))) {
      level <- c(95, 90)[level_index]
      binary_probability_prediction_rows[[length(binary_probability_prediction_rows) + 1L]] <<-
        data.frame(
          model = model,
          design = design_name,
          covariance_type = covariance_type,
          id = id[row],
          level = level,
          estimate = probability,
          standard_error = standard_error,
          lower = lower[level_index],
          upper = upper[level_index]
        )
    }
  }
}
set_factor_level <- function(data, variable, values, levels) {
  if (variable == "region") {
    data$region <- factor(values, levels = levels)
  } else if (variable == "group_code") {
    data$group_code <- factor(as.numeric(values),
                              levels = as.numeric(levels))
  } else if (variable == "is_west") {
    data$is_west <- factor(values == "true",
                           levels = c(FALSE, TRUE))
  } else {
    stop("unknown factor variable")
  }
  data
}
append_factor_contrasts <- function(model, design, covariance_type, fit,
                                    covariance, estimates, family, factor_variable,
                                    factor_levels, baseline_level) {
  retained_rows <- as.integer(rownames(model.frame(fit)))
  retained_data <- data[retained_rows, , drop = FALSE]
  contrast_matrix <- function(level) {
    support <- retained_data[rep(1L, length(factor_levels)), , drop = FALSE]
    newdata <- rbind(retained_data, support)
    values <- c(rep(level, nrow(retained_data)), factor_levels)
    newdata <- set_factor_level(newdata, factor_variable, values, factor_levels)
    matrix <- model.matrix(delete.response(terms(fit)), data = newdata,
                           contrasts.arg = fit$contrasts)
    matrix[seq_len(nrow(retained_data)), , drop = FALSE]
  }
  baseline_matrix <- contrast_matrix(baseline_level)
  baseline_eta <- drop(baseline_matrix %*% estimates)
  baseline_probability <- family$linkinv(baseline_eta)
  baseline_derivative <- family$mu.eta(baseline_eta)
  for (level in factor_levels[factor_levels != baseline_level]) {
    target_matrix <- contrast_matrix(level)
    target_eta <- drop(target_matrix %*% estimates)
    target_probability <- family$linkinv(target_eta)
    target_derivative <- family$mu.eta(target_eta)
    gradient <- colMeans(
      sweep(target_matrix, 1L, target_derivative, `*`) -
        sweep(baseline_matrix, 1L, baseline_derivative, `*`)
    )
    estimate <- mean(target_probability - baseline_probability)
    standard_error <- sqrt(drop(gradient %*% covariance %*% gradient))
    statistic <- estimate / standard_error
    factor_contrast_rows[[length(factor_contrast_rows) + 1L]] <<- data.frame(
      model = model,
      design = design,
      covariance_type = covariance_type,
      factor_variable = factor_variable,
      baseline_level = baseline_level,
      comparison_level = level,
      estimate = estimate,
      standard_error = standard_error,
      statistic = statistic,
      p_value = 2 * pnorm(-abs(statistic)),
      confidence_lower = estimate - qnorm(0.975) * standard_error,
      confidence_upper = estimate + qnorm(0.975) * standard_error
    )
  }
}
append_continuous_marginal_effects <- function(model, design, covariance_type,
                                               fit, covariance, estimates) {
  X <- model.matrix(fit)
  eta <- drop(X %*% estimates)
  if (model == "logit") {
    probability <- plogis(eta)
    first_derivative <- probability * (1 - probability)
    second_derivative <- first_derivative * (1 - 2 * probability)
  } else {
    first_derivative <- dnorm(eta)
    second_derivative <- -eta * first_derivative
  }
  for (variable in "x") {
    index <- match(variable, colnames(X))
    stopifnot(!is.na(index))
    estimate <- estimates[index] * mean(first_derivative)
    gradient <- estimates[index] * colMeans(sweep(X, 1L, second_derivative, `*`))
    gradient[index] <- gradient[index] + mean(first_derivative)
    standard_error <- sqrt(drop(gradient %*% covariance %*% gradient))
    statistic <- estimate / standard_error
    continuous_marginal_effect_rows[[length(continuous_marginal_effect_rows) + 1L]] <<-
      data.frame(
        model = model,
        design = design,
        covariance_type = covariance_type,
        variable = variable,
        estimate = estimate,
        standard_error = standard_error,
        statistic = statistic,
        p_value = 2 * pnorm(-abs(statistic)),
        confidence_lower = estimate - qnorm(0.975) * standard_error,
        confidence_upper = estimate + qnorm(0.975) * standard_error
      )
  }
}
append_binary_lincom <- function(model, covariance_type, estimates, covariance) {
  append <- function(combination, weights, offset) {
    stopifnot(all(names(weights) %in% names(estimates)))
    weight_vector <- setNames(rep(0, length(estimates)), names(estimates))
    weight_vector[names(weights)] <- weights
    estimate <- offset + sum(weight_vector * estimates)
    standard_error <- sqrt(drop(weight_vector %*% covariance %*% weight_vector))
    statistic <- estimate / standard_error
    binary_lincom_rows[[length(binary_lincom_rows) + 1L]] <<- data.frame(
      model = model,
      covariance_type = covariance_type,
      combination = combination,
      estimate = estimate,
      standard_error = standard_error,
      statistic = statistic,
      p_value = 2 * pnorm(-abs(statistic)),
      confidence_lower = estimate - qnorm(0.975) * standard_error,
      confidence_upper = estimate + qnorm(0.975) * standard_error,
      confidence_level = 95
    )
  }
  append("slope_offset", c(x = 1), 0.5)
  append("factor_contrast", c(x = 1, regionNorth = 1, regionWest = -1), 0.5)
  append("intercept_slope", c(`(Intercept)` = 1, x = 1), 0)
}
append_discrete_change <- function(model, design, covariance_type, fit,
                                   covariance, estimates) {
  X <- model.matrix(fit)
  index <- match("treated", colnames(X))
  stopifnot(!is.na(index))
  from <- X
  to <- X
  from[, index] <- 0
  to[, index] <- 1
  eta_from <- drop(from %*% estimates)
  eta_to <- drop(to %*% estimates)
  if (model == "logit") {
    probability_from <- plogis(eta_from)
    probability_to <- plogis(eta_to)
    derivative_from <- probability_from * (1 - probability_from)
    derivative_to <- probability_to * (1 - probability_to)
  } else {
    probability_from <- pnorm(eta_from)
    probability_to <- pnorm(eta_to)
    derivative_from <- dnorm(eta_from)
    derivative_to <- dnorm(eta_to)
  }
  estimate <- mean(probability_to - probability_from)
  gradient <- colMeans(
    sweep(to, 1L, derivative_to, `*`) -
      sweep(from, 1L, derivative_from, `*`)
  )
  standard_error <- sqrt(drop(gradient %*% covariance %*% gradient))
  statistic <- estimate / standard_error
  discrete_change_rows[[length(discrete_change_rows) + 1L]] <<- data.frame(
    model = model,
    design = design,
    covariance_type = covariance_type,
    predictor = "treated",
    estimate = estimate,
    standard_error = standard_error,
    statistic = statistic,
    p_value = 2 * pnorm(-abs(statistic)),
    confidence_lower = estimate - qnorm(0.975) * standard_error,
    confidence_upper = estimate + qnorm(0.975) * standard_error
  )
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
      reference <- logit_polished_reference(fit)
      estimates <- reference$beta
      covariance <- reference$covariance
    }
    if (design == "default") {
      for (cutoff in c(0.5, 0.35)) {
        append_classification(model, fit, estimates, family, cutoff)
      }
    }
    factor_variable <- switch(
      design,
      numeric_factor = "group_code",
      boolean_factor = "is_west",
      "region"
    )
    factor_levels <- switch(
      design,
      numeric_factor = c("1", "2", "3"),
      boolean_factor = c("false", "true"),
      c("East", "North", "West")
    )
    baseline_level <- if (design == "west_reference") "West" else factor_levels[1L]
    append_factor_contrasts(model, design, "model_based", fit, covariance,
                            estimates, family, factor_variable, factor_levels,
                            baseline_level)
    append_continuous_marginal_effects(model, design, "model_based", fit,
                                       covariance, estimates)
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
      append_probability_predictions(model, fit, estimates, "model_based",
                                     covariance, family, "default", TRUE)
      append_binary_lincom(model, "model_based", estimates, covariance)
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
      append_probability_predictions(model, fit, estimates, "robust",
                                     robust_covariance, family, "default", TRUE)
      append_binary_lincom(model, "robust", estimates, robust_covariance)
      append_factor_contrasts(model, design, "robust", fit, robust_covariance,
                              estimates, family, factor_variable, factor_levels,
                              baseline_level)
      append_continuous_marginal_effects(model, design, "robust", fit,
                                         robust_covariance, estimates)
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
      append_probability_predictions(model, fit, estimates, "cluster",
                                     cluster_covariance, family, "default", TRUE)
      append_binary_lincom(model, "cluster", estimates, cluster_covariance)
      append_factor_contrasts(model, design, "cluster", fit, cluster_covariance,
                              estimates, family, factor_variable, factor_levels,
                              baseline_level)
      append_continuous_marginal_effects(model, design, "cluster", fit,
                                         cluster_covariance, estimates)
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

# A continuous-only model exercises the prediction path that has no retained
# factor design terms. The factor column is intentionally irrelevant here.
for (model in c("logit", "probit")) {
  family <- binomial(link = model)
  fit <- glm(y ~ x, data = data, family = family, na.action = na.exclude,
             control = glm.control(epsilon = 1e-13, maxit = 100L))
  reference <- if (model == "probit") {
    probit_observed_reference(fit)
  } else {
    logit_polished_reference(fit)
  }
  estimates <- reference$beta
  covariance <- reference$covariance
  append_probability_predictions(model, fit, estimates, "model_based", covariance,
                                 family, "continuous", FALSE)
  X <- model.matrix(fit)
  eta_sample <- drop(X %*% estimates)
  probability_sample <- family$linkinv(eta_sample)
  score_multiplier <- if (model == "logit") {
    fit$y - probability_sample
  } else {
    (fit$y - probability_sample) * dnorm(eta_sample) /
      (probability_sample * (1 - probability_sample))
  }
  score_matrix <- X * score_multiplier
  robust_covariance <- covariance %*% crossprod(score_matrix) %*% covariance *
    nrow(X) / (nrow(X) - 1L)
  append_probability_predictions(model, fit, estimates, "robust", robust_covariance,
                                 family, "continuous", FALSE)
  retained_rows <- as.integer(rownames(model.frame(fit)))
  cluster_scores <- rowsum(score_matrix, data$cluster[retained_rows], reorder = FALSE)
  cluster_covariance <- covariance %*% crossprod(cluster_scores) %*% covariance *
    nrow(cluster_scores) / (nrow(cluster_scores) - 1L)
  append_probability_predictions(model, fit, estimates, "cluster", cluster_covariance,
                                 family, "continuous", FALSE)
}

discrete_data <- transform(data, treated = as.integer(x > 0))
for (model in c("logit", "probit")) {
  family <- binomial(link = model)
  for (design in c("default", "west_reference", "noconstant",
                   "numeric_factor", "boolean_factor")) {
    fit <- fit_discrete_one(discrete_data, family, design)
    if (model == "probit") {
      reference <- probit_observed_reference(fit)
    } else {
      reference <- logit_polished_reference(fit)
    }
    estimates <- reference$beta
    covariance <- reference$covariance
    append_discrete_change(model, design, "model_based", fit,
                           covariance, estimates)
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
      append_discrete_change(model, design, "robust", fit,
                             robust_covariance, estimates)
      retained_rows <- as.integer(rownames(model.frame(fit)))
      cluster_scores <- rowsum(
        score_matrix,
        data$cluster[retained_rows],
        reorder = FALSE
      )
      cluster_count <- nrow(cluster_scores)
      cluster_covariance <- covariance %*% crossprod(cluster_scores) %*% covariance *
        cluster_count / (cluster_count - 1L)
      append_discrete_change(model, design, "cluster", fit,
                             cluster_covariance, estimates)
    }
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
write.csv(do.call(rbind, factor_contrast_rows),
          "binary_factor_contrast_reference.csv", row.names = FALSE)
write.csv(do.call(rbind, continuous_marginal_effect_rows),
          "binary_factor_marginal_effect_reference.csv", row.names = FALSE)
write.csv(do.call(rbind, discrete_change_rows),
          "binary_factor_discrete_change_reference.csv", row.names = FALSE)
write.csv(do.call(rbind, binary_lincom_rows),
          "binary_factor_lincom_reference.csv", row.names = FALSE)
write.csv(do.call(rbind, binary_classification_rows),
          "binary_factor_classification_reference.csv", row.names = FALSE)
write.csv(do.call(rbind, binary_probability_prediction_rows),
          "binary_factor_probability_inference_reference.csv",
          row.names = FALSE, na = "")
