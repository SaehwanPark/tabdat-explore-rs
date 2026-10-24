# Development-only reference: R glm/lm + unmodified sampleSelection heckitVcov.
# No full sampleSelection installation or production R dependency is claimed.
args <- commandArgs(trailingOnly=TRUE)
if(length(args)!=3)stop('usage: reference_heckman_two_step.R original.asc predictions.csv heckitVcov.R')
if(file.exists(args[2]))stop('predictions output already exists')
# CRAN mirror sampleSelection1.2-16, commit5115e01a03d09a46ac1b92813c785c5799b4d654.
# External GPL>=2 development source only; do not vendor into native production.
if(unname(tools::md5sum(args[3]))!='40b68e067652a7419fd68541a197eeb6')
  stop('unmodified pinned sampleSelection heckitVcov.R required')
reference <- new.env(parent=baseenv())
sys.source(args[3],envir=reference)
d <- read.table(args[1],na.strings='.')
names(d) <- c('e','lnwage','ystar','ytrunc','ycens','dy')
stopifnot(nrow(d)==200,ncol(d)==6,all(is.finite(as.matrix(d[,c(1:3,5:6)]))),
  all(d$dy %in% c(0,1)),sum(d$dy==1)==130,sum(d$dy==0)==70,
  all(is.na(d$ytrunc)==(d$dy==0)),all(d$ytrunc[d$dy==1]==d$ystar[d$dy==1]),
  all(d$ycens==pmax(d$ystar,0)))
n <- nrow(d);selected <- d$dy==1;n1 <- sum(selected)
z <- cbind(intercept=1,lnwage=d$lnwage)
x <- cbind(intercept=1,lnwage=d$lnwage)
glmfit <- glm(dy~lnwage,data=d,family=binomial(link='probit'),
  control=glm.control(epsilon=1e-12,maxit=100))
# glm supplies independent starting coefficients. Symbolic full observed
# derivatives refine/certify them; glm's Fisher covariance is not substituted.
parameters <- c('g0','g1')
expression <- deriv3(~s*log(pnorm(g0+g1*z))+(1-s)*log(pnorm(-g0-g1*z)),
  parameters,function.arg=c(parameters,'z','s'))
evaluate <- function(theta) {
  value <- expression(theta[1],theta[2],d$lnwage,d$dy)
  list(ll=sum(value),score=colSums(attr(value,'gradient')),
    information=-apply(attr(value,'hessian'),c(2,3),sum))
}
refine <- function(theta) {
  for(iteration in 1:20) {
    e <- evaluate(theta);chol(e$information);step <- solve(e$information,e$score)
    score_bound <- max(abs(e$score)*pmax(1,abs(theta)))/n
    step_bound <- max(abs(step)/pmax(1,abs(theta)))
    if(score_bound<1e-12 && step_bound<1e-12)
      return(list(theta=theta,e=e,covariance=solve(e$information),score_bound=score_bound,step_bound=step_bound))
    candidate <- theta+step
    stopifnot(evaluate(candidate)$ll>=e$ll-32*.Machine$double.eps*(1+abs(e$ll)))
    theta <- candidate
  }
  stop('independent observed-Newton probit refinement failed')
}
fit <- refine(unname(coef(glmfit)))
stopifnot(abs(fit$e$ll-as.numeric(logLik(glmfit)))<1e-8)
gamma <- fit$theta;selection_covariance <- fit$covariance
index <- drop(z%*%gamma)
probability <- pnorm(index)
mills <- exp(dnorm(index,log=TRUE)-pnorm(index,log.p=TRUE))
delta <- mills*(mills+index)
w <- cbind(x,mills_coefficient=mills)[selected,,drop=FALSE]
z1 <- z[selected,,drop=FALSE]
y1 <- d$ytrunc[selected]
outcome <- lm.fit(w,y1)
stopifnot(outcome$rank==ncol(w))
beta <- unname(outcome$coefficients);alpha <- beta[3]
sse <- sum(outcome$residuals^2)
# Conventional two-step moment, NOT residual-DF corrected OLS sigma.
sigma <- sqrt(sse/n1+alpha^2*mean(delta[selected]))
rho <- alpha/sigma
stopifnot(is.finite(sigma),sigma>0,is.finite(rho),abs(rho)<1,
  all(is.finite(c(index,probability,mills,delta))),all(delta>=0 & delta<=1))
vcov_reference <- reference$heckitVcov(w,z1,selection_covariance,rho,delta[selected],sigma)
# Independent algebraic reconstruction binds the generated-regressor correction.
a_inv <- solve(crossprod(w));f <- crossprod(w,delta[selected]*z1)
conditional_variance <- sigma^2-alpha^2*delta[selected]
vcov_manual <- a_inv%*%(crossprod(w,conditional_variance*w)+
  alpha^2*f%*%selection_covariance%*%t(f))%*%a_inv
stopifnot(max(abs(vcov_reference-vcov_manual)/(1+abs(vcov_reference)))<1e-11)
invisible(chol(vcov_reference))
normal_equation_bound <- max(abs(crossprod(w,outcome$residuals))/
  (n1*max(1,max(abs(y1)))*sqrt(colMeans(w^2))))
stopifnot(normal_equation_bound<1e-10)
latent <- drop(x%*%beta[1:2])
conditional <- latent+alpha*mills
observed_contribution <- probability*latent+alpha*dnorm(index)
stopifnot(all(is.finite(c(latent,conditional,observed_contribution))))
rows <- list()
emit <- function(quantity,value,i=0,j=0)
  rows[[length(rows)+1L]] <<- data.frame(quantity=quantity,i=i,j=j,value=as.numeric(value))
inference <- function(prefix,value,se,i) {
  value <- unname(value);se <- unname(se)
  statistic <- value/se;p <- 2*pnorm(abs(statistic),lower.tail=FALSE)
  values <- c(value=value,standard_error=se,statistic=statistic,p_value=p,
    confidence_lower=value-qnorm(.975)*se,confidence_upper=value+qnorm(.975)*se)
  stopifnot(all(is.finite(values)),se>0,p>0)
  for(name in names(values))emit(paste0(prefix,'_',name),values[name],i)
}
for(i in 1:2)inference('selection',gamma[i],sqrt(selection_covariance[i,i]),i)
for(i in 1:3)inference('outcome',beta[i],sqrt(vcov_reference[i,i]),i)
for(i in 1:2)for(j in 1:2)emit('selection_covariance',selection_covariance[i,j],i,j)
for(i in 1:3)for(j in 1:3)emit('outcome_covariance',vcov_reference[i,j],i,j)
summary <- c(n=n,selected_count=n1,unselected_count=n-n1,selection_log_likelihood=fit$e$ll,
  sigma=sigma,rho=rho,mills_coefficient=alpha,outcome_sse=sse,
  mean_mills=mean(mills),mean_selected_delta=mean(delta[selected]),
  regressor_mills_correlation=cor(d$lnwage,mills),excluded_selection_predictor_count=0,
  selection_scaled_score=fit$score_bound,selection_scaled_step=fit$step_bound,
  outcome_scaled_normal_equation=normal_equation_bound)
for(name in names(summary))emit(name,summary[name])
predictions <- data.frame(source_row=1:n,d,selection_index=index,selection_probability=probability,
  mills=mills,latent_mean=latent,conditional_selected_mean=conditional,
  observed_contribution_mean=observed_contribution)
# Publish only after all fit/reference/certificate checks succeed.
write.csv(predictions,args[2],row.names=FALSE,na='')
write.csv(do.call(rbind,rows),stdout(),row.names=FALSE,quote=FALSE)
cat('Reference:',R.version.string,'glm/lm; unmodified sampleSelection1.2-16 heckitVcov at5115e01a03d09a46ac1b92813c785c5799b4d654\n',file=stderr())
