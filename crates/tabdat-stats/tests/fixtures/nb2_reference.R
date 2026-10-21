# Development-only NB2 ML reference: MASS start, joint observed information.
# Rscript nb2_reference.R > nb2_reference.csv
# Rscript nb2_reference.R mma20p1count.asc R_nb2_predictions.csv > R_nb2_reference.csv
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args) %in% c(0L,2L))
rows <- list()
emit <- function(case,robust,quantity,i,j,value) {
  rows[[length(rows)+1L]] <<- data.frame(case=case,robust=as.integer(robust),
    quantity=quantity,i=i,j=j,value=unname(value))
}
# This includes beta/lnalpha cross curvature. MASS vcov alone does not.
evaluate <- function(par,X,y) {
  k <- ncol(X); r <- exp(-par[k+1L]); eta <- drop(X %*% par[seq_len(k)])
  mu <- exp(eta)
  sr <- digamma(y+r)-digamma(r)+log(r/(r+mu))+1-(r+y)/(r+mu)
  scores <- cbind(X*(r*(y-mu)/(r+mu)),lnalpha=-r*sr)
  cross <- drop(crossprod(X,r*mu*(y-mu)/(r+mu)^2))
  iaa <- sum(-r*sr-r^2*(trigamma(y+r)-trigamma(r)+1/r-1/(r+mu)+(y-mu)/(r+mu)^2))
  I <- rbind(cbind(crossprod(X,X*(r*mu*(r+y)/(r+mu)^2)),cross),c(cross,iaa))
  list(ll=sum(dnbinom(y,size=r,mu=mu,log=TRUE)),score=colSums(scores),
    information=I,scores=scores,mu=mu,eta=eta)
}
polish <- function(par,X,y) {
  for (iteration in 1:50) {
    e <- evaluate(par,X,y); step <- solve(e$information,e$score)
    if (max(abs(step)) < 1e-14) return(list(par=unname(par),e=e))
    par <- unname(par+step)
  }
  stop('reference did not converge')
}
reference <- function(case,y,predictors,intercept=TRUE,predictions_path=NULL) {
  X <- as.matrix(predictors)
  if (intercept) X <- cbind(intercept=1,X)
  n <- length(y); k <- ncol(X); p <- k+1L
  stopifnot(n>p,all(is.finite(y)),all(y>=0 & y==floor(y)),max(y)<=100000,
    all(is.finite(X)),qr(X)$rank==k)
  frame <- data.frame(y=y,predictors)
  formula <- if (ncol(predictors)==0L) y~1 else if (intercept) y~. else y~.-1
  initial <- MASS::glm.nb(formula,data=frame,control=glm.control(epsilon=1e-13,maxit=100))
  stopifnot(initial$converged)
  fitted <- polish(c(unname(coef(initial)),lnalpha=-log(initial$theta)),X,y)
  par <- unname(fitted$par); e <- fitted$e; alpha <- exp(par[p]); r <- 1/alpha
  B <- solve(e$information)
  # Restricted NB null reestimates nuisance dispersion; it is not a Poisson null.
  if (intercept) {
    null_X <- matrix(1,n,1)
    null_start <- c(log(mean(y)),par[p])
  } else {
    null_X <- matrix(numeric(0),n,0)
    optimum <- optimize(function(a)sum(dnbinom(y,size=exp(-a),mu=1,log=TRUE)),
      c(-12,12),maximum=TRUE,tol=1e-12)
    null_start <- optimum$maximum
  }
  null <- polish(null_start,null_X,y)
  null_ll <- null$e$ll
  lr <- max(0,2*(e$ll-null_ll)); q <- k-as.integer(intercept)
  poisson_fit <- glm.fit(X,y,family=poisson())
  poisson_ll <- sum(dpois(y,poisson_fit$fitted.values,log=TRUE))
  dispersion_lr_raw <- 2*(e$ll-poisson_ll)
  stopifnot(dispersion_lr_raw >= -1e-8)
  dispersion_lr <- max(0,dispersion_lr_raw)
  dispersion_p <- if(dispersion_lr==0) 1 else
    .5*pchisq(dispersion_lr,1,lower.tail=FALSE)
  deviance <- sum(2*(ifelse(y==0,0,y*log(ifelse(y==0,1,y)/e$mu))-
    (y+r)*log((y+r)/(e$mu+r))))
  pearson <- sum((y-e$mu)^2/(e$mu+alpha*e$mu^2))
  # Numerical score differentiation checks the analytic curvature in normalized
  # coordinates; the separate Rust comparisons use this observed, not Fisher, I.
  scales <- c(sqrt(colSums(X^2)),sqrt(n))
  scaled_gradient <- function(t) evaluate(t/scales,X,y)$score/scales
  t <- par*scales; h <- 1e-4
  numeric_I <- -sapply(seq_len(p),function(j) {
    up <- down <- t; up[j]<-up[j]+h; down[j]<-down[j]-h
    (scaled_gradient(up)-scaled_gradient(down))/(2*h)
  })
  expected_I <- e$information/outer(scales,scales)
  stopifnot(max(abs(numeric_I-expected_I))<1e-7,
    min(eigen(expected_I,symmetric=TRUE,only.values=TRUE)$values)>0)
  if (!is.null(predictions_path)) {
    write.csv(data.frame(prepared_row=seq_len(n),MDU=y,predictors,mu=e$mu,xb=e$eta),
      predictions_path,row.names=FALSE,quote=FALSE)
  }
  for (robust in c(FALSE,TRUE)) {
    V <- if (robust) n/(n-1)*B %*% crossprod(e$scores) %*% B else B
    fields <- c(observation_count=n,zero_count=sum(y==0),count_sum=sum(y),
      log_likelihood=e$ll,null_log_likelihood=null_ll,null_alpha=exp(tail(null$par,1)),
      likelihood_ratio=lr,likelihood_ratio_df=q,mcfadden_r_squared=1-e$ll/null_ll,
      residual_deviance=deviance,pearson_chi_square=pearson,df_residual=n-p,
      pearson_dispersion=pearson/(n-p),alpha=alpha,log_alpha=par[p],
      alpha_standard_error=alpha*sqrt(V[p,p]),log_alpha_standard_error=sqrt(V[p,p]),
      alpha_confidence_lower=exp(par[p]-qnorm(.975)*sqrt(V[p,p])),
      alpha_confidence_upper=exp(par[p]+qnorm(.975)*sqrt(V[p,p])),
      log_alpha_confidence_lower=par[p]-qnorm(.975)*sqrt(V[p,p]),
      log_alpha_confidence_upper=par[p]+qnorm(.975)*sqrt(V[p,p]))
    for (name in names(fields)) emit(case,robust,name,0,0,fields[[name]])
    emit(case,robust,'dispersion_test_statistic',0,0,dispersion_lr)
    emit(case,robust,'dispersion_test_p_value',0,0,dispersion_p)
    if (!robust && q>0) emit(case,robust,'likelihood_ratio_p_value',0,0,pchisq(lr,q,lower.tail=FALSE))
    if (robust && q>0) {
      keep <- seq.int(1L+as.integer(intercept),k)
      w <- drop(t(par[keep]) %*% solve(V[keep,keep,drop=FALSE],par[keep]))
      emit(case,robust,'wald_statistic',0,0,w)
      emit(case,robust,'wald_p_value',0,0,pchisq(w,q,lower.tail=FALSE))
    }
    normal <- function(prefix,i,value,variance) {
      value <- unname(value); se <- sqrt(variance)
      fields <- c(value=value,standard_error=se,statistic=value/se,
        p_value=2*pnorm(-abs(value/se)),confidence_lower=value-qnorm(.975)*se,
        confidence_upper=value+qnorm(.975)*se)
      for (name in names(fields)) emit(case,robust,paste0(prefix,name),i,0,fields[[name]])
    }
    for (i in seq_len(k)) normal('coefficient_',i-1L,par[i],V[i,i])
    for (i in seq_len(p)) for (j in seq_len(p)) emit(case,robust,'covariance',i-1L,j-1L,V[i,j])
    for (column in seq_len(q)) {
      j <- column+as.integer(intercept)
      gradient <- c(par[j]*colMeans(X*e$mu),0); gradient[j]<-gradient[j]+mean(e$mu)
      normal('effect_',column-1L,par[j]*mean(e$mu),drop(t(gradient)%*%V%*%gradient))
    }
    if (is.null(predictions_path)) for (i in seq_len(n)) {
      emit(case,robust,'mean',i-1L,0,e$mu[i]); emit(case,robust,'xb',i-1L,0,e$eta[i])
    }
  }
}
if (length(args)==2L) {
  values <- scan(args[1],quiet=TRUE)
  stopifnot(length(values)==20186L*18L)
  d <- as.data.frame(matrix(values,ncol=18,byrow=TRUE))
  names(d) <- c('MDU','LC','IDP','LPI','FMDE','PHYSLIM','NDISEASE','HLTHG','HLTHF',
    'HLTHP','LINC','LFAM','EDUCDEC','AGE','FEMALE','CHILD','FEMCHILD','BLACK')
  stopifnot(sum(d$MDU)==57746,sum(d$MDU==0)==6308,max(d$MDU)==77)
  reference('mma20p1count',d$MDU,d[-1],predictions_path=args[2])
} else {
  x <- seq(-2,2,length.out=36); z <- rep(c(0,1,.4),12)
  # Fixed realizations, not a reference requiring a particular RNG implementation.
  y <- c(0,0,2,0,0,3,0,3,1,3,8,3,0,0,0,2,1,2,0,0,2,1,1,2,0,5,4,1,1,0,0,2,2,0,1,0)
  reference('multiple',y,data.frame(x=x,z=z))
  reference('noconstant',y,data.frame(x=x,z=z),FALSE)
  reference('intercept',y,matrix(numeric(0),36,0))
}
write.csv(do.call(rbind,rows),stdout(),row.names=FALSE,quote=FALSE,na='NA')
