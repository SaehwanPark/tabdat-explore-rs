# Base R only. Regenerates the deterministic fixture and independent model
# references used by the continuous interaction integration test.
options(digits = 17)
script_path <- sub("^--file=", "", grep("^--file=", commandArgs(), value = TRUE)[1])
root <- dirname(normalizePath(script_path))
n <- 240L
id <- seq_len(n)
x <- sin(id * 0.37) + 0.15 * cos(id * 0.11)
z <- cos(id * 0.23) + ((id %% 5L) - 2L) * 0.1
u <- (((id * 37L) %% 997L) + 0.5) / 997
noise <- (((id * 71L) %% 101L) / 101 - 0.5) * 0.1
eta_logit <- -0.25 + 0.35 * x - 0.25 * z + 0.5 * x * z
eta_probit <- 0.2 - 0.2 * x + 0.3 * z + 0.45 * x * z
y_ols <- 1 + 0.4 * x - 0.3 * z + 0.55 * x * z + noise
y_logit <- as.integer(u < plogis(eta_logit))
y_probit <- as.integer(u < pnorm(eta_probit))
x[3] <- NA_real_
z[4] <- NA_real_
x[5] <- 1e200
z[5] <- 1e200
y_ols[6] <- NA_real_
y_logit[7] <- NA_real_
y_probit[8] <- NA_real_
data <- data.frame(id, y_ols, y_logit, y_probit, x, z)
write.csv(data, file.path(root, "continuous_interaction_data.csv"), row.names = FALSE, na = "")

records <- list()
add <- function(model, kind, name, value, se = NA_real_) {
  records[[length(records) + 1L]] <<- data.frame(
    model = model, kind = kind, name = name, value = value, se = se,
    stringsAsFactors = FALSE
  )
}
add_fit <- function(model_name, outcome_name, design_kind = c("full", "product_only"),
                    link = NULL) {
  design_kind <- match.arg(design_kind)
  sample <- is.finite(data[[outcome_name]]) & is.finite(x) & is.finite(z) & is.finite(x * z)
  xs <- x[sample]
  zs <- z[sample]
  ys <- data[[outcome_name]][sample]
  if (design_kind == "full") {
    X <- cbind(intercept = 1, x = xs, z = zs, x__interaction__z = xs * zs)
  } else {
    X <- cbind(intercept = 1, x__interaction__z = xs * zs)
  }
  if (is.null(link)) {
    fit <- lm.fit(X, ys)
    residual <- ys - drop(X %*% fit$coefficients)
    sigma2 <- sum(residual^2) / (length(ys) - ncol(X))
    covariance <- sigma2 * solve(crossprod(X))
    coefficients <- fit$coefficients
    add(model_name, "n", "fit", length(ys))
    for (j in seq_along(coefficients)) {
      add(model_name, "coefficient", colnames(X)[j], coefficients[j], sqrt(covariance[j, j]))
      for (k in seq_along(coefficients)) {
        add(model_name, "covariance", paste(colnames(X)[j], colnames(X)[k], sep = "|"), covariance[j, k])
      }
    }
    all_x <- cbind(intercept = 1, x = x, z = z, x__interaction__z = x * z)
    prediction <- drop(all_x %*% coefficients)
    prediction[!is.finite(all_x[, 1]) | !is.finite(all_x[, 2]) |
                 !is.finite(all_x[, 3]) | !is.finite(all_x[, 4])] <- NA_real_
    for (i in seq_len(n)) add(model_name, "prediction", as.character(i), prediction[i])
    return(invisible(NULL))
  }

  family <- binomial(link = link)
  fit <- glm.fit(
    x = X,
    y = ys,
    family = family,
    control = glm.control(maxit = 100, epsilon = 1e-13)
  )
  if (link == "logit") {
    coefficients <- fit$coefficients
    for (iteration in seq_len(30L)) {
      eta <- drop(X %*% coefficients)
      probability <- plogis(eta)
      variance <- probability * (1 - probability)
      score <- drop(crossprod(X, ys - probability))
      if (sqrt(sum(score^2)) < 1e-13) break
      coefficients <- coefficients + solve(crossprod(X, X * variance), score)
    }
    eta <- drop(X %*% coefficients)
    probability <- plogis(eta)
    covariance <- solve(crossprod(X, X * probability * (1 - probability)))
  } else {
    coefficients <- fit$coefficients
    for (iteration in seq_len(30L)) {
      eta <- drop(X %*% coefficients)
      probability <- pnorm(eta)
      density <- dnorm(eta)
      variance <- probability * (1 - probability)
      a <- density / variance
      residual <- ys - probability
      curvature <- density * a + residual * a * (eta + (1 - 2 * probability) * a)
      score <- drop(crossprod(X, residual * a))
      if (sqrt(sum(score^2)) < 1e-13) break
      coefficients <- coefficients + solve(crossprod(X, X * curvature), score)
    }
    eta <- drop(X %*% coefficients)
    probability <- pnorm(eta)
    density <- dnorm(eta)
    variance <- probability * (1 - probability)
    a <- density / variance
    residual <- ys - probability
    curvature <- density * a + residual * a * (eta + (1 - 2 * probability) * a)
    covariance <- solve(crossprod(X, X * curvature))
  }
  eta <- drop(X %*% coefficients)
  probability <- family$linkinv(eta)
  add(model_name, "n", "fit", length(ys))
  for (j in seq_along(coefficients)) {
    add(model_name, "coefficient", colnames(X)[j], coefficients[j], sqrt(covariance[j, j]))
  }
  if (link == "logit") {
    first_link <- probability * (1 - probability)
    second_link <- first_link * (1 - 2 * probability)
  } else {
    first_link <- dnorm(eta)
    second_link <- -eta * first_link
  }
  variables <- list(x = if (design_kind == "full") cbind(0, 1, 0, z[sample]) else cbind(0, z[sample]),
                    z = if (design_kind == "full") cbind(0, 0, 1, xs) else cbind(0, xs))
  for (variable in names(variables)) {
    derivative <- variables[[variable]]
    eta_derivative <- drop(derivative %*% coefficients)
    effect <- mean(first_link * eta_derivative)
    gradient <- colMeans(second_link * eta_derivative * X + first_link * derivative)
    se <- sqrt(drop(gradient %*% covariance %*% gradient))
    add(model_name, "ame", variable, effect, se)
  }
  all_x <- if (design_kind == "full") cbind(intercept = 1, x = x, z = z, x__interaction__z = x * z) else cbind(intercept = 1, x__interaction__z = x * z)
  all_eta <- drop(all_x %*% coefficients)
  all_probability <- family$linkinv(all_eta)
  all_valid <- apply(is.finite(all_x), 1, all)
  all_probability[!all_valid] <- NA_real_
  for (i in seq_len(n)) add(model_name, "prediction", as.character(i), all_probability[i])
  all_link_derivative <- if (link == "logit") {
    all_probability * (1 - all_probability)
  } else {
    dnorm(all_eta)
  }
  all_eta_variance <- rowSums((all_x %*% covariance) * all_x)
  all_eta_se <- sqrt(all_eta_variance)
  all_probability_se <- all_link_derivative * all_eta_se
  all_probability_lower <- family$linkinv(all_eta - qnorm(0.975) * all_eta_se)
  all_probability_upper <- family$linkinv(all_eta + qnorm(0.975) * all_eta_se)
  all_probability_se[!all_valid] <- NA_real_
  all_probability_lower[!all_valid] <- NA_real_
  all_probability_upper[!all_valid] <- NA_real_
  for (i in seq_len(n)) {
    add(model_name, "prediction_se", as.character(i), all_probability_se[i])
    add(model_name, "prediction_lower", as.character(i), all_probability_lower[i])
    add(model_name, "prediction_upper", as.character(i), all_probability_upper[i])
  }
}

add_fit("ols", "y_ols", "full")
add_fit("logit", "y_logit", "full", "logit")
add_fit("probit", "y_probit", "full", "probit")
add_fit("logit_product_only", "y_logit", "product_only", "logit")
add_fit("probit_product_only", "y_probit", "product_only", "probit")
write.csv(do.call(rbind, records), file.path(root, "continuous_interaction_reference.csv"), row.names = FALSE, na = "")
