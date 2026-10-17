# Development-only R 4.6.1 reference, no production R dependency.
# Source specification: mma14p1binary.do, C&T (2005) section 14.2, Table 14.2,
# classical probit column.
#
# Convergence convention: a plain glm fit (epsilon = binomial default 1e-13)
# stops on a *deviance-change* criterion. Near a probit optimum the deviance is
# flat: the stopped fit can still carry a gradient norm orders of magnitude
# above the MLE (on this design the glm stop has score ~2e-9 while the polished
# fixed point is ~4e-16). The reference therefore polishes glm's solution with
# Newton steps on the observed information until the gradient norm is < 1e-13,
# evaluated with R's own pnorm/dnorm/linear algebra, and prints the glm-stop and
# polished score norms as evidence. Note: Stata 8's probit output computes SEs
# from the penultimate Iterate Hessian, so its classical-column SEs can differ
# from these fixed-point values in trailing digits; this script does not
# replicate that lagged-iterate convention.
# Rscript examples/mma14p1binary/reference_probit.R Nldata.csv [R.csv [native.csv]]
args <- commandArgs(trailingOnly = TRUE)
stopifnot(length(args) %in% 1:3)
d <- read.csv(args[[1]])
stopifnot(nrow(d)==1182L, identical(d$source_row,seq_len(1182L)))
d <- d[d$mode %in% c(2,4), ]; d <- d[order(d$source_row), ]
d$charter <- as.integer(d$mode==4)
d$lnrelp <- log(d$pcharter/d$ppier)
stopifnot(nrow(d)==630L, sum(d$charter)==452L,
  sum(d$charter==0)==178L, all(is.finite(d$lnrelp)))

# Binary-probit per-row observed-information pieces. Variance==0 rows (exact
# tail saturation) have no curvature and no score contribution by construction;
# score is per-parameter and must NOT be masked by the per-row keep mask (R
# would pad it with NAs).
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
  score <- drop(crossprod(X, r*a))
  list(z=z, p=p, pdf=pdf, keep=keep, diag=diag, score=score,
    score_norm=sqrt(sum(score^2)))
}
polish <- function(X, y, b, tol = 1e-13) {
  glm_score <- curv(X, y, b)$score_norm
  for (it in 1:20) {
    c <- curv(X, y, b)
    if (c$score_norm < tol) {
      cat("score at glm stop:", sprintf("%g", glm_score),
        "; score at fixed point:", sprintf("%g", c$score_norm),
        "; polish steps:", it - 1L, "\n")
      return(b)
    }
    step <- solve(crossprod(X, X*c$diag), c$score)
    if (max(abs(step)) < 1e-15) stop("polish stalled at score ", c$score_norm)
    b <- b + step
  }
  stop("polish did not converge in 20 steps")
}

fit <- glm(charter~lnrelp,data=d,family=binomial(link="probit"),
  control=glm.control(epsilon=1e-13,maxit=100))
stopifnot(fit$converged, !fit$boundary)
X <- model.matrix(fit)
b <- polish(X, d$charter, coef(fit))
c <- curv(X, d$charter, b)
z <- c$z; p <- c$p; pdf <- c$pdf
V <- solve(crossprod(X, X*c$diag))
se <- sqrt(diag(V)); zz <- b/se
critical <- qnorm(.975)
ll <- sum(ifelse(d$charter==1,pnorm(z,log.p=TRUE),pnorm(-z,log.p=TRUE)))
null_ll <- sum(dbinom(d$charter,1,mean(d$charter),log=TRUE))
lr <- 2*(ll-null_ll)
# Conditional-on-observed-X continuous average partial derivative and delta SE.
ame <- b[2]*mean(pdf)
g <- -b[2]*as.vector(crossprod(X, pdf*z))/nrow(d)
g[2] <- g[2]+mean(pdf)
ame_se <- sqrt(drop(t(g)%*%V%*%g))
stopifnot(identical(as.vector(round(b,3)),c(1.194,-1.056)))
print(list(observations=nrow(d),events=sum(d$charter),parameters=b,covariance=V,
  standard_errors=se,z=zz,p_value=2*pnorm(-abs(zz)),
  lower95=b-critical*se,upper95=b+critical*se,log_likelihood=ll,
  null_log_likelihood=null_ll,likelihood_ratio=lr,
  lr_p_value=pchisq(lr,1,lower.tail=FALSE),
  mcfadden_r_squared=1-ll/null_ll,probability_summary=c(mean(p),min(p),max(p)),
  continuous_ame=ame,ame_gradient=g,ame_se=ame_se,
  ame_z=ame/ame_se,ame_p_value=2*pnorm(-abs(ame/ame_se)),
  ame_lower95=ame-critical*ame_se,ame_upper95=ame+critical*ame_se),digits=17)
if(length(args)>=2) write.csv(data.frame(source_row=d$source_row,charter=d$charter,
  lnrelp=d$lnrelp,probability=p),args[[2]],row.names=FALSE)
if(length(args)==3) {
  native <- read.csv(args[[3]])
  stopifnot(all(c("source_row","charter","lnrelp","phat","pindex") %in% names(native)),
    nrow(native)==630L, identical(native$source_row,d$source_row),
    identical(native$charter,d$charter),
    max(abs(native$lnrelp-d$lnrelp))<1e-12,
    max(abs(native$phat-p))<1e-10,
    max(abs(native$pindex-z))<1e-10)
  cat("Native comparison: all 630 source-row identities, outcomes, designs, probabilities and indices match.\n")
  print(c(max_probability_error=max(abs(native$phat-p)),
    max_index_error=max(abs(native$pindex-z))),digits=17)
}
