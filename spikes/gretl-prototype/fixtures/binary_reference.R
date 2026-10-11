# Independent R glm MLE, no runtime R dependency.
data <- read.csv(commandArgs(trailingOnly=TRUE)[1])
for (mode in c("logit", "probit")) {
  fit <- glm(y ~ x1 + x2, data=data, family=binomial(link=mode), control=glm.control(epsilon=1e-12, maxit=100))
  stopifnot(fit$converged, nobs(fit) == 200, identical(rownames(model.frame(fit)), as.character(1:200)))
  # Observed-Hessian covariance for comparison with gretl/statsmodels.
  # glm's Fisher-information covariance can differ for the noncanonical probit link.
  x <- model.matrix(fit)
  eta <- fit$linear.predictors
  p <- fit$fitted.values
  if (mode == "logit") {
    weights <- p * (1-p)
  } else {
    density <- dnorm(eta)
    score_derivative <- -eta * density * (data$y - p) / (p * (1-p)) - density^2 / (p*(1-p)) - density^2 * (data$y-p) * (1-2*p) / (p^2 * (1-p)^2)
    weights <- -score_derivative
  }
  information <- crossprod(x, weights*x)
  numerical <- optimHess(coef(fit), function(beta) {
    linear <- as.vector(x %*% beta)
    probability <- if (mode == "logit") plogis(linear) else pnorm(linear)
    -sum(dbinom(data$y, size=1, prob=probability, log=TRUE))
  })
  stopifnot(max(abs(information-numerical)) < 1e-4)
  covariance <- solve(information)
  values <- c(coef(fit), sqrt(diag(covariance)), as.vector(t(covariance)), as.numeric(logLik(fit)))
  cat("R,", mode, ",", paste(sprintf("%.17g", values), collapse=","), "\n", sep="")
}
cat(R.version.string, "\n", file=stderr())
