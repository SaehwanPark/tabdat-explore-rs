# Development-only calculation of Abadie and Imbens (2016), section 4.
# Input: CSV with y, t, x1 and x2. No trimming, score ties, or survey weights.
# This is not an inference implementation for the NSW illustration.
args <- commandArgs(trailingOnly = TRUE)
if (length(args) != 1) stop("usage: Rscript nearest_variance_prototype.R data.csv")
d <- read.csv(args[[1]])
if (!all(c("y", "t", "x1", "x2") %in% names(d))) stop("missing required columns")
d <- d[, c("y", "t", "x1", "x2")]
if (any(!is.finite(as.matrix(d))) || !all(d$t %in% c(0, 1))) stop("invalid data")
if (min(table(factor(d$t, levels = c(0, 1)))) < 3) stop("need at least three observations per arm")
fit <- glm(t ~ x1 + x2, data = d, family = binomial())
if (!fit$converged || any(!is.finite(coef(fit)))) stop("logit failed")
p <- fitted(fit)
if (any(p <= 0 | p >= 1) || anyDuplicated(p)) stop("scores must be interior and distinct")
x <- model.matrix(fit)
n <- nrow(d)
n1 <- sum(d$t)
# Diagonal standardization defines the covariate matching metric for d_t.
sds <- apply(x[, -1, drop = FALSE], 2, sd)
if (any(!is.finite(sds) | sds <= 0)) stop("invalid covariate scale")
z <- sweep(x[, -1, drop = FALSE], 2, sds, "/")
score_distance <- abs(outer(p, p, "-"))
cov_distance <- matrix(0, n, n)
for (j in seq_len(ncol(z))) cov_distance <- cov_distance + outer(z[, j], z[, j], "-")^2
nearest <- function(distance, candidates, count) {
  ord <- order(distance[candidates])
  v <- distance[candidates][ord]
  if (length(v) > count && v[count] == v[count + 1]) stop("distance tie at matching boundary")
  candidates[ord[seq_len(count)]]
}
mu <- matrix(NA_real_, n, 2)
local_cov <- array(NA_real_, c(n, ncol(x), 2))
local_var <- numeric(n)
opp_score <- opp_x <- integer(n)
for (i in seq_len(n)) {
  for (arm in 0:1) {
    candidates <- which(d$t == arm)
    two <- nearest(score_distance[i, ], candidates, 2)
    local_cov[i, , arm + 1] <- (x[two[1], ] - x[two[2], ]) * (d$y[two[1]] - d$y[two[2]]) / 2
    if (arm == d$t[i]) {
      local_var[i] <- (d$y[two[1]] - d$y[two[2]])^2 / 2
      candidates <- candidates[candidates != i]
    }
    one <- nearest(score_distance[i, ], candidates, 1)
    mu[i, arm + 1] <- d$y[one]
    if (arm != d$t[i]) opp_score[i] <- one
  }
  opp_x[i] <- nearest(cov_distance[i, ], which(d$t != d$t[i]), 1)
}
treated <- which(d$t == 1)
control <- which(d$t == 0)
theta <- mean(d$y[treated] - d$y[opp_score[treated]])
k <- tabulate(opp_score[treated], nbins = n)
base <- (sum((d$y[treated] - d$y[opp_score[treated]] - theta)^2) +
  sum(k[control] * (k[control] - 1) * local_var[control])) / n1^2
f <- p * (1 - p)
c1 <- colSums(x * (f * (mu[, 2] - mu[, 1] - theta))) / n1
c2 <- colSums((local_cov[, , 2] + local_cov[, , 1] * (p / (1 - p))) * f) / n1
c <- c1 + c2
d_t <- colSums(x * (f * ((2 * d$t - 1) * (d$y - d$y[opp_x]) - theta))) / n1
information <- crossprod(x, x * f)
c_adjust <- drop(crossprod(c, solve(information, c)))
d_adjust <- drop(crossprod(d_t, solve(information, d_t)))
v <- base - c_adjust + d_adjust
if (!is.finite(v) || v <= 0) stop("nonpositive or nonfinite variance")
cat(sprintf("n=%d treated=%d effect=%.10f base=%.10f c_adjust=%.10f d_adjust=%.10f se=%.10f\n", n, n1, theta, base, c_adjust, d_adjust, sqrt(v)))
