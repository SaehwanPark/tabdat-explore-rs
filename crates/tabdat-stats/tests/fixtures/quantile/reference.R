# Independent quantreg6.1 point/IID covariance; native normal rather than t tails.
options(digits=17)
root <- 'crates/tabdat-stats/tests/fixtures/quantile/'
i <- 1:120;x <- (i-60)/13;z <- sin(.7*i)+.2*cos(.19*i)
d <- data.frame(y=1.2+.75*x-.2*z+.35*sin(1.37*i)+.2*cos(.17*i),x,z)
write.csv(d,paste0(root,'inputs.csv'),row.names=FALSE,quote=FALSE)
rows <- list();emit <- function(case,path,value) rows[[length(rows)+1]] <<- data.frame(case,path,value=unname(value))
for(case in c('q10','median','q90','noconstant','shifted','rescaled')) {
 f <- d;tau <- switch(case,q10=.1,q90=.9,.5)
 if(case=='shifted') f$y <- f$y+100
 if(case=='rescaled') f$x <- f$x*100
 fit <- quantreg::rq(if(case=='noconstant') y~x+z-1 else y~x+z,f,tau=tau,method='br')
 s <- summary(fit,se='iid',covariance=TRUE);b <- unname(coef(fit));K <- length(b);V <- s$cov
 for(j in 1:K) {
  se <- sqrt(V[j,j]);stat <- b[j]/se
  v <- c(value=b[j],standard_error=se,statistic=stat,p_value=2*pnorm(-abs(stat)),ci_lower=b[j]-qnorm(.975)*se,ci_upper=b[j]+qnorm(.975)*se)
  for(n in names(v))emit(case,paste0('/coefficients/',j-1,'/',n),v[[n]])
  for(k in 1:K)emit(case,paste0('/covariance_matrix/',j-1,'/',k-1),V[j,k])
 }
 loss <- function(r)sum(r*(tau-(r<0)))
 emit(case,'/check_loss',loss(residuals(fit)));emit(case,'/sparsity',1/s$scale)
 emit(case,'/observation_count',nrow(f));emit(case,'/quantile',tau)
 if(case!='noconstant') {
  null <- loss(f$y-quantile(f$y,tau,type=1,names=FALSE))
  emit(case,'/null_check_loss',null);emit(case,'/pseudo_r_squared',1-loss(residuals(fit))/null)
 }
}
write.csv(do.call(rbind,rows),paste0(root,'reference.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,'quantreg',as.character(packageVersion('quantreg')),';',length(rows),'quantile quantities\n')
