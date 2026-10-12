library(survival)
d <- read.csv(commandArgs(trailingOnly=TRUE)[1])
for (family in c('exponential','weibull')) {
  time <- if(family=='exponential') d$time_exp else d$time_weibull
  fail <- if(family=='exponential') d$fail_exp else d$fail_weibull
  fit <- survreg(Surv(time,fail) ~ x1+x2,data=d,dist=family,control=survreg.control(rel.tolerance=1e-12,maxiter=100))
  stopifnot(nobs(fit)==320)
  b <- coef(fit);s <- fit$scale;k <- length(b);dim <- k+(family=='weibull')
  J <- matrix(0,dim,dim);diag(J)[1:k] <- -1/s
  if(family=='weibull') {J[1:k,dim] <- b/s;J[dim,dim] <- -1}
  v <- J%*%vcov(fit)%*%t(J)
  params <- c(-b/s,if(family=='weibull') -log(s) else NULL)
  values <- c(params,sqrt(diag(v)),as.vector(t(v)),fit$loglik[2])
  cat('R,',family,',',paste(sprintf('%.17g',values),collapse=','),'\n',sep='')
}
