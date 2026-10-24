# Development reference: preferred utility normalization with ONE shared scale.
# A separately labelled extension of MMA15.6.3's source design, not its Stata8 fit.
# Tested R4.6.1/mlogit2.0.0/survival3.8.6; no runtime dependency.
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==2)
library(mlogit)
library(survival)
stopifnot(packageVersion('mlogit')=='2.0.0')
options(digits=17)
w <- read.csv(args[1])
stopifnot(nrow(w)==1182,identical(w$source_row,seq_len(1182)),all(is.finite(as.matrix(w))))
stopifnot(all(w$mode %in% 1:4),all(as.matrix(w[c('dbeach','dpier','dprivate','dcharter')])==outer(w$mode,1:4,'==')))
pnames <- c('pbeach','ppier','pprivate','pcharter')
qnames <- c('qbeach','qpier','qprivate','qcharter')
y <- w$income/1000
shore <- as.numeric(w$mode<=2)
px <- as.matrix(w[pnames])[cbind(seq_len(1182),w$mode)]
qx <- as.matrix(w[qnames])[cbind(seq_len(1182),w$mode)]
for(j in 1:4) {
  assign(paste0('p',j),w[[pnames[j]]])
  assign(paste0('q',j),w[[qnames[j]]])
}
d <- do.call(rbind,lapply(seq_len(1182),function(i) {
  data.frame(id=i,alternative=1:4,nest=c(1,1,2,2),chosen=as.numeric(w$mode[i]==1:4),
    p=as.numeric(w[i,pnames]),q=as.numeric(w[i,qnames]),ydiv1000=y[i],
    dshore=c(1,1,0,0),dshorey=c(1,1,0,0)*y[i])
}))
f <- dfidx(d,idx=c('id','alternative'),choice='chosen')
package_fit <- mlogit(chosen~p+q+dshore+dshorey|0,data=f,
  nests=list(shore=c('1','2'),boat=c('3','4')),unscaled=FALSE,un.nest.el=TRUE,
  method='bfgs',tol=1e-11,ftol=1e-14,steptol=1e-13,iterlim=2000)
initial_par <- unname(coef(package_fit))
stopifnot(identical(names(coef(package_fit)),c('p','q','dshore','dshorey','iv')),initial_par[5]>0)
# Under preferred normalization the package's nest-constant features enter the
# top-level utility unchanged: no lambda multiplier as in the earlier variant.
expr <- expression((bp*px+bq*qx)/r+shore*(a+b*y)+
  (r-1)*(shore*log(exp((bp*p1+bq*q1)/r)+exp((bp*p2+bq*q2)/r))+
    (1-shore)*log(exp((bp*p3+bq*q3)/r)+exp((bp*p4+bq*q4)/r)))-
  log(exp(a+b*y)*(exp((bp*p1+bq*q1)/r)+exp((bp*p2+bq*q2)/r))^r+
    (exp((bp*p3+bq*q3)/r)+exp((bp*p4+bq*q4)/r))^r))
fn <- deriv3(expr,c('bp','bq','a','b','r'),function.arg=TRUE)
X <- lapply(1:4,function(j)as.matrix(d[d$alternative==j,c('p','q','dshore','dshorey')]))
Y <- lapply(1:4,function(j)as.logical(d$chosen[d$alternative==j]))
names(X) <- names(Y) <- as.character(1:4)
package_value <- getFromNamespace('lnl.nlogit','mlogit')(coef(package_fit),X,Y,weights=rep(1,1182),
  gradient=TRUE,opposite=1,nests=list(shore=c('1','2'),boat=c('3','4')),unscaled=FALSE,un.nest.el=TRUE)
initial <- do.call(fn,as.list(initial_par))
stopifnot(abs(sum(initial)-as.numeric(logLik(package_fit)))<1e-9,
  abs(sum(initial)-as.numeric(package_value))<1e-9,
  max(abs(colSums(attr(initial,'gradient'))-attr(package_value,'gradient')))<1e-8)
par <- initial_par
D <- diag(c(.01,1,1,1,1))
converged <- FALSE
for(iteration in 1:15) {
  value <- do.call(fn,as.list(par))
  g <- colSums(attr(value,'gradient'));H <- apply(attr(value,'hessian'),c(2,3),sum)
  step <- drop(D%*%solve(-D%*%H%*%D,D%*%g))
  if(max(abs(solve(D,step)))<1e-11 && max(abs(D%*%g))<1e-9) {converged<-TRUE;break}
  next_par <- par+step
  stopifnot(next_par[5]>0,sum(do.call(fn,as.list(next_par)))>=sum(value)-1e-9)
  par <- next_par
}
stopifnot(converged,min(eigen(-D%*%H%*%D,symmetric=TRUE,only.values=TRUE)$values)>0)
V <- D%*%solve(-D%*%H%*%D)%*%D
ll <- sum(value)
restricted <- survival::clogit(chosen~p+q+dshore+dshorey+strata(id),data=d,method='exact',
  control=survival::coxph.control(eps=1e-12,toler.chol=1e-14,iter.max=100))
restricted_par <- unname(coef(restricted));restricted_D <- diag(c(.01,1,1,1))
restricted_value <- do.call(fn,as.list(c(restricted_par,1)))
restricted_H <- apply(attr(restricted_value,'hessian'),c(2,3),sum)[1:4,1:4]
restricted_step <- restricted_D%*%solve(-restricted_D%*%restricted_H%*%restricted_D,
  restricted_D%*%colSums(attr(restricted_value,'gradient'))[1:4])
stopifnot(max(abs(solve(restricted_D,restricted_step)))<1e-12,
  abs(sum(restricted_value)-as.numeric(logLik(restricted)))<1e-9)
restricted_V <- restricted_D%*%solve(-restricted_D%*%restricted_H%*%restricted_D)%*%restricted_D
restricted_ll <- sum(restricted_value)
probability <- function(data,par) {
  r <- par[5]
  scaled <- matrix((par[1]*data$p+par[2]*data$q)/r,nrow=4)
  center_s <- apply(scaled[1:2,,drop=FALSE],2,max)
  center_b <- apply(scaled[3:4,,drop=FALSE],2,max)
  es <- exp(sweep(scaled[1:2,,drop=FALSE],2,center_s))
  eb <- exp(sweep(scaled[3:4,,drop=FALSE],2,center_b))
  S <- colSums(es);B <- colSums(eb)
  A <- par[3]+par[4]*data$ydiv1000[seq(1,nrow(data),4)]+r*(center_s+log(S))
  C <- r*(center_b+log(B))
  ps <- plogis(A-C)
  as.vector(rbind(sweep(es,2,S,'/'),sweep(eb,2,B,'/'))*rbind(ps,ps,1-ps,1-ps))
}
stopifnot(max(abs(probability(d,initial_par)-as.vector(t(attr(package_value,'probabilities')))))<1e-12)
stopifnot(max(abs(probability(d,c(restricted_par,1))-predict(restricted,type='expected')))<1e-12)
quantities <- list()
emit <- function(model,quantity,i,j,value) {
  quantities[[length(quantities)+1L]] <<- data.frame(model=model,quantity=quantity,i=i,j=j,value=value)
}
for(model in c('restricted','normalized_common')) {
  beta <- if(model=='normalized_common')par else restricted_par
  covariance <- if(model=='normalized_common')V else restricted_V
  fit_ll <- if(model=='normalized_common')ll else restricted_ll
  for(name in c('observation_count','choice_set_count','parameter_count','log_likelihood'))
    emit(model,name,0,0,switch(name,observation_count=4728,choice_set_count=1182,parameter_count=length(beta),log_likelihood=fit_ll))
  for(i in seq_along(beta)) {
    se <- sqrt(covariance[i,i]);null <- if(i<=4)0 else 1
    z <- (beta[i]-null)/se
    for(name in c('value','standard_error','test_null','statistic','p_value','confidence_lower','confidence_upper'))
      emit(model,paste0('parameter_',name),i,0,switch(name,value=beta[i],standard_error=se,test_null=null,
        statistic=z,p_value=2*pnorm(-abs(z)),confidence_lower=beta[i]-qnorm(.975)*se,confidence_upper=beta[i]+qnorm(.975)*se))
    for(j in seq_along(beta))emit(model,'covariance',i,j,covariance[i,j])
  }
}
lr <- 2*(ll-restricted_ll)
emit('normalized_common','independence_likelihood_ratio',0,0,lr)
emit('normalized_common','independence_df',0,0,1)
emit('normalized_common','independence_p_value',0,0,pchisq(lr,1,lower.tail=FALSE))
expected <- d
expected$restricted <- probability(d,c(restricted_par,1))
expected$baseline <- probability(d,par)
income <- d;income$ydiv1000 <- income$ydiv1000+1
expected$income <- probability(income,par)
price <- d;price$p[price$alternative==4] <- price$p[price$alternative==4]+1
expected$price <- probability(price,par)
catch <- d;catch$q[catch$alternative==4] <- catch$q[catch$alternative==4]+.001
expected$catch <- probability(catch,par)
for(alternative in 1:4) {
  rows <- expected$alternative==alternative
  for(name in c('mean_probability','income_contrast','price_contrast','catch_contrast')) {
    contrast <- switch(name,mean_probability=expected$baseline,income_contrast=expected$income-expected$baseline,
      price_contrast=100*(expected$price-expected$baseline),catch_contrast=1000*(expected$catch-expected$baseline))
    emit('normalized_common',name,alternative,0,mean(contrast[rows]))
  }
}
stopifnot(par[5]>1) # This source fit diagnoses a RUM-bound violation, not consistency.
for(name in names(expected))expected[[name]] <- sprintf('%.17g',expected[[name]])
write.csv(expected,args[2],row.names=FALSE,quote=FALSE)
write.csv(do.call(rbind,quantities),stdout(),row.names=FALSE,quote=FALSE)
