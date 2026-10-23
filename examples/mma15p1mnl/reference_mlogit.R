# Development-only R4.6.1 / nnet7.3.20 reference, not a runtime dependency.
# Rscript reference_mlogit.R Nldata.csv R_predictions.csv > R_quantities.csv
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==2L)
d <- read.csv(args[1])
stopifnot(nrow(d)==1182L,identical(d$source_row,seq_len(1182L)),
  all(is.finite(as.matrix(d))),identical(as.integer(table(factor(d$mode,levels=1:4))),c(134L,178L,418L,452L)))
d$ydiv1000 <- d$income/1000
X <- cbind(1,d$ydiv1000); n <- nrow(X); k <- ncol(X); J <- 4L
fit <- nnet::multinom(factor(mode,levels=1:4)~ydiv1000,d,trace=FALSE,
  Hess=TRUE,maxit=1000,abstol=1e-14,reltol=1e-14)
stopifnot(fit$convergence==0L)
theta <- as.vector(t(coef(fit))); initial <- theta
probabilities <- function(theta,design=X) {
  eta <- cbind(0,design%*%t(matrix(theta,nrow=J-1,byrow=TRUE)))
  weights <- exp(eta-apply(eta,1,max)); weights/rowSums(weights)
}
observed <- model.matrix(~factor(d$mode,levels=1:4)-1)
# nnet optimizes to a likelihood tolerance. Polish the score fixed point using
# nnet's independently implemented alternative-summed Fisher curvature, not
# native coefficients or a cached finite-tolerance Hessian.
for(iteration in 1:20) {
  pr <- probabilities(theta); fit$fitted.values <- pr
  information <- nnet:::multinomHess(fit,Z=X)
  score <- as.vector(crossprod(X,observed[,-1]-pr[,-1]))
  step <- solve(information,score)
  if(max(abs(score))<1e-10 && max(abs(step))<1e-10) break
  theta <- theta+step
}
stopifnot(max(abs(score))<1e-10,max(abs(step))<1e-10,max(abs(theta-initial))<1e-5)
V <- solve(information)
pr <- probabilities(theta)
ll <- sum(log(pr[cbind(seq_len(n),d$mode)]))
counts <- tabulate(d$mode,nbins=4); null <- sum(log((counts/n)[d$mode]))
lr <- 2*(ll-null)
rows <- list()
emit <- function(quantity,i,j,value) {
  rows[[length(rows)+1L]] <<- data.frame(quantity=quantity,i=i,j=j,value=value)
}
for(name in c("observation_count","log_likelihood","null_log_likelihood",
  "likelihood_ratio","likelihood_ratio_df","likelihood_ratio_p_value","mcfadden_r_squared")) {
  value <- switch(name,observation_count=n,log_likelihood=ll,null_log_likelihood=null,
    likelihood_ratio=lr,likelihood_ratio_df=3,likelihood_ratio_p_value=pchisq(lr,3,lower.tail=FALSE),
    mcfadden_r_squared=1-ll/null)
  emit(name,0,0,value)
}
for(i in 1:4) emit("category_count",i-1,0,counts[i])
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
  p <- probabilities(theta); slopes <- c(0,theta[seq(2,length(theta),by=2)])
  colMeans(p*(matrix(slopes,nrow=n,ncol=J,byrow=TRUE)-drop(p%*%slopes)))
}
gradient <- matrix(0,J,length(theta))
for(i in seq_along(theta)) {
  difference <- function(h) {
    plus <- minus <- theta; plus[i]<-plus[i]+h; minus[i]<-minus[i]-h
    (ame(plus)-ame(minus))/(2*h)
  }
  h <- 1e-4*(1+abs(theta[i]))
  gradient[,i] <- (4*difference(h/2)-difference(h))/3
  stopifnot(max(abs(gradient[,i]-difference(h/2)))<1e-8)
}
values <- ame(theta)
for(i in seq_len(J)) normal("effect_",i-1,values[i],sqrt(drop(gradient[i,]%*%V%*%gradient[i,])))
expected <- d
indices <- cbind(0,X%*%t(matrix(theta,nrow=J-1,byrow=TRUE)))
for(i in seq_len(J)) {
  expected[[paste0("p",i)]] <- pr[,i]
  expected[[paste0("xb",i)]] <- indices[,i]
}
write.csv(expected,args[2],row.names=FALSE)
write.csv(do.call(rbind,rows),stdout(),row.names=FALSE,quote=FALSE)
