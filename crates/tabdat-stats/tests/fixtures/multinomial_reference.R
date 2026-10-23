# Development-only R4.6.1 / nnet7.3.20 reference for exact three-class cell odds.
# Rscript multinomial_reference.R > multinomial_reference.csv
options(digits=17)
counts <- rbind(c(1,1,6),c(2,4,6),c(4,16,6))
x <- y <- numeric()
for(g in 1:3) for(category in 1:3) {
  x <- c(x,rep(g-2,counts[g,category])); y <- c(y,rep(category,counts[g,category]))
}
d <- data.frame(y=factor(y),x=x); X <- cbind(1,x); n <- nrow(X)
fit <- nnet::multinom(y~x,d,trace=FALSE,Hess=TRUE,maxit=1000,reltol=1e-14,abstol=1e-14)
theta <- c(log(2),log(2),log(3),-log(2))
stopifnot(fit$convergence==0L,max(abs(drop(t(coef(fit)))-theta))<1e-6)
probabilities <- function(theta) {
  eta <- cbind(0,X%*%t(matrix(theta,nrow=2,byrow=TRUE)))
  weights <- exp(eta-apply(eta,1,max)); weights/rowSums(weights)
}
pr <- probabilities(theta)
stopifnot(max(abs(crossprod(X,model.matrix(~d$y-1)[,-1]-pr[,-1])))<1e-12)
# Evaluate nnet's independently implemented alternative-summed Fisher curvature
# at the exact cell-odds MLE, not its finite-tolerance optimizer's cached Hessian.
fit$fitted.values <- pr
V <- solve(nnet:::multinomHess(fit,Z=X))
ll <- sum(log(pr[cbind(seq_along(y),y)])); shares <- colSums(counts)/n
null <- sum(log(shares[y])); lr <- 2*(ll-null)
rows <- list()
emit <- function(quantity,i,j,value) {
  rows[[length(rows)+1L]] <<- data.frame(quantity=quantity,i=i,j=j,value=value)
}
emit("log_likelihood",0,0,ll); emit("null_log_likelihood",0,0,null)
emit("likelihood_ratio",0,0,lr); emit("likelihood_ratio_p_value",0,0,pchisq(lr,2,lower.tail=FALSE))
emit("mcfadden_r_squared",0,0,1-ll/null)
normal <- function(prefix,i,value,se) {
  emit(paste0(prefix,"value"),i,0,value); emit(paste0(prefix,"standard_error"),i,0,se)
  emit(paste0(prefix,"statistic"),i,0,value/se)
  emit(paste0(prefix,"p_value"),i,0,2*pnorm(-abs(value/se)))
  emit(paste0(prefix,"confidence_lower"),i,0,value-qnorm(.975)*se)
  emit(paste0(prefix,"confidence_upper"),i,0,value+qnorm(.975)*se)
}
for(i in seq_along(theta)) {
  normal("coefficient_",i-1,theta[i],sqrt(V[i,i]))
  for(j in seq_along(theta)) emit("covariance",i-1,j-1,V[i,j])
}
ame <- function(theta) {
  p <- probabilities(theta); slopes <- c(0,theta[c(2,4)])
  colMeans(p*(matrix(slopes,nrow=n,ncol=3,byrow=TRUE)-drop(p%*%slopes)))
}
# Independent numerical derivatives, Richardson-extrapolated central differences.
gradient <- matrix(0,3,4)
for(i in 1:4) {
  difference <- function(h) {
    plus <- minus <- theta; plus[i]<-plus[i]+h; minus[i]<-minus[i]-h
    (ame(plus)-ame(minus))/(2*h)
  }
  h <- 1e-4*(1+abs(theta[i]))
  gradient[,i] <- (4*difference(h/2)-difference(h))/3
}
values <- ame(theta)
for(i in 1:3) normal("effect_",i-1,values[i],sqrt(drop(gradient[i,]%*%V%*%gradient[i,])))
write.csv(do.call(rbind,rows),stdout(),row.names=FALSE,quote=FALSE)
