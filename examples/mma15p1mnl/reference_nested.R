# Development reference for MMA §15.6.3 p511's earlier non-normalized model.
# Validated with R4.6.1,mlogit2.0.0 and survival3.8.6; never a runtime dependency.
# Args: prepared Nldata.csv, output R_nested_predictions.csv; stdout quantities CSV.
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
  nests=list(shore=c('1','2'),boat=c('3','4')),unscaled=TRUE,
  method='bfgs',tol=1e-11,ftol=1e-15,steptol=1e-13,iterlim=2000)
v <- coef(package_fit)
stopifnot(identical(names(v),c('p','q','dshore','dshorey','iv:shore','iv:boat')),all(v[5:6]>0))
# mlogit puts every regressor inside conditional utility; these upper features
# are nest-constant, so top-level coefficients are lambda_shore times its values.
par <- unname(c(v[1:2],v[3:4]*v[5],v[5:6]))
initial_par <- par
# Base R automatically differentiates the independent scalar likelihood twice.
# This avoids mlogit2.0.0's default OPG covariance and BFGS function-value-only
# stopping; neither is substituted for observed information or tight convergence.
expr <- expression(bp*px+bq*qx+shore*(a+b*y)+
  shore*(rs-1)*log(exp(bp*p1+bq*q1)+exp(bp*p2+bq*q2))+
  (1-shore)*(rb-1)*log(exp(bp*p3+bq*q3)+exp(bp*p4+bq*q4))-
  log(exp(a+b*y)*(exp(bp*p1+bq*q1)+exp(bp*p2+bq*q2))^rs+
    (exp(bp*p3+bq*q3)+exp(bp*p4+bq*q4))^rb))
fn <- deriv3(expr,c('bp','bq','a','b','rs','rb'),function.arg=TRUE)
initial <- do.call(fn,as.list(par))
J <- diag(6)
J[3,3] <- J[4,4] <- 1/par[5]
J[3,5] <- -par[3]/par[5]^2
J[4,5] <- -par[4]/par[5]^2
# Reevaluate the package score at reported parameters rather than using its
# optimizer cache. This version-pinned development check uses its internal API.
X <- lapply(1:4,function(j)as.matrix(d[d$alternative==j,c('p','q','dshore','dshorey')]))
Y <- lapply(1:4,function(j)as.logical(d$chosen[d$alternative==j]))
names(X) <- names(Y) <- as.character(1:4)
package_value <- getFromNamespace('lnl.nlogit','mlogit')(v,X,Y,weights=rep(1,1182),
  gradient=TRUE,opposite=1,nests=list(shore=c('1','2'),boat=c('3','4')),unscaled=TRUE)
stopifnot(abs(sum(initial)-as.numeric(logLik(package_fit)))<1e-9,
  abs(sum(initial)-as.numeric(package_value))<1e-9,
  max(abs(colSums(attr(initial,'gradient'))-drop(t(J)%*%attr(package_value,'gradient'))))<1e-8)
# Fixed numerical coordinates only; no model restriction or covariance correction.
D <- diag(c(.01,1,1,1,30,15))
converged <- FALSE
for(iteration in 1:15) {
  value <- do.call(fn,as.list(par))
  g <- colSums(attr(value,'gradient'))
  H <- apply(attr(value,'hessian'),c(2,3),sum)
  step <- drop(D%*%solve(-D%*%H%*%D,D%*%g))
  if(max(abs(solve(D,step)))<1e-10 && max(abs(D%*%g))<1e-9) {converged<-TRUE;break}
  next_par <- par+step
  stopifnot(all(next_par[5:6]>0),sum(do.call(fn,as.list(next_par)))>=sum(value)-1e-9)
  par <- next_par
}
stopifnot(converged,min(eigen(-D%*%H%*%D,symmetric=TRUE,only.values=TRUE)$values)>0)
V <- D%*%solve(-D%*%H%*%D)%*%D
ll <- sum(value)
restricted <- survival::clogit(chosen~p+q+dshore+dshorey+strata(id),data=d,method='exact',
  control=survival::coxph.control(eps=1e-12,toler.chol=1e-14,iter.max=100))
restricted_ll <- as.numeric(logLik(restricted))
probability <- function(data,par) {
  utility <- matrix(par[1]*data$p+par[2]*data$q,nrow=4)
  center_s <- apply(utility[1:2,,drop=FALSE],2,max)
  center_b <- apply(utility[3:4,,drop=FALSE],2,max)
  S <- colSums(exp(sweep(utility[1:2,,drop=FALSE],2,center_s)))
  B <- colSums(exp(sweep(utility[3:4,,drop=FALSE],2,center_b)))
  A <- par[3]+par[4]*data$ydiv1000[seq(1,nrow(data),4)]+par[5]*(center_s+log(S))
  C <- par[6]*(center_b+log(B))
  ps <- plogis(A-C)
  within <- rbind(sweep(exp(sweep(utility[1:2,,drop=FALSE],2,center_s)),2,S,'/'),
    sweep(exp(sweep(utility[3:4,,drop=FALSE],2,center_b)),2,B,'/'))
  as.vector(within*rbind(ps,ps,1-ps,1-ps))
}
stopifnot(max(abs(probability(d,initial_par)-as.vector(t(attr(package_value,'probabilities')))))<1e-12)
quantities <- list()
emit <- function(model,quantity,i,j,value) {
  quantities[[length(quantities)+1L]] <<- data.frame(model=model,quantity=quantity,i=i,j=j,value=value)
}
for(model in c('restricted','nested')) {
  beta <- if(model=='nested')par else coef(restricted)
  covariance <- if(model=='nested')V else vcov(restricted)
  fit_ll <- if(model=='nested')ll else restricted_ll
  for(name in c('observation_count','choice_set_count','parameter_count','log_likelihood')) {
    emit(model,name,0,0,switch(name,observation_count=4728,choice_set_count=1182,
      parameter_count=length(beta),log_likelihood=fit_ll))
  }
  for(i in seq_along(beta)) {
    se <- sqrt(covariance[i,i]);null <- if(i<=4)0 else 1
    z <- (beta[i]-null)/se
    for(name in c('value','standard_error','test_null','statistic','p_value','confidence_lower','confidence_upper')) {
      emit(model,paste0('parameter_',name),i,0,switch(name,value=beta[i],standard_error=se,
        test_null=null,statistic=z,p_value=2*pnorm(-abs(z)),
        confidence_lower=beta[i]-qnorm(.975)*se,confidence_upper=beta[i]+qnorm(.975)*se))
    }
    for(j in seq_along(beta))emit(model,'covariance',i,j,covariance[i,j])
  }
}
lr <- 2*(ll-restricted_ll)
emit('nested','independence_likelihood_ratio',0,0,lr)
emit('nested','independence_df',0,0,2)
emit('nested','independence_p_value',0,0,pchisq(lr,2,lower.tail=FALSE))
expected <- d
expected$restricted <- probability(d,c(coef(restricted),1,1))
stopifnot(max(abs(expected$restricted-predict(restricted,type='expected')))<1e-12)
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
    contrast <- switch(name,mean_probability=expected$baseline,
      income_contrast=expected$income-expected$baseline,
      price_contrast=100*(expected$price-expected$baseline),
      catch_contrast=1000*(expected$catch-expected$baseline))
    emit('nested',name,alternative,0,mean(contrast[rows]))
  }
}
# Exact source/design checks require round-trip floats, not write.csv's digits15.
for(name in names(expected))expected[[name]] <- sprintf('%.17g',expected[[name]])
write.csv(expected,args[2],row.names=FALSE,quote=FALSE)
write.csv(do.call(rbind,quantities),stdout(),row.names=FALSE,quote=FALSE)
