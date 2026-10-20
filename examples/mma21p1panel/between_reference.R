# Development-only independent R/sandwich reference; no author data redistributed.
# Rscript between_reference.R /path/to/MOM.dat fresh_reference_directory
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
# Independent split/mean construction; no SQL/window reuse.
parts <- split(d, d$id)
f <- do.call(rbind, lapply(parts, function(group) {
  group <- group[is.finite(group$lnhr) & is.finite(group$lnwg), ]
  stopifnot(nrow(group)==10L)
  data.frame(id=group$id[1],model_rows=nrow(group),first_year=min(group$year),
    last_year=max(group$year),mean_lnhr=mean(group$lnhr),mean_lnwg=mean(group$lnwg))
}))
f <- f[order(f$id), ]
stopifnot(nrow(f)==532L)
dir.create(args[2])
rows <- list()
emit <- function(case,covariance,section,quantity,i=0L,j=0L,value) {
  rows[[length(rows)+1L]] <<- data.frame(case,covariance,section,quantity,i,j,value=unname(value))
}
for (case in 'between') {
  frame <- f
  outcome <- 'mean_lnhr'
  predictor <- 'mean_lnwg'
  fit <- lm(reformulate(predictor,outcome),data=frame)
  X <- model.matrix(fit); y <- model.response(model.frame(fit)); n <- nrow(X); k <- ncol(X)
  b <- unname(coef(fit)); residual <- residuals(fit); rss <- sum(residual^2)
  for(covariance in c('nonrobust','robust')) {
    V <- switch(covariance,nonrobust=vcov(fit),robust=sandwich::vcovHC(fit,type='HC1'))
    df <- n-k
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
  prefix <- 'between'
  frame[[paste0(prefix,'_fitted')]] <- fitted(fit)
  frame[[paste0(prefix,'_residual')]] <- residual
  write.csv(frame,file.path(args[2],paste0(case,'_predictions.csv')),row.names=FALSE,quote=FALSE)
}
write.csv(do.call(rbind,rows),file.path(args[2],'reference.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,'; sandwich',as.character(packageVersion('sandwich')),'\n')
cat('Scalar references:',length(rows),'; original records:',nrow(d),'; person means:',nrow(f),'\n')
# Separate audit of the author's float32 storage and printed classical facts.
# Native float64 estimates above remain the numerical oracle.
float32 <- function(x) readBin(writeBin(as.double(x),raw(),size=4),double(),n=length(x),size=4)
a <- d
for (name in names(a)[-1]) a[[name]] <- float32(a[[name]])
am <- aggregate(cbind(lnhr,lnwg) ~ id,data=a,FUN=mean)
af <- lm(lnhr ~ lnwg,data=am)
printed <- data.frame(quantity=c('slope','slope_se','intercept','intercept_se','root_mse','r_squared','F'),
  value=c(coef(af)[2],sqrt(vcov(af)[2,2]),coef(af)[1],sqrt(vcov(af)[1,1]),
    summary(af)$sigma,summary(af)$r.squared,unname(summary(af)$fstatistic[1])),
  printed=c('0.0668379','0.0196635','7.483021','0.0518829','0.1772555','0.0213','11.55'))
write.csv(printed,file.path(args[2],'author_printed.csv'),row.names=FALSE,quote=FALSE)
