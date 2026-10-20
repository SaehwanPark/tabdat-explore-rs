# Development-only: Rscript fixed_effects_reference.R output.csv
# R4.6.1/plm2.6.7; deterministic owned fixtures, no external data/RNG.
options(digits=17)
args <- commandArgs(trailingOnly=TRUE); stopifnot(length(args)==1L)
rows <- list()
emit <- function(case,covariance,pointer,value) {
  rows[[length(rows)+1L]] <<- data.frame(case,covariance,pointer,value=unname(value))
}
for(case in c('balanced','unbalanced','interleaved')) {
  sizes <- if(case=='unbalanced') 1:6 else rep(4,6)
  d <- do.call(rbind,lapply(1:6,function(id) {
    t <- 0:(sizes[id]-1)
    x <- .3*id+.2*t+.07*((id*t+2)%%5)
    z <- .13*t*t+.03*((id+2*t)%%4)
    y <- 1+.4*id+.7*x-.3*z+.11*((id*t+3)%%7)
    data.frame(id=paste0('g',id),time=t,y,x,z)
  }))
  if(case=='interleaved') d <- d[order(-d$time,-as.integer(sub('g','',d$id))),]
  rownames(d) <- seq_len(nrow(d))
  n <- nrow(d);p <- 2;G <- length(unique(d$id));df <- n-G-p
  fit <- plm::plm(y~x+z,d,index=c('id','time'),model='within')
  lsdv <- lm(y~x+z+factor(id),d)
  b <- unname(coef(fit));stopifnot(max(abs(b-coef(lsdv)[c('x','z')]))<1e-10,df.residual(fit)==df)
  X <- as.matrix(d[c('x','z')]);Xw <- X
  for(j in 1:p) Xw[,j] <- X[,j]-ave(X[,j],d$id)
  yw <- d$y-ave(d$y,d$id); fitted_within <- drop(Xw%*%b);e <- yw-fitted_within
  stopifnot(max(abs(e-residuals(lsdv)))<1e-10)
  rss <- sum(e^2);tss <- sum(yw^2);B <- solve(crossprod(Xw));S <- rowsum(Xw*e,d$id)
  for(covariance in c('classical','cluster')) {
    V <- if(covariance=='classical') vcov(fit) else plm::vcovHC(fit,method='arellano',type='sss',cluster='group')
    expected <- if(covariance=='classical') vcov(lsdv)[c('x','z'),c('x','z')] else G/(G-1)*(n-1)/(n-p)*B%*%crossprod(S)%*%B
    stopifnot(max(abs(V-expected))<1e-10)
    infer_df <- if(covariance=='classical') df else G-1
    for(i in 1:p) {
      values <- c(b[i],sqrt(V[i,i]),b[i]/sqrt(V[i,i]),2*pt(-abs(b[i]/sqrt(V[i,i])),infer_df))
      for(j in 1:4) emit(case,covariance,paste0('/within/coefficients/',i-1,'/',c('value','standard_error','statistic','p_value')[j]),values[j])
      for(j in 1:p) emit(case,covariance,paste0('/within/covariance/matrix/',i-1,'/',j-1),V[i,j])
    }
    stats <- c(observation_count=n,parameter_count=p,degrees_of_freedom=df,
      r_squared=1-rss/tss,root_mse=sqrt(rss/df),residual_sum_of_squares=rss,total_sum_of_squares=tss)
    for(name in names(stats)) emit(case,covariance,paste0('/within/fit_statistics/',name),stats[[name]])
    if(covariance=='classical') emit(case,covariance,'/within/fit_statistics/f_statistic',((tss-rss)/p)/(rss/df))
    common <- mean(d$y)-sum(colMeans(X)*b)
    emit(case,covariance,'/common_intercept',common)
    emit(case,covariance,'/conditional_r_squared',1-rss/sum((d$y-mean(d$y))^2))
    emit(case,covariance,'/minimum_group_size',min(sizes));emit(case,covariance,'/maximum_group_size',max(sizes))
    emit(case,covariance,'/singleton_groups',sum(sizes==1))
    for(i in seq_along(unique(d$id))) {
      selected <- which(d$id==unique(d$id)[i]); ym <- mean(d$y[selected]);xm <- colMeans(X[selected,,drop=FALSE]);alpha <- ym-sum(xm*b)
      prefix <- paste0('/effects/',i-1,'/')
      values <- c(observation_count=length(selected),response_mean=ym,intercept=alpha,effect=alpha-common)
      for(name in names(values)) emit(case,covariance,paste0(prefix,name),values[[name]])
      for(j in 1:p) emit(case,covariance,paste0(prefix,'predictor_means/',j-1),xm[j])
    }
    for(i in 1:n) {
      emit(case,covariance,paste0('/fitted_values/',i-1),fitted(lsdv)[i])
      emit(case,covariance,paste0('/within/fitted_values/',i-1),fitted_within[i])
      emit(case,covariance,paste0('/within/residuals/',i-1),e[i])
    }
    weights <- c(1,-.5);estimate <- sum(weights*b)+2;se <- sqrt(drop(t(weights)%*%V%*%weights));tt <- estimate/se
    values <- c(estimate=estimate,standard_error=se,statistic=tt,p_value=2*pt(-abs(tt),infer_df),
      ci_lower=estimate-qt(.975,infer_df)*se,ci_upper=estimate+qt(.975,infer_df)*se,degrees_of_freedom=infer_df)
    for(name in names(values)) emit(case,covariance,paste0('/lincom/',name),values[[name]])
    F <- drop(t(b)%*%solve(V,b))/p
    values <- c(f_statistic=F,df_num=p,df_denom=infer_df,p_value=pf(F,p,infer_df,lower.tail=FALSE))
    for(name in names(values)) emit(case,covariance,paste0('/joint/',name),values[[name]])
  }
}
write.csv(do.call(rbind,rows),args[1],row.names=FALSE,quote=FALSE)
cat(R.version.string,'plm',as.character(packageVersion('plm')),'references:',length(rows),'\n')
