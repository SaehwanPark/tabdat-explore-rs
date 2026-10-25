# Development-only source extension: the author COMMENTS on a truncated MLE,
# but executes no such estimator. This is not a historical/table coefficient replay.
# R4.6.1/truncreg0.2-5/maxLik1.5-2.2; no production R dependency.
args <- commandArgs(trailingOnly=TRUE)
if(length(args)!=3)stop('usage: reference_truncreg.R original.asc predictions.csv pinned_truncreg_0.2-5.tar.gz')
if(file.exists(args[2]))stop('predictions output already exists')
stopifnot(unname(tools::md5sum(args[3]))=='6ddc73494a349da69b23f859ffe89a54')
suppressPackageStartupMessages(library(truncreg))
stopifnot(packageDescription('truncreg')$Version=='0.2-5',packageDescription('maxLik')$Version=='1.5-2.2')
# Bind the installed reference routines to the pinned external CRAN source,
# not merely a mutable package version label. No GPL source is vendored here.
tmp <- tempfile('truncreg-reference-');dir.create(tmp)
utils::untar(args[3],files='truncreg/R/truncreg.R',exdir=tmp)
source <- new.env(parent=baseenv());sys.source(file.path(tmp,'truncreg/R/truncreg.R'),envir=source,keep.source=FALSE)
for(name in c('truncreg','truncreg.fit','ml.truncreg'))stopifnot(isTRUE(all.equal(body(get(name,asNamespace('truncreg'))),body(get(name,source)),check.attributes=FALSE)))
unlink(tmp,recursive=TRUE)
d <- read.table(args[1],na.strings='.');names(d) <- c('e','lnwage','ystar','ytrunc','ycens','dy')
stopifnot(nrow(d)==200,ncol(d)==6,all(is.finite(as.matrix(d[,c(1:3,5:6)]))),
 all(d$ycens==pmax(d$ystar,0)),all(d$dy==as.numeric(d$ycens>0)),
 sum(d$dy==0)==70,sum(d$dy==1)==130,all(is.na(d$ytrunc)==(d$dy==0)),
 all(d$ytrunc[d$dy==1]==d$ystar[d$dy==1]),all(d$ytrunc[d$dy==1]>0))
selected <- is.finite(d$ytrunc)&is.finite(d$lnwage);n <- sum(selected)
# Fixed unit rescaling for conditioning; raw LL includes the exact Jacobian.
unit <- 1000;x <- d$lnwage[selected];y <- d$ytrunc[selected]/unit
model <- data.frame(y=y,x=x)
f <- truncreg(y~x,data=model,point=0,direction='left',scaled=FALSE,method='NR',iterlim=1000)
f0 <- truncreg(y~1,data=model,point=0,direction='left',scaled=FALSE,method='NR',iterlim=1000)
parameters <- c('b0','b1','eta')
likelihood <- deriv3(~-eta-.5*((y-b0-b1*x)/exp(eta))^2-.5*log(2*pi)-log(pnorm((b0+b1*x)/exp(eta))),parameters,function.arg=c(parameters,'x','y'))
evaluate <- function(theta) {
 part <- likelihood(theta[1],theta[2],theta[3],x,y)
 list(ll=sum(part),score=colSums(attr(part,'gradient')),information=-apply(attr(part,'hessian'),c(2,3),sum))
}
refine <- function(theta,free) {
 for(iteration in 1:20) {
  e <- evaluate(theta);information <- e$information[free,free,drop=FALSE];invisible(chol(information))
  step <- solve(information,e$score[free])
  score_bound <- max(abs(e$score[free])*pmax(1,abs(theta[free])))/n
  step_bound <- max(abs(step)/pmax(1,abs(theta[free])))
  if(score_bound<1e-11&&step_bound<1e-10)return(list(theta=theta,e=e,covariance=solve(information),score_bound=score_bound,step_bound=step_bound))
  candidate <- theta;candidate[free] <- theta[free]+step
  stopifnot(evaluate(candidate)$ll>=e$ll-32*.Machine$double.eps*(1+abs(e$ll)))
  theta <- candidate
 }
 stop('independent symbolic observed-Newton refinement did not converge')
}
fit <- refine(c(unname(coef(f)[1:2]),log(unname(coef(f)[3]))),1:3)
null <- refine(c(unname(coef(f0)[1]),0,log(unname(coef(f0)[2]))),c(1,3))
package_transform <- diag(c(1,1,1/unname(coef(f)[3])))
stopifnot(abs(fit$e$ll-as.numeric(logLik(f)))<1e-8,abs(null$e$ll-as.numeric(logLik(f0)))<1e-8,
 max(abs(fit$covariance-package_transform%*%vcov(f)%*%package_transform)/(1+abs(fit$covariance)))<1e-7)
beta <- fit$theta[1:2]*unit;eta <- fit$theta[3]+log(unit);sigma <- exp(eta)
transform <- diag(c(unit,unit,1));v <- transform%*%fit$covariance%*%transform
latent <- beta[1]+beta[2]*d$lnwage;t <- latent/sigma;p <- pnorm(t)
mills <- exp(dnorm(t,log=TRUE)-pnorm(t,log.p=TRUE));conditional_mean <- sigma*(t+mills)
delta <- mills*(mills+t);h <- 1-delta;hprime <- delta*(2*mills+t)-mills
mean_h <- mean(h[selected])
mean_gradient <- c(mean(hprime[selected]/sigma),mean(hprime[selected]*x/sigma),mean(-t[selected]*hprime[selected]))
ame <- beta[2]*mean_h;ame_gradient <- beta[2]*mean_gradient;ame_gradient[2] <- ame_gradient[2]+mean_h
ame_variance <- drop(t(ame_gradient)%*%v%*%ame_gradient)
# Separate finite differences test the analytic FULL-joint AME gradient.
point <- c(beta,eta)
ame_at <- function(a) {s<-exp(a[3]);t<-(a[1]+a[2]*x)/s;m<-exp(dnorm(t,log=TRUE)-pnorm(t,log.p=TRUE));a[2]*mean(1-m*(m+t))}
for(relative_step in c(1e-5,3e-6,1e-6)) {
 numerical <- sapply(1:3,function(j){step<-relative_step*max(1,abs(point[j]));a<-b<-point;a[j]<-a[j]+step;b[j]<-b[j]-step;(ame_at(a)-ame_at(b))/(2*step)})
 stopifnot(max(abs(numerical-ame_gradient)/(1+abs(ame_gradient)))<1e-7)
}
stopifnot(all(is.finite(c(latent,t,p,mills,conditional_mean,mean_gradient,ame_variance))),all(conditional_mean>0),all(h>0),ame_variance>0)
rows <- list()
emit <- function(quantity,value,i=0,j=0)rows[[length(rows)+1L]] <<- data.frame(quantity=quantity,i=i,j=j,value=as.numeric(value))
inference <- function(prefix,value,se,i) {
 statistic <- value/se;tail <- 2*pnorm(abs(statistic),lower.tail=FALSE)
 stopifnot(is.finite(statistic),tail>0,is.finite(se),se>0)
 for(name in c('value','standard_error','statistic','p_value','confidence_lower','confidence_upper')) {
  number <- switch(name,value=value,standard_error=se,statistic=statistic,p_value=tail,confidence_lower=value-qnorm(.975)*se,confidence_upper=value+qnorm(.975)*se)
  emit(paste0(prefix,'_',name),number,i)
 }
}
for(i in 1:2)inference('coefficient',beta[i],sqrt(v[i,i]),i)
for(i in 1:3)for(j in 1:3)emit('joint_covariance',v[i,j],i,j)
for(i in 1:3)emit('mean_conditional_derivative_gradient',mean_gradient[i],i)
inference('ame',ame,sqrt(ame_variance),1)
lr <- 2*(fit$e$ll-null$e$ll)
summary <- c(n=n,dropped_count=sum(!selected),lower_limit=0,log_likelihood=fit$e$ll-n*log(unit),null_log_likelihood=null$e$ll-n*log(unit),
 likelihood_ratio=lr,likelihood_ratio_df=1,likelihood_ratio_p_value=pchisq(lr,1,lower.tail=FALSE),sigma=sigma,log_sigma=eta,
 log_sigma_standard_error=sqrt(v[3,3]),sigma_standard_error=sigma*sqrt(v[3,3]),sigma_confidence_lower=exp(eta-qnorm(.975)*sqrt(v[3,3])),
 sigma_confidence_upper=exp(eta+qnorm(.975)*sqrt(v[3,3])),mean_conditional_derivative=mean_h,
 scaled_score=fit$score_bound,scaled_step=fit$step_bound,null_scaled_score=null$score_bound,null_scaled_step=null$step_bound)
for(name in names(summary))emit(name,summary[name])
output <- data.frame(source_row=1:nrow(d),d,estimation_row=as.integer(selected),latent=latent,retention_probability=p,conditional_mean=conditional_mean)
write.csv(output,args[2],row.names=FALSE,na='');write.csv(do.call(rbind,rows),stdout(),row.names=FALSE,quote=FALSE)
cat('Reference:',R.version.string,'truncreg',packageDescription('truncreg')$Version,'maxLik',packageDescription('maxLik')$Version,
 '; pinned CRAN source/archive and installed bodies checked; symbolic observed refinement/full joint delta\n',file=stderr())
