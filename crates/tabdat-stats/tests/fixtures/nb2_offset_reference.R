# Development-only NB2 fixed-offset reference. Run from repository root:
# Rscript crates/tabdat-stats/tests/fixtures/nb2_offset_reference.R
options(digits=17)
y <- c(0,0,2,0,0,3,0,3,1,3,8,3,0,0,0,2,1,2,0,0,2,1,1,2,0,5,4,1,1,0,0,2,2,0,1,0)
x <- seq(-2,2,length.out=36); z <- rep(c(0,1,.4),12)
o <- rep(c(-1,.2,.6,1.2),9); groups <- rep(1:6,6)
write.csv(data.frame(id=seq_along(y),y,x,z,log_time=o,person=groups),
  'examples/nb2-offset/nb2_offset.csv',row.names=FALSE,quote=FALSE)
rows <- list()
emit <- function(case,quantity,i=0L,j=0L,value) {
  rows[[length(rows)+1L]] <<- data.frame(case,quantity,i,j,value=unname(value))
}
# Full joint beta/lnalpha observed information, not MASS's conditional vcov.
evaluate <- function(par,X,y) {
  k <- ncol(X); r <- exp(-par[k+1L]); eta <- drop(X%*%par[seq_len(k)])+o
  mu <- exp(eta)
  sr <- digamma(y+r)-digamma(r)+log(r/(r+mu))+1-(r+y)/(r+mu)
  scores <- cbind(X*(r*(y-mu)/(r+mu)),lnalpha=-r*sr)
  cross <- drop(crossprod(X,r*mu*(y-mu)/(r+mu)^2))
  iaa <- sum(-r*sr-r^2*(trigamma(y+r)-trigamma(r)+1/r-1/(r+mu)+(y-mu)/(r+mu)^2))
  information <- rbind(cbind(crossprod(X,X*(r*mu*(r+y)/(r+mu)^2)),cross),c(cross,iaa))
  list(ll=sum(dnbinom(y,size=r,mu=mu,log=TRUE)),score=colSums(scores),
    information=information,scores=scores,mu=mu,eta=eta)
}
polish <- function(par,X,y) {
  for (iteration in 1:50) {
    e <- evaluate(par,X,y); step <- solve(e$information,e$score)
    if (max(abs(step))<1e-14) return(list(par=unname(par),e=e))
    par <- unname(par+step)
  }
  stop('joint reference did not converge')
}
for (design in c('multiple','noconstant','intercept')) {
  formula <- switch(design,multiple=y~x+z+offset(o),noconstant=y~x+z+offset(o)-1,intercept=y~1+offset(o))
  initial <- MASS::glm.nb(formula,control=glm.control(epsilon=1e-13,maxit=100))
  stopifnot(initial$converged)
  X <- model.matrix(initial); n <- length(y); k <- ncol(X); p <- k+1L
  intercept <- design!='noconstant'; q <- k-as.integer(intercept)
  fitted <- polish(c(unname(coef(initial)),-log(initial$theta)),X,y)
  par <- fitted$par; e <- fitted$e; alpha <- exp(par[p]); r <- 1/alpha
  B <- solve(e$information)
  stopifnot(min(eigen(e$information,symmetric=TRUE,only.values=TRUE)$values)>0)
  if (intercept) {
    restricted <- MASS::glm.nb(y~1+offset(o),control=glm.control(epsilon=1e-13,maxit=100))
    null <- polish(c(unname(coef(restricted)),-log(restricted$theta)),matrix(1,n,1),y)
  } else {
    null_X <- matrix(numeric(0),n,0)
    optimum <- optimize(function(a)sum(dnbinom(y,size=exp(-a),mu=exp(o),log=TRUE)),
      c(-12,12),maximum=TRUE,tol=1e-12)
    null <- polish(optimum$maximum,null_X,y)
  }
  null_ll <- null$e$ll; lr <- max(0,2*(e$ll-null_ll))
  # Independently differentiate dnbinom and the analytic score in normalized
  # coordinates, both at the optimum and at a nearby nonzero-score point.
  scales <- c(sqrt(colSums(X^2)),sqrt(n)); h <- 1e-4
  for (t in list(par*scales,par*scales+seq_len(p)*.03)) {
    at <- evaluate(t/scales,X,y)
    numeric_score <- sapply(seq_len(p),function(j) {
      up <- down <- t; up[j]<-up[j]+h; down[j]<-down[j]-h
      (evaluate(up/scales,X,y)$ll-evaluate(down/scales,X,y)$ll)/(2*h)
    })
    numeric_I <- -sapply(seq_len(p),function(j) {
      up <- down <- t; up[j]<-up[j]+h; down[j]<-down[j]-h
      (evaluate(up/scales,X,y)$score/scales-evaluate(down/scales,X,y)$score/scales)/(2*h)
    })
    stopifnot(max(abs(numeric_score-at$score/scales))<1e-7,
      max(abs(numeric_I-at$information/outer(scales,scales)))<1e-7)
  }
  deviance <- sum(2*(ifelse(y==0,0,y*log(ifelse(y==0,1,y)/e$mu))-(y+r)*log((y+r)/(e$mu+r))))
  pearson <- sum((y-e$mu)^2/(e$mu+alpha*e$mu^2))
  for (covariance in c('model','robust','cluster')) {
    case <- paste(design,covariance,sep='_')
    V <- switch(covariance,model=B,robust=n/(n-1)*B%*%crossprod(e$scores)%*%B,
      cluster=6/5*B%*%crossprod(rowsum(e$scores,groups))%*%B)
    se_a <- sqrt(V[p,p])
    fields <- c(log_likelihood=e$ll,null_log_likelihood=null_ll,null_alpha=exp(tail(null$par,1)),
      likelihood_ratio=lr,likelihood_ratio_p_value=if(covariance=='model' && q>0) pchisq(lr,q,lower.tail=FALSE) else NA_real_,
      residual_deviance=deviance,pearson_chi_square=pearson,df_residual=n-p,
      mcfadden_r_squared=1-e$ll/null_ll,alpha=alpha,log_alpha=par[p],
      alpha_standard_error=alpha*se_a,log_alpha_standard_error=se_a,
      alpha_confidence_lower=exp(par[p]-qnorm(.975)*se_a),alpha_confidence_upper=exp(par[p]+qnorm(.975)*se_a),
      log_alpha_confidence_lower=par[p]-qnorm(.975)*se_a,log_alpha_confidence_upper=par[p]+qnorm(.975)*se_a)
    for (name in names(fields)) emit(case,name,value=fields[[name]])
    if (covariance!='model' && q>0) {
      keep <- seq.int(1L+as.integer(intercept),k)
      w <- drop(t(par[keep])%*%solve(V[keep,keep,drop=FALSE],par[keep]))
      emit(case,'wald_statistic',value=w);emit(case,'wald_p_value',value=pchisq(w,q,lower.tail=FALSE))
    }
    normal <- function(prefix,i,value,variance) {
      se <- sqrt(variance)
      values <- c(value, se, value/se, 2*pnorm(-abs(value/se)),value-qnorm(.975)*se,value+qnorm(.975)*se)
      names(values) <- c('value','standard_error','statistic','p_value','confidence_lower','confidence_upper')
      for (name in names(values)) emit(case,paste0(prefix,name),i,value=values[[name]])
    }
    for (i in seq_len(k)) normal('coefficient_',i-1L,par[i],V[i,i])
    for (i in seq_len(p)) for (j in seq_len(p)) emit(case,'covariance',i-1L,j-1L,V[i,j])
    for (column in seq_len(q)) {
      j <- column+as.integer(intercept)
      gradient <- c(par[j]*colMeans(X*e$mu),0);gradient[j]<-gradient[j]+mean(e$mu)
      normal('effect_',column-1L,par[j]*mean(e$mu),drop(t(gradient)%*%V%*%gradient))
    }
    for (i in seq_len(n)) {
      emit(case,'mean',i-1L,value=e$mu[i]);emit(case,'xb',i-1L,value=e$eta[i])
    }
  }
}
write.csv(do.call(rbind,rows),'crates/tabdat-stats/tests/fixtures/nb2_offset_reference.csv',
  row.names=FALSE,quote=FALSE,na='NA')
cat(R.version.string,'; MASS',as.character(packageVersion('MASS')),'\n')
