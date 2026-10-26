# Independent OLS QR and profiled exponential-density likelihood; development only.
args <- commandArgs(trailingOnly=TRUE)
if (length(args)!=2) stop('usage: Rscript reference.R source.asc new-output-directory')
if (dir.exists(args[2])) stop('output directory already exists')
d <- read.table(args[1],col.names=c('y','x'))
stopifnot(nrow(d)==10000,all(is.finite(d$x)),all(is.finite(d$y)),all(d$y>0))
n <- nrow(d); X <- cbind(1,d$x)
ols <- lm.fit(X,d$y); u <- ols$residuals; olsfit <- as.vector(X%*%ols$coefficients)
A <- solve(crossprod(X)); ols_classical <- sum(u*u)/(n-2)*A
ols_robust <- n/(n-2)*A%*%crossprod(X,X*(u*u))%*%A
score <- function(b) {w <- d$y*exp(b*d$x); mean(d$x)-sum(w*d$x)/sum(w)}
b <- uniroot(score,c(-3,3),tol=1e-14)$root
a <- log(n/sum(d$y*exp(b*d$x)))
coef <- c(a,b); eta <- as.vector(X%*%coef); weight <- d$y*exp(eta)
B <- solve(crossprod(X,X*weight))
exp_robust <- n/(n-1)*B%*%crossprod(X,X*(1-weight)^2)%*%B
mu <- exp(-eta); e <- d$y-mu
stopifnot(abs(score(b))<1e-12,all(is.finite(mu)),all(mu>0))
reference <- data.frame(key=character(),value=double())
add <- function(k,v) {reference <<- rbind(reference,data.frame(key=k,value=v))}
for (model in c('ols_classical','ols_robust','exp_classical','exp_robust')) {
  isols <- startsWith(model,'ols_'); beta <- if(isols) ols$coefficients else coef
  V <- switch(model,ols_classical=ols_classical,ols_robust=ols_robust,
    exp_classical=B,exp_robust=exp_robust)
  se <- sqrt(diag(V)); stat <- beta/se
  p <- if(isols) 2*pt(-abs(stat),df=n-2) else 2*pnorm(-abs(stat))
  for (i in 1:2) {
    prefix <- paste(model,i,sep='.')
    add(paste0(prefix,'.value'),beta[i]); add(paste0(prefix,'.se'),se[i])
    add(paste0(prefix,'.stat'),stat[i]); add(paste0(prefix,'.p'),p[i])
    if (!isols) {
      add(paste0(prefix,'.lower'),beta[i]-qnorm(.975)*se[i])
      add(paste0(prefix,'.upper'),beta[i]+qnorm(.975)*se[i])
    }
    for (j in 1:2) add(paste(model,'cov',i,j,sep='.'),V[i,j])
  }
}
add('ols_log_likelihood',-n/2*(log(2*pi)+1+log(sum(u*u)/n)))
add('ols_r_squared',1-sum(u*u)/sum((d$y-mean(d$y))^2))
add('density_log_likelihood',sum(eta-weight))
add('poisson_log_likelihood',sum(eta-weight+log(d$y)))
add('exponential_r_squared',1-sum(e*e)/sum((d$y-mean(d$y))^2))
projections <- data.frame(source_row=seq_len(n),y=d$y,x=d$x,
  ols_classical=olsfit,ols_hc1=olsfit,integrated_classical=weight,integrated_robust=weight,
  exponential_mean=mu,density_log_likelihood=eta-weight,exponential_residual=e,ols_residual=u)
dir.create(args[2]); options(digits=17)
write.csv(reference,file.path(args[2],'reference.csv'),row.names=FALSE,quote=FALSE)
write.csv(projections,file.path(args[2],'projections.csv'),row.names=FALSE,quote=FALSE)
cat(sprintf('OLS a=%.17g b=%.17g; exponential a=%.17g b=%.17g; density LL=%.17g\n',ols$coefficients[1],ols$coefficients[2],a,b,sum(eta-weight)))
