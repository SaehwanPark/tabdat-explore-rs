# Development-only reference for MMA16P1TOBIT §2A; no generated replacement sample.
# R4.6.1 / survival3.8.6; full joint observed information in beta/lnsigma coordinates.
suppressPackageStartupMessages(library(survival))
args <- commandArgs(trailingOnly=TRUE)
if(length(args)!=2)stop('usage: reference_tobit.R original.asc predictions.csv')
d <- read.table(args[1],na.strings='.')
names(d) <- c('e','lnwage','ystar','ytrunc','ycens','dy')
stopifnot(nrow(d)==200,ncol(d)==6,all(is.finite(as.matrix(d[,c(1:3,5:6)]))),
  all(d$ycens==pmax(d$ystar,0)),all(d$dy==as.numeric(d$ycens>0)),
  sum(d$dy==0)==70,sum(d$dy==1)==130,
  all(is.na(d$ytrunc)==(d$dy==0)),all(d$ytrunc[d$dy==1]==d$ystar[d$dy==1]))
if(file.exists(args[2]))stop('predictions output already exists')
x <- d$lnwage;y <- d$ycens;event <- d$dy==1;n <- nrow(d)
f <- survreg(Surv(ycens,dy,type='left')~lnwage,data=d,dist='gaussian',
  control=survreg.control(maxiter=100,rel.tolerance=1e-12))
f0 <- survreg(Surv(ycens,dy,type='left')~1,data=d,dist='gaussian',
  control=survreg.control(maxiter=100,rel.tolerance=1e-12))
# Independent symbolic derivatives of the actual censored likelihood, not OPG
# or a conditional-on-sigma covariance extracted from a Gaussian regression.
parameters <- c('b0','b1','eta')
arguments <- c(parameters,'x','y')
censored <- deriv3(~log(pnorm(-(b0+b1*x)/exp(eta))),parameters,function.arg=arguments)
uncensored <- deriv3(~-eta-.5*((y-b0-b1*x)/exp(eta))^2-.5*log(2*pi),parameters,function.arg=arguments)
evaluate <- function(theta) {
  cpart <- censored(theta[1],theta[2],theta[3],x,y)
  upart <- uncensored(theta[1],theta[2],theta[3],x,y)
  gradient <- colSums(attr(cpart,'gradient')[!event,,drop=FALSE])+colSums(attr(upart,'gradient')[event,,drop=FALSE])
  information <- -apply(attr(cpart,'hessian')[!event,,,drop=FALSE],c(2,3),sum)-
    apply(attr(upart,'hessian')[event,,,drop=FALSE],c(2,3),sum)
  list(ll=sum(cpart[!event])+sum(upart[event]),score=gradient,information=information)
}
refine <- function(theta,free) {
  for(iteration in 1:20) {
    e <- evaluate(theta);information <- e$information[free,free,drop=FALSE]
    chol(information)
    step <- solve(information,e$score[free])
    score_bound <- max(abs(e$score[free])*pmax(1,abs(theta[free])))/n
    step_bound <- max(abs(step)/pmax(1,abs(theta[free])))
    if(score_bound<1e-11 && step_bound<1e-10)
      return(list(theta=theta,e=e,covariance=solve(information),score_bound=score_bound,step_bound=step_bound))
    candidate <- theta;candidate[free] <- theta[free]+step
    stopifnot(evaluate(candidate)$ll>=e$ll-32*.Machine$double.eps*(1+abs(e$ll)))
    theta <- candidate
  }
  stop('independent observed-Newton refinement did not converge')
}
fit <- refine(c(unname(coef(f)),log(f$scale)),1:3)
null <- refine(c(unname(coef(f0)),0,log(f0$scale)),c(1,3))
stopifnot(abs(fit$e$ll-f$loglik[2])<1e-8,abs(null$e$ll-f0$loglik[2])<1e-8,
  max(abs(fit$covariance-vcov(f))/(1+abs(vcov(f))))<1e-7)
beta <- fit$theta[1:2];eta <- fit$theta[3];sigma <- exp(eta);v <- fit$covariance
latent <- beta[1]+beta[2]*x;z <- latent/sigma;p <- pnorm(z);density <- dnorm(z)
observed_mean <- latent*p+sigma*density
# Conditional-X AME of E[max(0,Y*)|X], not the latent slope or truncated mean.
mean_p <- mean(p)
mean_gradient <- c(mean(density/sigma),mean(density*x/sigma),mean(-z*density))
ame <- beta[2]*mean_p;ame_gradient <- beta[2]*mean_gradient
ame_gradient[2] <- ame_gradient[2]+mean_p
ame_variance <- drop(t(ame_gradient)%*%v%*%ame_gradient)
stopifnot(all(is.finite(c(latent,z,p,observed_mean,mean_gradient,ame_variance))),ame_variance>0)
rows <- list()
emit <- function(quantity,value,i=0,j=0)rows[[length(rows)+1L]] <<- data.frame(quantity=quantity,i=i,j=j,value=as.numeric(value))
inference <- function(prefix,value,se,i) {
  statistic <- value/se;tail <- 2*pnorm(abs(statistic),lower.tail=FALSE)
  stopifnot(is.finite(statistic),tail>0,is.finite(se),se>0)
  for(name in c('value','standard_error','statistic','p_value','confidence_lower','confidence_upper')) {
    number <- switch(name,value=value,standard_error=se,statistic=statistic,p_value=tail,
      confidence_lower=value-qnorm(.975)*se,confidence_upper=value+qnorm(.975)*se)
    emit(paste0(prefix,'_',name),number,i)
  }
}
for(i in 1:2)inference('coefficient',beta[i],sqrt(v[i,i]),i)
for(i in 1:3)for(j in 1:3)emit('joint_covariance',v[i,j],i,j)
for(i in 1:3)emit('mean_probability_gradient',mean_gradient[i],i)
inference('ame',ame,sqrt(ame_variance),1)
lr <- 2*(fit$e$ll-null$e$ll)
summary <- c(n=n,censored_count=sum(!event),uncensored_count=sum(event),lower_limit=0,
  log_likelihood=fit$e$ll,null_log_likelihood=null$e$ll,likelihood_ratio=lr,
  likelihood_ratio_df=1,likelihood_ratio_p_value=pchisq(lr,1,lower.tail=FALSE),
  sigma=sigma,log_sigma=eta,log_sigma_standard_error=sqrt(v[3,3]),sigma_standard_error=sigma*sqrt(v[3,3]),
  sigma_confidence_lower=exp(eta-qnorm(.975)*sqrt(v[3,3])),sigma_confidence_upper=exp(eta+qnorm(.975)*sqrt(v[3,3])),
  mean_uncensored_probability=mean_p,scaled_score=fit$score_bound,scaled_step=fit$step_bound)
for(name in names(summary))emit(name,summary[name])
output <- data.frame(source_row=1:n,d,latent=latent,uncensored_probability=p,observed_mean=observed_mean)
# Validate completely before publishing either table. Never overwrite predictions.
write.csv(output,args[2],row.names=FALSE,na='')
write.csv(do.call(rbind,rows),stdout(),row.names=FALSE,quote=FALSE)
cat('Reference:',R.version.string,'survival',as.character(packageVersion('survival')),'\n',file=stderr())
