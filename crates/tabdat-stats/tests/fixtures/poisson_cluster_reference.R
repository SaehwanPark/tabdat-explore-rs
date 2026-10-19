# Development-only independent one-way Poisson cluster reference.
# Rscript poisson_cluster_reference.R > poisson_cluster_reference.csv
# Rscript poisson_cluster_reference.R mma20p1count.asc randdata.dta R_predictions.csv > R_reference.csv
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args) %in% c(0L,3L))
rows <- list()
emit <- function(case,quantity,i=0L,j=0L,value) {
  rows[[length(rows)+1L]] <<- data.frame(case=case,quantity=quantity,i=i,j=j,value=unname(value))
}
reference <- function(case,y,predictors,groups,intercept=TRUE,metadata=NULL,predictions_path=NULL) {
  frame <- data.frame(y=y,predictors)
  formula <- if (ncol(predictors)==0L) y~1 else if (intercept) y~. else y~.-1
  fit <- glm(formula,data=frame,family=poisson(),control=glm.control(epsilon=1e-13,maxit=100))
  # glm's likelihood stopping rule leaves its QR/working weights at the prior
  # IRLS point. Warm-start at the fixed point before asking sandwich for V.
  fit <- glm(formula,data=frame,family=poisson(),start=coef(fit),control=glm.control(epsilon=1e-13,maxit=100))
  stopifnot(fit$converged,max(abs(fit$weights-fit$fitted.values))<1e-10)
  X <- model.matrix(fit); b <- coef(fit); mu <- fitted(fit)
  n <- length(y); k <- length(b); q <- k-as.integer(intercept)
  G <- length(unique(groups))
  stopifnot(length(groups)==n,!anyNA(groups),G>=2L,qr(X)$rank==k,n>k)
  V <- sandwich::vcovCL(fit,cluster=groups,type='HC0',cadjust=TRUE)
  S <- rowsum(X*(y-mu),groups,reorder=FALSE)
  B <- solve(crossprod(X,X*mu))
  manual <- G/(G-1)*B %*% crossprod(S) %*% B
  stopifnot(max(abs(V-manual))<1e-10)
  null_mu <- if (intercept) mean(y) else 1
  ll <- sum(dpois(y,mu,log=TRUE)); null_ll <- sum(dpois(y,null_mu,log=TRUE))
  lr <- max(0,2*(ll-null_ll))
  fields <- c(observation_count=n,zero_count=sum(y==0),count_sum=sum(y),
    cluster_group_count=G,cluster_correction=G/(G-1),log_likelihood=ll,
    null_log_likelihood=null_ll,likelihood_ratio=lr,likelihood_ratio_df=q,
    likelihood_ratio_p_value=NA_real_,mcfadden_r_squared=1-ll/null_ll,
    residual_deviance=sum(2*(ifelse(y==0,0,y*log(ifelse(y==0,1,y)/mu))-y+mu)),
    pearson_chi_square=sum((y-mu)^2/mu),df_residual=n-k,
    pearson_dispersion=sum((y-mu)^2/mu)/(n-k))
  for (name in names(fields)) emit(case,name,value=fields[[name]])
  if (q>0L && q<G) {
    keep <- seq.int(1L+as.integer(intercept),k)
    w <- drop(t(b[keep]) %*% solve(V[keep,keep,drop=FALSE],b[keep]))
    emit(case,'wald_statistic',value=w)
    emit(case,'wald_p_value',value=pchisq(w,q,lower.tail=FALSE))
  }
  normal <- function(prefix,i,value,variance) {
    value <- unname(value)
    se <- sqrt(variance)
    f <- c(value=unname(value),standard_error=se,
      statistic=if(se>0)value/se else NA_real_,
      p_value=if(se>0)2*pnorm(-abs(value/se)) else NA_real_,
      confidence_lower=if(se>0)value-qnorm(.975)*se else NA_real_,
      confidence_upper=if(se>0)value+qnorm(.975)*se else NA_real_)
    for (name in names(f)) emit(case,paste0(prefix,name),i,value=f[[name]])
  }
  for (i in seq_len(k)) normal('coefficient_',i-1L,b[i],V[i,i])
  for (i in seq_len(k)) for (j in seq_len(k)) emit(case,'covariance',i-1L,j-1L,V[i,j])
  for (column in seq_len(q)) {
    j <- column+as.integer(intercept)
    gradient <- b[j]*colMeans(X*mu); gradient[j]<-gradient[j]+mean(mu)
    normal('effect_',column-1L,b[j]*mean(mu),drop(t(gradient)%*%V%*%gradient))
  }
  if (is.null(predictions_path)) {
    for (i in seq_len(n)) {
      emit(case,'mean',i-1L,value=mu[i]);emit(case,'xb',i-1L,value=predict(fit)[i])
    }
  } else {
    stopifnot(nrow(metadata)==n)
    write.csv(data.frame(metadata,MDU=y,predictors,mu=mu,xb=predict(fit)),
      predictions_path,row.names=FALSE,quote=FALSE)
  }
}
if (length(args)==3L) {
  values <- scan(args[1],quiet=TRUE)
  stopifnot(length(values)==20186L*18L)
  d <- as.data.frame(matrix(values,ncol=18,byrow=TRUE))
  names(d) <- c('MDU','LC','IDP','LPI','FMDE','PHYSLIM','NDISEASE','HLTHG','HLTHF',
    'HLTHP','LINC','LFAM','EDUCDEC','AGE','FEMALE','CHILD','FEMCHILD','BLACK')
  r <- foreign::read.dta(args[2],convert.factors=FALSE)
  keep <- !is.na(r$educdec)
  stopifnot(nrow(r)==20190L,identical(which(!keep),c(9746L,10153L,10400L,10859L)))
  selected <- r[keep,]
  map <- c('mdvis','logc','idp','lpi','fmde','physlm','disea','hlthg','hlthf',
    'hlthp','linc','lfam','educdec','xage','female','child','fchild','black')
  stopifnot(nrow(selected)==nrow(d),identical(d$MDU,as.numeric(selected$mdvis)),
    all(vapply(seq_along(map),function(j)max(abs(d[[j]]-selected[[map[j]]])),numeric(1))<5e-6),
    !anyNA(selected$zper),all(is.finite(selected$zper)),all(selected$zper==floor(selected$zper)),
    identical(sort(unique(selected$year)),as.numeric(1:5)),
    !anyDuplicated(selected[c('zper','year')]),length(unique(selected$zper))==5908L,
    identical(as.integer(table(selected$year)),c(5638L,5574L,5545L,1715L,1714L)),
    identical(as.integer(table(as.integer(table(selected$zper)))),c(265L,246L,3743L,70L,1584L)))
  metadata <- data.frame(prepared_row=seq_len(nrow(d)),original_row=which(keep),
    person_id=selected$zper,study_year=selected$year)
  reference('mma20p1count_person',d$MDU,d[-1],metadata$person_id,
    metadata=metadata,predictions_path=args[3])
} else {
  y <- c(0,0,2,0,0,3,0,3,1,3,8,3,0,0,0,2,1,2,0,0,2,1,1,2,0,5,4,1,1,0,0,2,2,0,1,0)
  predictors <- data.frame(x=seq(-2,2,length.out=36),z=rep(c(0,1,.4),12))
  groups <- rep(seq_len(6),6)
  reference('multiple',y,predictors,groups)
  reference('noconstant',y,predictors,groups,FALSE)
  reference('intercept',y,matrix(numeric(0),36,0),groups)
  reference('singleton',y,predictors,seq_len(36))
}
write.csv(do.call(rbind,rows),stdout(),row.names=FALSE,quote=FALSE,na='NA')
