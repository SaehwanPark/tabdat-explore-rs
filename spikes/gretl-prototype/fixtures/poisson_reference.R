# Independent R Poisson GLM, no runtime R dependency.
data <- read.csv(commandArgs(trailingOnly=TRUE)[1])
fit <- glm(y ~ x1 + x2, data=data, family=poisson(link="log"),
           control=glm.control(epsilon=1e-12, maxit=100))
stopifnot(fit$converged, nobs(fit) == 240,
          identical(rownames(model.frame(fit)), as.character(1:240)),
          identical(names(coef(fit)), c("(Intercept)", "x1", "x2")))
# For canonical Poisson, observed and expected information coincide.
x <- model.matrix(fit)
covariance <- solve(crossprod(x, fit$fitted.values*x))
stopifnot(max(abs(covariance-vcov(fit))) < 1e-10)
values <- c(coef(fit), sqrt(diag(covariance)), as.vector(t(covariance)), as.numeric(logLik(fit)))
cat("R,poisson,", paste(sprintf("%.17g", values), collapse=","), "\n", sep="")
cat(R.version.string, "\n", file=stderr())
