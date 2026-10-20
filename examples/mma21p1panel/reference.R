# Development-only independent R/sandwich reference; no author data redistributed.
# Rscript reference.R /path/to/MOM.dat fresh_reference_directory
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==2L, !dir.exists(args[2]), !file.exists(args[2]))
v <- scan(args[1],quiet=TRUE)
stopifnot(length(v)==5320L*8L,all(is.finite(v)))
d <- as.data.frame(matrix(v,ncol=8,byrow=TRUE))
names(d) <- c('lnhr','lnwg','kids','ageh','agesq','disab','id','year')
d <- data.frame(source_row=seq_len(nrow(d)),d)
stopifnot(identical(d$id,rep(as.numeric(1:532),each=10)),
  identical(d$year,rep(as.numeric(1979:1988),532)),!anyDuplicated(d[c('id','year')]))
# Construct consecutive differences by explicit key lookup, NOT adjacent file
# rows or deletion of incomplete model rows before finding the previous year.
key <- paste(d$id,d$year,sep='/')
previous <- match(paste(d$id,d$year-1,sep='/'),key)
keep <- !is.na(previous)
f <- data.frame(source_row=d$source_row[keep],id=d$id[keep],year=d$year[keep],
  lnhr=d$lnhr[keep],lnwg=d$lnwg[keep],previous_source_row=d$source_row[previous[keep]],
  previous_year=d$year[previous[keep]],dlnhr=d$lnhr[keep]-d$lnhr[previous[keep]],
  dlnwg=d$lnwg[keep]-d$lnwg[previous[keep]])
stopifnot(nrow(f)==4788L,length(unique(f$id))==532L,all(f$year-f$previous_year==1))
dir.create(args[2])
rows <- list()
emit <- function(case,covariance,section,quantity,i=0L,j=0L,value) {
  rows[[length(rows)+1L]] <<- data.frame(case,covariance,section,quantity,i,j,value=unname(value))
}
for (case in c('pooled','fd')) {
  frame <- if(case=='pooled') d else f
  outcome <- if(case=='pooled') 'lnhr' else 'dlnhr'
  predictor <- if(case=='pooled') 'lnwg' else 'dlnwg'
  fit <- lm(reformulate(predictor,outcome),data=frame)
  X <- model.matrix(fit); y <- model.response(model.frame(fit)); n <- nrow(X); k <- ncol(X)
  b <- unname(coef(fit)); residual <- residuals(fit); rss <- sum(residual^2); G <- length(unique(frame$id))
  B <- solve(crossprod(X)); S <- rowsum(X*residual,frame$id)
  for(covariance in c('nonrobust','robust','cluster(id)')) {
    V <- switch(covariance,nonrobust=vcov(fit),robust=sandwich::vcovHC(fit,type='HC1'),
      'cluster(id)'=sandwich::vcovCL(fit,cluster=frame$id,type='HC1',cadjust=TRUE))
    if(covariance=='cluster(id)') stopifnot(max(abs(V-G/(G-1)*(n-1)/(n-k)*B%*%crossprod(S)%*%B))<1e-10)
    df <- if(covariance=='cluster(id)') G-1 else n-k
    fields <- c(observation_count=n,df_residual=n-k,r_squared=summary(fit)$r.squared,
      adjusted_r_squared=summary(fit)$adj.r.squared,root_mse=sqrt(rss/(n-k)),
      log_likelihood=-n/2*(log(2*pi)+1+log(rss/n)))
    for(name in names(fields)) emit(case,covariance,'fit',name,value=fields[[name]])
    for(i in seq_len(k)) {
      se <- sqrt(V[i,i]); t <- b[i]/se
      fields <- c(value=b[i],standard_error=se,statistic=t,p_value=2*pt(-abs(t),df))
      names(fields) <- c('value','standard_error','statistic','p_value')
      for(name in names(fields)) emit(case,covariance,'coefficient',name,i-1L,value=fields[[name]])
      for(j in seq_len(k)) emit(case,covariance,'covariance','value',i-1L,j-1L,V[i,j])
    }
    se <- sqrt(V[2,2]); t <- b[2]/se; p <- 2*pt(-abs(t),df)
    fields <- c(estimate=b[2],standard_error=se,statistic=t,p_value=p,
      ci_lower=b[2]-qt(.975,df)*se,ci_upper=b[2]+qt(.975,df)*se,ci_level=95,df_residual=df)
    names(fields) <- c('estimate','standard_error','statistic','p_value','ci_lower','ci_upper','ci_level','df_residual')
    for(name in names(fields)) emit(case,covariance,'lincom',name,value=fields[[name]])
    fields <- c(statistic=t*t,p_value=pf(t*t,1,df,lower.tail=FALSE),df=1,df_residual=df)
    for(name in names(fields)) emit(case,covariance,'test',name,value=fields[[name]])
  }
  prefix <- if(case=='pooled') 'pols' else 'fd'
  frame[[paste0(prefix,'_fitted')]] <- fitted(fit)
  frame[[paste0(prefix,'_residual')]] <- residual
  write.csv(frame,file.path(args[2],paste0(case,'_predictions.csv')),row.names=FALSE,quote=FALSE)
}
write.csv(do.call(rbind,rows),file.path(args[2],'reference.csv'),row.names=FALSE,quote=FALSE)
# The author log's describe output identifies float32 storage. Audit its
# printed coefficient/SE facts separately; do NOT round the native f64 design.
float32 <- function(x) readBin(writeBin(as.double(x),raw(),size=4),double(),n=length(x),size=4)
a <- as.data.frame(matrix(float32(v),ncol=8,byrow=TRUE)); names(a) <- names(d)[-1]
selected <- which(a$year>1979)
af <- data.frame(id=a$id[selected],
  dlnhr=float32(a$lnhr[selected]-a$lnhr[selected-1L]),
  dlnwg=float32(a$lnwg[selected]-a$lnwg[selected-1L]))
author <- list()
for(case in c('pooled','fd')) {
  frame <- if(case=='pooled') a else af
  predictor <- if(case=='pooled') 'lnwg' else 'dlnwg'
  fit <- lm(reformulate(predictor,if(case=='pooled') 'lnhr' else 'dlnhr'),data=frame)
  for(covariance in c('nonrobust','robust','cluster(id)')) {
    V <- switch(covariance,nonrobust=vcov(fit),robust=sandwich::vcovHC(fit,type='HC1'),
      'cluster(id)'=sandwich::vcovCL(fit,cluster=frame$id,type='HC1',cadjust=TRUE))
    author[[length(author)+1L]] <- data.frame(case,covariance,parameter=c('intercept',predictor),
      value=unname(coef(fit)),standard_error=sqrt(diag(V)))
  }
}
write.csv(do.call(rbind,author),file.path(args[2],'author_float32.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,'; sandwich',as.character(packageVersion('sandwich')),'\n')
cat('Scalar references:',length(rows),'; original records:',nrow(d),'; first differences:',nrow(f),'\n')
