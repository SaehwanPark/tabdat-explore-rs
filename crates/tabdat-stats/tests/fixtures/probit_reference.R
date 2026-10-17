# Development-only independent R 4.6.1 reference for pure probit contracts.
# Rscript crates/tabdat-stats/tests/fixtures/probit_reference.R
#
# Convergence convention: a plain glm fit (epsilon=binomial default 1e-13) is
# only a first solution. glm stops on a *deviance-change* criterion, and near
# a probit optimum the deviance is flat: the stopped fit can still carry a
# gradient norm ~3e-8 with coefficients up to ~1e-9 away from the MLE (e.g.
# first variant below: score 3.0e-8 at the glm stop versus 7.3e-15 after the
# polish). Knowing the optimum to 1e-10 requires targeting the analytic
# score=0 point, so the reference polishes glm's solution with Newton steps
# on the observed information until the gradient norm is < 1e-13, evaluated
# with R's own pnorm/dnorm/linear algebra. The Newton/OIM engine under test
# targets that same point, and the polish's final score norm is printed as
# evidence for every design.
options(digits=17)
i <- 1:40
x1 <- ((i*7)%%17)/5-1.6
x2 <- ((i*11)%%19)/7-9/7
y <- as.integer(((i*13)%%37)<18+4*x1-3*x2)

# Binary-probit per-row observed-information pieces with the variance>0
# drop-out rule shared by the saturated design (p==0 or p==1 rows carry
# zero score and zero curvature exactly).
curv <- function(X, y, b) {
  z <- drop(X %*% b)
  p <- pnorm(z); pdf <- dnorm(z)
  variance <- p*(1-p); keep <- variance > 0
  a <- rep(0, length(p)); w <- rep(0, length(p))
  a[keep] <- pdf[keep]/variance[keep]
  w[keep] <- pdf[keep]*a[keep]
  r <- y - p
  diag <- w + r*a*(z + (1-2*p)*a)
  diag[!keep] <- 0
  # Rows with variance==0 carry a==0, so r*a is already zero there; score is
  # per-parameter and must NOT be masked by the per-row keep mask (R would
  # pad it with NAs).
  score <- drop(crossprod(X, r*a))
  list(z=z, p=p, pdf=pdf, a=a, keep=keep, diag=diag, score=score,
    score_norm=sqrt(sum(score^2)))
}

polish <- function(X, y, b, tol = 1e-13) {
  for (it in 1:20) {
    c <- curv(X, y, b)
    if (c$score_norm < tol) {
      cat("  polished in", it - 1L, "step(s); score", sprintf("%g", c$score_norm), "\n")
      return(b)
    }
    step <- solve(crossprod(X, X*c$diag), c$score)
    if (max(abs(step)) < 1e-15) stop("polish stalled at score ", c$score_norm)
    b <- b + step
  }
  stop("polish did not converge in 20 steps")
}

for(intercept in c(TRUE,FALSE)) for(two in c(TRUE,FALSE)) {
  f <- if(two) y~x1+x2 else y~x1
  if(!intercept) f <- update(f,~.-1)
  first <- glm(f,family=binomial(link="probit"),control=glm.control(epsilon=1e-13,maxit=100))
  stopifnot(first$converged)
  cat("\nintercept:",intercept," two predictors:",two,"\n")
  X <- model.matrix(first)
  b <- polish(X, y, coef(first))
  c <- curv(X, y, b)
  p <- c$p; pdf <- c$pdf; z <- c$z
  V <- solve(crossprod(X, X*c$diag))
  se <- sqrt(diag(V))
  k <- ncol(X)
  cat("score:",sprintf("%g",c$score_norm),"\n")
  cat("coefficients:",sprintf("%.17g",b),"\n")
  cat("coefficient_p:",sprintf("%.17g",2*pnorm(-abs(b/se))),"\n")
  cat("covariance:",sprintf("%.17g",V),"\n")
  cat("probabilities(1,20,40):",sprintf("%.17g",p[c(1,20,40)]),"\n")
  null_ll <- if(intercept) sum(dbinom(y,1,mean(y),log=TRUE)) else -40*log(2)
  ll <- sum(ifelse(y==1,pnorm(z,log.p=TRUE),pnorm(-z,log.p=TRUE)))
  lr <- 2*(ll-null_ll)
  df <- k - as.integer(intercept)
  cat("ll:",sprintf("%.17g",ll)," lr:",sprintf("%.17g",lr),
    " lr_p:",sprintf("%.17g",pchisq(lr,df,lower.tail=FALSE)),"\n")
  slope_idx <- if(intercept) 2:k else 1:k
  grad <- function(j) {
    g <- -b[j]*as.vector(crossprod(X, pdf*z))/40
    g[j] <- g[j] + mean(pdf)
    g
  }
  ame <- b[slope_idx]*mean(pdf)
  ame_se <- sapply(slope_idx, function(j) sqrt(drop(t(grad(j)) %*% V %*% grad(j))))
  ame_p <- 2*pnorm(-abs(ame/ame_se))
  cat("ame:",sprintf("%.17g",ame)," se:",sprintf("%.17g",ame_se),
    " p:",sprintf("%.17g",ame_p),"\n")
}

# Intercept-only reference: model with no predictors. The non-canonical
# probit link's observed information is n*p*q per observation in the logit
# case, but not here; print both the total and its inverse.
nnn <- 10
y <- c(0., 0., 0., 0., 0., 0., 0., 1., 1., 1.)
c0 <- qnorm(mean(y))
z <- rep(c0, nnn); p <- pnorm(z); pdf <- dnorm(z)
a <- pdf/(p*(1-p)); r <- y - p
diag <- pdf*a + r*a*(z + (1-2*p)*a)
cat("\n== intercept only\n")
cat("coef:", sprintf("%.17g", c0), "\n")
cat("total observed info:", sprintf("%.17g", sum(diag)), "\n")
cat("cov:", sprintf("%.17g", 1/sum(diag)), "\n")
cat("null_ll:", sprintf("%.17g", sum(dbinom(y,1,mean(y),log=TRUE))), "\n")

# Saturated-outlier problem: consistent huge-value rows vanish exactly like
# the engine (variance underflows, score/curvature guard to zero), and the
# identified 10-point model must remain finite.
n12 <- 12
xs <- c(-2., -2., -1., -1., 0., 0., 1., 1., 2., 2., -1000., 1000.)
y <- c(0., 0., 0., 1., 0., 1., 0., 1., 1., 1., 0., 1.)
first <- glm(y~xs,family=binomial(link="probit"),control=glm.control(epsilon=1e-13,maxit=100))
stopifnot(first$converged)
X <- model.matrix(first)
b <- polish(X, y, coef(first))
stopifnot(!first$boundary)
c <- curv(X, y, b)
p <- c$p; pdf <- c$pdf
V <- solve(crossprod(X, X*c$diag))
stopifnot(sum(c$keep) == 10L, p[11] < 0.5, p[12] > 0.5, y[11] == 0, y[12] == 1)
pdf_all <- pdf; pdf_all[!c$keep] <- 0
cat("\n== saturated outliers score:", sprintf("%g", c$score_norm), "\n")
cat("coefficients:", sprintf("%.17g", b), "\n")
cat("offdiag:", sprintf("%.17g", V[2,1]), "\n")
cat("cov:", sprintf("%.17g", V[1,1]), sprintf("%.17g", V[2,2]), "\n")
cat("probabilities(1,11,12):", sprintf("%.17g", p[1]), sprintf("%.17g", p[11]), sprintf("%.17g", p[12]), "\n")
null_ll <- sum(dbinom(y,1,mean(y),log=TRUE))
ll <- sum(ifelse(y==1,pnorm(c$z,log.p=TRUE),pnorm(-c$z,log.p=TRUE)))
cat("ll:", sprintf("%.17g", ll), " null_ll:", sprintf("%.17g", null_ll),
  " lr:", sprintf("%.17g", 2*(ll - null_ll)), "\n")
ame <- b[2]*mean(pdf_all)
grad <- -b[2]*as.vector(crossprod(X, pdf_all*c$z))/n12
grad[2] <- grad[2] + mean(pdf_all)
ame_se <- sqrt(drop(t(grad) %*% V %*% grad))
cat("ame 2:", sprintf("%.17g", ame), " se:", sprintf("%.17g", ame_se),
  " z:", sprintf("%.17g", ame/ame_se),
  " p:", sprintf("%.17g", 2*pnorm(-abs(ame/ame_se))), "\n")
