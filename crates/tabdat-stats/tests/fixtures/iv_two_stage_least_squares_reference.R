# Independent reference for iv_two_stage_least_squares.rs.
# R 4.6.1 base stats::lm supplies each stage's QR fit. The covariance matrices
# below use the standard IV scores Xhat_i * residual_i and grouped score sums.
n <- 60L
i <- seq_len(n)
w <- sin(i / 7) + i / 90
z1 <- cos(i / 5) + i / 70
z2 <- sin(i / 11) - cos(i / 13) + i / 200
v1 <- sin(i / 3) + cos(i / 9)
v2 <- cos(i / 4) - sin(i / 6)
x1 <- 0.3 + 0.4 * w + 0.9 * z1 + 0.2 * z2 + v1
x2 <- -0.2 + 0.2 * w - 0.3 * z1 + 1.1 * z2 + v2
y <- 1.2 + 0.5 * w + 1.7 * x1 - 0.6 * x2 + sin(i / 1.7) + cos(i / 2.3) + sin(i / 5.9)

first_stage <- list(lm(x1 ~ w + z1 + z2), lm(x2 ~ w + z1 + z2))
restricted <- list(lm(x1 ~ w), lm(x2 ~ w))
xhat <- cbind(1, w, fitted(first_stage[[1]]), fitted(first_stage[[2]]))
x <- cbind(1, w, x1, x2)
beta <- drop(solve(crossprod(xhat), crossprod(xhat, y)))
e <- y - drop(x %*% beta)
bread <- solve(crossprod(xhat))
rss <- sum(e^2)
df <- n - ncol(x)
vcov_classical <- bread * rss / df
vcov_hc1 <- bread %*% (crossprod(xhat * e) * n / df) %*% bread
cluster <- rep(1:12, each = 5)
cluster_scores <- rowsum(xhat * e, cluster)
vcov_cluster <- bread %*% (crossprod(cluster_scores) * (12 / 11) * ((n - 1) / df)) %*% bread
first_stage_f <- function(full, reduced) {
  rss_full <- sum(resid(full)^2)
  rss_reduced <- sum(resid(reduced)^2)
  q <- 2
  c(f = ((rss_reduced - rss_full) / q) / (rss_full / df.residual(full)),
    partial_r_squared = (rss_reduced - rss_full) / rss_reduced)
}
# Godfrey's variance/R-squared identity for Shea partial R-squared, using
# classical OLS and IV covariance with identical residual-DF corrections:
# https://www.stata.com/statalist/archive/2008-03/msg00754.html
# This is a design diagnostic, independent of the requested reporting VCE.
ols <- lm(y ~ w + x1 + x2)
shea <- diag(vcov(ols)) / diag(vcov_classical) * rss / sum(resid(ols)^2)
print(list(
  beta = beta,
  classical_se = sqrt(diag(vcov_classical)),
  hc1_se = sqrt(diag(vcov_hc1)),
  cluster_se = sqrt(diag(vcov_cluster)),
  first_stage_1 = first_stage_f(first_stage[[1]], restricted[[1]]),
  first_stage_2 = first_stage_f(first_stage[[2]], restricted[[2]]),
  shea_partial_r_squared = shea[c("x1", "x2")]
), digits = 14)
