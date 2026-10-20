# Development-only base R reference; run from repository root.
# Rscript crates/tabdat-stats/tests/fixtures/poisson_offset_reference.R
options(digits=17)
y <- c(0,0,2,0,0,3,0,3,1,3,8,3,0,0,0,2,1,2,0,0,2,1,1,2,0,5,4,1,1,0,0,2,2,0,1,0)
x <- seq(-2,2,length.out=36); z <- rep(c(0,1,.4),12)
o <- rep(c(-1,.2,.6,1.2),9); groups <- rep(1:6,6)
write.csv(data.frame(id=seq_along(y),y,x,z,log_time=o,person=groups),
  'examples/poisson-offset/poisson_offset.csv',row.names=FALSE,quote=FALSE)
rows <- list()
emit <- function(case,quantity,i=0L,j=0L,value) {
  rows[[length(rows)+1L]] <<- data.frame(case,quantity,i,j,value=unname(value))
}
for (design in c('multiple','noconstant','intercept')) {
  formula <- switch(design,multiple=y~x+z,noconstant=y~x+z-1,intercept=y~1)
  fit <- glm(formula,family=poisson(),offset=o,control=glm.control(epsilon=1e-13,maxit=100))
  fit <- glm(formula,family=poisson(),offset=o,start=coef(fit),control=glm.control(epsilon=1e-13,maxit=100))
  stopifnot(fit$converged)
  X <- model.matrix(fit); b <- coef(fit); mu <- fitted(fit)
  n <- length(y); k <- length(b); intercept <- design!='noconstant'; q <- k-as.integer(intercept)
  bread <- solve(crossprod(X,X*mu)); score <- X*(y-mu)
  null <- if (intercept) glm(y~1,family=poisson(),offset=o,control=glm.control(epsilon=1e-13)) else NULL
  null_mu <- if (intercept) fitted(null) else exp(o)
  ll <- sum(dpois(y,mu,log=TRUE)); null_ll <- sum(dpois(y,null_mu,log=TRUE))
  for (covariance in c('model','robust','cluster')) {
    case <- paste(design,covariance,sep='_')
    V <- switch(covariance,model=bread,
      robust=n/(n-1)*bread%*%crossprod(score)%*%bread,
      cluster=6/5*bread%*%crossprod(rowsum(score,groups))%*%bread)
    normal <- function(prefix,i,value,variance) {
      se <- sqrt(variance)
      values <- c(value=unname(value),standard_error=se,statistic=unname(value)/se,
        p_value=2*pnorm(-abs(value/se)),confidence_lower=value-qnorm(.975)*se,
        confidence_upper=value+qnorm(.975)*se)
      names(values) <- c('value','standard_error','statistic','p_value','confidence_lower','confidence_upper')
      for (name in names(values)) emit(case,paste0(prefix,name),i,value=values[[name]])
    }
    fields <- c(log_likelihood=ll,null_log_likelihood=null_ll,likelihood_ratio=2*(ll-null_ll),
      likelihood_ratio_p_value=if(covariance=='model' && q>0) pchisq(2*(ll-null_ll),q,lower.tail=FALSE) else NA_real_,
      residual_deviance=deviance(fit),pearson_chi_square=sum((y-mu)^2/mu),
      df_residual=n-k,mcfadden_r_squared=1-ll/null_ll)
    for (name in names(fields)) emit(case,name,value=fields[[name]])
    if (covariance!='model' && q>0L) {
      keep <- seq.int(1L+as.integer(intercept),k)
      w <- drop(t(b[keep])%*%solve(V[keep,keep,drop=FALSE],b[keep]))
      emit(case,'wald_statistic',value=w)
      emit(case,'wald_p_value',value=pchisq(w,q,lower.tail=FALSE))
    }
    for (i in seq_len(k)) {
      normal('coefficient_',i-1L,b[i],V[i,i])
      for (j in seq_len(k)) emit(case,'covariance',i-1L,j-1L,V[i,j])
    }
    for (column in seq_len(q)) {
      j <- column+as.integer(intercept)
      gradient <- b[j]*colMeans(X*mu); gradient[j] <- gradient[j]+mean(mu)
      normal('effect_',column-1L,b[j]*mean(mu),drop(t(gradient)%*%V%*%gradient))
    }
    for (i in seq_len(n)) {
      emit(case,'mean',i-1L,value=mu[i]); emit(case,'xb',i-1L,value=predict(fit)[i])
    }
  }
}
write.csv(do.call(rbind,rows),'crates/tabdat-stats/tests/fixtures/poisson_offset_reference.csv',
  row.names=FALSE,quote=FALSE,na='NA')
cat(R.version.string,'\n')
