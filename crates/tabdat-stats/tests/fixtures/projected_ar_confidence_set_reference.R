# Independent base-R reference for the analytic projected AR fixtures.
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

# Second fixture: a non-orthogonal design with nonzero nuisance cross terms.
# The orthogonal fixture above is symmetric around the 2SLS center, so a sign
# error in the cross-term profile would not change its reported endpoints.
n2 <- 64
wave <- function(k, phase = 0) sin(2 * pi * k * seq_len(n2) / n2 + phase)
w1 <- wave(1) + 0.2 * wave(3, pi / 2)
w2 <- wave(1, pi / 2) - 0.3 * wave(5)
z1 <- wave(2, pi / 2) + 0.4 * wave(1)
z2 <- wave(3) + 0.3 * wave(4, pi / 2)
z3 <- wave(4, pi / 2) + 0.2 * wave(2)
x1 <- 0.9 * z1 + 0.4 * z2 + 0.2 * w1 + 0.5 * wave(6)
x2 <- 0.3 * z1 + 0.8 * z2 + 0.5 * z3 + 0.2 * w2 + 0.4 * wave(7, pi / 2)
x3 <- 0.2 * z1 + 0.3 * z2 + 0.9 * z3 + 0.3 * w1 - 0.2 * w2 + 0.3 * wave(8)
y2 <- 1.2 * x1 - 0.5 * x2 + 0.3 * x3 + 0.7 * w1 - 0.4 * w2 +
  0.6 * z1 + 0.2 * z2 + wave(9)
W2 <- cbind(1, w1, w2)
Z2 <- cbind(W2, z1, z2, z3)
E2 <- cbind(x1, x2, x3)
df2 <- n2 - ncol(Z2)
critical2 <- qf(0.95, 3, df2)
k2 <- 1 + 3 * critical2 / df2
ar_q2 <- function(beta) {
  adjusted <- y2 - drop(E2 %*% beta)
  sum(lm.fit(W2, adjusted)$residuals^2) -
    k2 * sum(lm.fit(Z2, adjusted)$residuals^2)
}
profile2 <- function(beta1) {
  optim(c(-0.5, 0.3), function(nuisance) ar_q2(c(beta1, nuisance)),
        method = "BFGS", control = list(reltol = 1e-14, maxit = 2000))$value
}
Qz2 <- qr.Q(qr(Z2))
Pz2 <- Qz2 %*% t(Qz2)
S2 <- cbind(W2, E2)
beta2 <- solve(crossprod(S2, Pz2 %*% S2), crossprod(S2, Pz2 %*% y2))
center2 <- unname(beta2[ncol(W2) + 1L, 1L])
lo2 <- uniroot(profile2, c(center2 - 5, center2), tol = 1e-13)$root
hi2 <- uniroot(profile2, c(center2, center2 + 5), tol = 1e-13)$root
cat(sprintf("correlated_critical_f=%.15f\n", critical2))
cat(sprintf("correlated_center=%.15f\n", center2))
cat(sprintf("correlated_lower=%.15f\n", lo2))
cat(sprintf("correlated_upper=%.15f\n", hi2))
