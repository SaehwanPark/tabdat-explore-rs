# Independent development-only base-R Poisson/Fisher/score/delta reference.
# Synthetic: Rscript poisson_reference.R > poisson_reference.csv
# Author ASCII: Rscript poisson_reference.R mma20p1count.asc R_predictions.csv > R_reference.csv
options(digits = 17)
args <- commandArgs(trailingOnly = TRUE)
stopifnot(length(args) %in% c(0L, 2L))
rows <- list()
emit <- function(case, robust, quantity, i, j, value) {
  rows[[length(rows)+1L]] <<- data.frame(case=case, robust=as.integer(robust),
    quantity=quantity, i=i, j=j, value=value)
}
reference <- function(case, y, predictors, intercept=TRUE, predictions_path=NULL) {
  X <- as.matrix(predictors)
  if (intercept) X <- cbind(intercept=1, X)
  n <- length(y); k <- ncol(X)
  stopifnot(n > k, all(is.finite(y)), all(y >= 0 & y == floor(y)),
    all(is.finite(X)), qr(X)$rank == k)
  fit <- glm.fit(X, y, family=poisson(), intercept=intercept,
    control=glm.control(epsilon=1e-13,maxit=100))
  stopifnot(fit$converged)
  b <- coef(fit)
  # Reevaluate information at the score fixed point, not cached glm QR weights.
  for (iteration in 1:100) {
    mu <- exp(drop(X %*% b))
    B <- solve(crossprod(X, X*mu))
    step <- drop(B %*% crossprod(X, y-mu))
    if (max(abs(step)) < 1e-14) break
    b <- b+step
  }
  if (intercept && k==1L) b <- c(intercept=log(mean(y)))
  eta <- drop(X %*% b); mu <- exp(eta)
  B <- solve(crossprod(X, X*mu)); scores <- X*(y-mu)
  ll <- sum(dpois(y,mu,log=TRUE))
  null_mu <- if (intercept) mean(y) else 1
  null_ll <- sum(dpois(y,null_mu,log=TRUE))
  lr <- max(0,2*(ll-null_ll)); q <- k-as.integer(intercept)
  deviance <- sum(2*ifelse(y==0,mu,y*log(ifelse(y==0,1,y)/mu)-y+mu))
  pearson <- sum((y-mu)^2/mu)
  if (!is.null(predictions_path)) {
    write.csv(data.frame(prepared_row=seq_len(n), MDU=y, predictors, mu=mu, xb=eta),
      predictions_path, row.names=FALSE, quote=FALSE)
  }
  for (robust in c(FALSE,TRUE)) {
    V <- if (robust) n/(n-1)*B %*% crossprod(scores) %*% B else B
    fields <- c(observation_count=n, zero_count=sum(y==0), count_sum=sum(y),
      log_likelihood=ll, null_log_likelihood=null_ll, likelihood_ratio=lr,
      likelihood_ratio_df=q, mcfadden_r_squared=1-ll/null_ll,
      residual_deviance=deviance, pearson_chi_square=pearson,
      df_residual=n-k, pearson_dispersion=pearson/(n-k))
    for (name in names(fields)) emit(case,robust,name,0,0,fields[[name]])
    if (!robust && q>0) emit(case,robust,"likelihood_ratio_p_value",0,0,pchisq(lr,q,lower.tail=FALSE))
    if (robust && q>0) {
      keep <- (k-q+1L):k
      if (all(diag(V)[keep]>0) && qr(V[keep,keep,drop=FALSE])$rank==q) {
        w <- drop(t(b[keep]) %*% solve(V[keep,keep,drop=FALSE],b[keep]))
        emit(case,robust,"wald_statistic",0,0,w)
        emit(case,robust,"wald_p_value",0,0,pchisq(w,q,lower.tail=FALSE))
      }
    }
    normal <- function(prefix, i, value, variance) {
      value <- unname(value)
      se <- sqrt(variance)
      z <- if (se>0) value/se else NA_real_
      fields <- c(value=value,standard_error=se,statistic=z,
        p_value=if (se>0) 2*pnorm(-abs(z)) else NA_real_,
        confidence_lower=if (se>0) value-qnorm(.975)*se else NA_real_,
        confidence_upper=if (se>0) value+qnorm(.975)*se else NA_real_)
      for (name in names(fields)) emit(case,robust,paste0(prefix,name),i,0,fields[[name]])
    }
    for (i in seq_len(k)) {
      normal("coefficient_",i-1L,b[i],V[i,i])
      for (j in seq_len(k)) emit(case,robust,"covariance",i-1L,j-1L,V[i,j])
    }
    for (column in seq_len(q)) {
      j <- column+as.integer(intercept)
      gradient <- b[j]*colMeans(X*mu); gradient[j] <- gradient[j]+mean(mu)
      normal("effect_",column-1L,b[j]*mean(mu),drop(t(gradient)%*%V%*%gradient))
    }
    if (is.null(predictions_path)) for (i in seq_len(n)) {
      emit(case,robust,"mean",i-1L,0,mu[i]); emit(case,robust,"xb",i-1L,0,eta[i])
    }
  }
}
if (length(args)==2L) {
  values <- scan(args[1],quiet=TRUE)
  stopifnot(length(values)==20186L*18L)
  d <- as.data.frame(matrix(values,ncol=18,byrow=TRUE))
  names(d) <- c("MDU","LC","IDP","LPI","FMDE","PHYSLIM","NDISEASE","HLTHG",
    "HLTHF","HLTHP","LINC","LFAM","EDUCDEC","AGE","FEMALE","CHILD","FEMCHILD","BLACK")
  stopifnot(sum(d$MDU)==57746, sum(d$MDU==0)==6308, max(d$MDU)==77)
  reference("mma20p1count",d$MDU,d[-1],predictions_path=args[2])
} else {
  x <- c(-2,-1,0,1,2,3,.5,1.5,-.5,2.5,-1.5,3.5)
  z <- c(0,1,0,1,0,1,0,1,1,0,.2,.7)
  y <- c(0,1,0,2,1,5,2,0,3,1,4,2)
  reference("multiple",y,data.frame(x=x,z=z))
  reference("noconstant",y,data.frame(x=x,z=z),FALSE)
  reference("intercept",c(0,1,2,4,0,3),matrix(numeric(0),6,0))
  reference("zero_noconstant",rep(0,4),data.frame(x=c(-2,-1,1,2)),FALSE)
  reference("constant",rep(1,4),matrix(numeric(0),4,0))
}
write.csv(do.call(rbind,rows),stdout(),row.names=FALSE,quote=FALSE,na="NA")
