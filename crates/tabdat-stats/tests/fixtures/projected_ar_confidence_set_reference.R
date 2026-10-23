# Independent base-R reference for the analytic projected AR fixture.
# The profile is minimized with optim over two nuisance coefficients; each
# objective evaluation independently fits both restricted and unrestricted OLS.
n <- 80
basis <- function(k) cos(2 * pi * k * seq_len(n) / n)
w <- basis(7)
z <- cbind(basis(1), basis(2), basis(3))
x <- cbind(
  0.55 * basis(1) + basis(4),
  1.40 * basis(2) + basis(5),
  1.20 * basis(3) + basis(6)
)
y <- 1.3 * x[, 1] - 0.6 * x[, 2] + 0.15 * x[, 3] + 0.5 * w + basis(8)
W <- cbind(1, w)
full <- cbind(W, z)
df <- n - ncol(full)
critical <- qf(0.95, ncol(z), df)
k <- 1 + ncol(z) * critical / df

ar_q <- function(beta) {
  adjusted <- y - drop(x %*% beta)
  rss_restricted <- sum(lm.fit(W, adjusted)$residuals^2)
  rss_unrestricted <- sum(lm.fit(full, adjusted)$residuals^2)
  rss_restricted - k * rss_unrestricted
}
profile <- function(beta1) {
  optim(c(-0.6, 0.15), function(nuisance) ar_q(c(beta1, nuisance)),
        method = "BFGS", control = list(reltol = 1e-14, maxit = 1000))$value
}
center <- 1.3
lo <- uniroot(function(offset) profile(center + offset), c(-5, 0), tol = 1e-12)$root
hi <- uniroot(function(offset) profile(center + offset), c(0, 5), tol = 1e-12)$root
cat(sprintf("critical_f=%.15f\n", critical))
cat(sprintf("lower=%.15f\n", center + lo))
cat(sprintf("upper=%.15f\n", center + hi))
cat(sprintf("profile_center=%.15f\n", profile(center)))
