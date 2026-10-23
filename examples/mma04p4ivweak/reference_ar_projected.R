#!/usr/bin/env Rscript
# Independent base-R projection reference for the three-endogenous DATA66 fit.
# The profile is minimized with optim over exp76/expsq76 and uses lm.fit for
# each restricted/unrestricted auxiliary OLS regression.
args <- commandArgs(trailingOnly = TRUE)
if (length(args) != 1L) stop("usage: reference_ar_projected.R DATA66.csv")

data <- read.csv(args[[1]], na.strings = "", check.names = FALSE)
included <- c(
  "black", "south76", "smsa76", paste0("reg", 2:9), "smsa66", "momdad14",
  "sinmom14", "nodaded", "nomomed", "daded", "momed", paste0("famed", 1:8)
)
endogenous <- c("grade76", "exp76", "expsq76")
excluded <- c("col4", "age76", "agesq76")
outcome <- "wage76"
variables <- c("nlsflt", outcome, included, endogenous, excluded)
data <- data[data$nlsflt == 1, , drop = FALSE]
data <- data[complete.cases(data[, variables]), , drop = FALSE]

W <- model.matrix(reformulate(included), data)
Z <- cbind(W, as.matrix(data[, excluded]))
E <- as.matrix(data[, endogenous])
S <- cbind(W, E)
y <- data[[outcome]]
Qz <- qr.Q(qr(Z))
Pz <- Qz %*% t(Qz)
beta_2sls <- solve(crossprod(S, Pz %*% S), crossprod(S, Pz %*% y))
center <- unname(beta_2sls[ncol(W) + 1L, 1L])
nuisance_center <- unname(beta_2sls[(ncol(W) + 2L):(ncol(W) + 3L), 1L])
df <- nrow(data) - ncol(Z)
q <- length(excluded)
critical <- qf(0.95, q, df)
k <- 1 + q * critical / df
profile <- function(grade) {
  optim(nuisance_center, function(nuisance) {
    adjusted <- y - drop(E %*% c(grade, nuisance))
    rss_restricted <- sum(lm.fit(W, adjusted)$residuals^2)
    rss_unrestricted <- sum(lm.fit(Z, adjusted)$residuals^2)
    rss_restricted - k * rss_unrestricted
  }, method = "BFGS", control = list(reltol = 1e-12, maxit = 1000))$value
}

root <- function(direction) {
  center_value <- profile(center)
  if (!is.finite(center_value) || center_value > 0) {
    stop("the 2SLS center is not in the profiled AR set")
  }
  step <- max(0.01, abs(center) * 0.01)
  outside <- center + direction * step
  value <- profile(outside)
  for (iteration in seq_len(60L)) {
    if (is.finite(value) && value >= 0) {
      bounds <- sort(c(center, outside))
      return(uniroot(profile, bounds, tol = 1e-10)$root)
    }
    step <- step * 2
    outside <- center + direction * step
    value <- profile(outside)
  }
  NA_real_
}

lower <- root(-1)
upper <- root(1)
cat(sprintf("N=%d\n", nrow(data)))
cat(sprintf("critical_f=%.15f\n", critical))
cat(sprintf("grade_2sls_center=%.15f\n", center))
cat(sprintf("profile_at_center=%.15f\n", profile(center)))
cat(sprintf("lower=%s\n", ifelse(is.finite(lower), sprintf("%.15f", lower), "-Inf")))
cat(sprintf("upper=%s\n", ifelse(is.finite(upper), sprintf("%.15f", upper), "+Inf")))
