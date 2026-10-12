library(quantreg)
d <- read.csv(commandArgs(trailingOnly=TRUE)[1]);X <- model.matrix(~x1+x2,d);y <- d$y;n <- length(y)
for (q in c(.25,.5,.75)) {
  fit <- rq(y ~ x1+x2,data=d,tau=q,method='br');b <- coef(fit);e <- y-X%*%b
  z <- qnorm(q);h0 <- n^(-1/3)*qnorm(.975)^(2/3)*(1.5*dnorm(z)^2/(2*z^2+1))^(1/3)
  h <- min(sqrt(mean((y-mean(y))^2)),diff(quantile(e,c(.25,.75),type=7))/1.34)*(qnorm(q+h0)-qnorm(q-h0))
  u <- e/h;density <- sum(ifelse(abs(u)<=1,.75*(1-u^2),0))/(n*h)
  inverse <- solve(crossprod(X));v <- q*(1-q)/density^2*inverse
  rho <- function(u) ifelse(u>=0,q*u,(q-1)*u)
  prsquared <- 1-sum(rho(e))/sum(rho(y-quantile(y,q,type=7)))
  values <- c(b,sqrt(diag(v)),as.vector(t(v)),h,density,prsquared)
  cat('R,',q,',',paste(sprintf('%.17g',values),collapse=','),'\n',sep='')
  # Native sparsity differs from the canonical density method: preserve R's iid
  # covariance separately for investigation, not as silently equivalent output.
  iid <- summary(fit,se='iid',covariance=TRUE)$cov
  cat('R-native-iid,',q,',',paste(sprintf('%.17g',as.vector(t(iid))),collapse=','),'\n',sep='')
}
