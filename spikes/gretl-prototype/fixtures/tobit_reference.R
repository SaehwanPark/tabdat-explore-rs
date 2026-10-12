library(survival)
args <- commandArgs(trailingOnly=TRUE)
d <- read.csv(args[1])
for (family in c('left','both')) {
  y <- if(family=='left') d$y_left else d$y_both
  lo <- ifelse(y<=0,-Inf,y)
  hi <- ifelse(y<=0,0,y)
  if(family=='both') {lo[y>=2] <- 2;hi[y>=2] <- Inf}
  fit <- survreg(Surv(lo,hi,type='interval2') ~ x1+x2,data=d,dist='gaussian',control=survreg.control(rel.tolerance=1e-12,maxiter=100))
  stopifnot(nobs(fit)==240)
  v <- vcov(fit)
  values <- c(coef(fit),sqrt(diag(v))[1:3],as.vector(t(v)),log(fit$scale),fit$loglik[2])
  cat('R,',family,',',paste(sprintf('%.17g',values),collapse=','),'\n',sep='')
}
