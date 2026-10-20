# Development-only R/plm and explicit LSDV reference. No author data bundled.
# Rscript within_reference.R /path/to/MOM.dat fresh_reference_directory
options(digits=17)
args <- commandArgs(trailingOnly=TRUE)
stopifnot(length(args)==2L,!file.exists(args[2]),!dir.exists(args[2]))
v <- scan(args[1],quiet=TRUE)
stopifnot(length(v)==5320L*8L,all(is.finite(v)))
d <- as.data.frame(matrix(v,ncol=8,byrow=TRUE))
names(d) <- c('lnhr','lnwg','kids','ageh','agesq','disab','id','year')
d <- data.frame(source_row=seq_len(nrow(d)),d)
stopifnot(identical(d$id,rep(as.numeric(1:532),each=10)),identical(d$year,rep(as.numeric(1979:1988),532)),!anyDuplicated(d[c('id','year')]))
fit <- plm::plm(lnhr~lnwg,data=d,index=c('id','year'),model='within',effect='individual')
lsdv <- lm(lnhr~0+factor(id)+lnwg,data=d)
b <- unname(coef(fit)); n <- nrow(d); G <- length(unique(d$id)); p <- length(b); df_fit <- n-G-p
stopifnot(n==5320L,G==532L,p==1L,df_fit==4787L,df.residual(fit)==df_fit,
  identical(as.numeric(as.character(plm::index(fit)[[1]])),d$id),
  identical(as.numeric(as.character(plm::index(fit)[[2]])),d$year),
  max(abs(b-tail(coef(lsdv),1)))<1e-12,max(abs(residuals(fit)-residuals(lsdv)))<1e-12)
W <- model.matrix(fit); wy <- plm::pmodel.response(fit); residual <- as.numeric(residuals(fit)); rss <- sum(residual^2)
alpha <- as.numeric(plm::fixef(fit,type='level')); ids <- as.numeric(names(plm::fixef(fit,type='level')))
c0 <- mean(d$lnhr)-mean(d$lnwg)*b
centers <- data.frame(id=ids,observations=as.numeric(table(d$id)[as.character(ids)]),
  response_mean=as.numeric(tapply(d$lnhr,d$id,mean)[as.character(ids)]),
  predictor_mean=as.numeric(tapply(d$lnwg,d$id,mean)[as.character(ids)]),intercept=alpha,effect=alpha-c0)
stopifnot(max(abs(alpha-(centers$response_mean-centers$predictor_mean*b)))<1e-12,
  abs(sum(centers$observations*centers$effect))<1e-9)
unit <- match(d$id,ids); mu <- alpha[unit]+d$lnwg*b
stopifnot(max(abs(mu-fitted(lsdv)))<1e-12,max(abs(d$lnhr-mu-residual))<1e-12)
B <- solve(crossprod(W)); S <- rowsum(W*residual,d$id)
multiplier <- G/(G-1)*(n-1)/(n-p)
rows <- list()
emit <- function(covariance,section,quantity,value) {
  rows[[length(rows)+1L]] <<- data.frame(covariance,section,quantity,value=as.numeric(value))
}
for (covariance in c('nonrobust','cluster(id)')) {
  V <- if(covariance=='nonrobust') vcov(fit) else plm::vcovHC(fit,method='arellano',type='sss',cluster='group')
  expected <- if(covariance=='nonrobust') rss/df_fit*B else multiplier*B%*%crossprod(S)%*%B
  stopifnot(max(abs(V-expected))<1e-12)
  if(covariance=='nonrobust') stopifnot(abs(V[1,1]-vcov(lsdv)[G+1,G+1])<1e-12)
  df <- if(covariance=='nonrobust') df_fit else G-1L
  fields <- c(observation_count=n,group_count=G,slope_count=p,absorbed_intercepts=G,df_residual=df_fit,inference_df=df,
    within_r_squared=1-rss/sum(wy^2),conditional_r_squared=1-rss/sum((d$lnhr-mean(d$lnhr))^2),
    root_mse=sqrt(rss/df_fit),residual_sum_of_squares=rss,within_total_sum_of_squares=sum(wy^2),common_intercept=c0,
    minimum_group_size=10,maximum_group_size=10,singleton_groups=0)
  for(name in names(fields)) emit(covariance,'fit',name,fields[[name]])
  se <- sqrt(V[1,1]); t <- b/se; tail <- 2*pt(-abs(t),df)
  fields <- c(value=b,standard_error=se,statistic=t,p_value=tail)
  for(name in names(fields)) emit(covariance,'coefficient',name,fields[[name]])
  emit(covariance,'covariance','value',V[1,1])
  fields <- c(estimate=b,standard_error=se,statistic=t,p_value=tail,
    ci_lower=b-qt(.975,df)*se,ci_upper=b+qt(.975,df)*se,ci_level=95,df_residual=df)
  for(name in names(fields)) emit(covariance,'lincom',name,fields[[name]])
  fields <- c(statistic=t*t,p_value=pf(t*t,1,df,lower.tail=FALSE),df=1,df_residual=df)
  for(name in names(fields)) emit(covariance,'test',name,fields[[name]])
  prefix <- if(covariance=='nonrobust') 'fe_classical' else 'fe_cluster'
  d[[paste0(prefix,'_xb')]] <- c0+d$lnwg*b
  d[[paste0(prefix,'_xbu')]] <- mu
  d[[paste0(prefix,'_u')]] <- centers$effect[unit]
  d[[paste0(prefix,'_residual')]] <- residual
}
dir.create(args[2])
write.csv(do.call(rbind,rows),file.path(args[2],'reference.csv'),row.names=FALSE,quote=FALSE)
write.csv(d,file.path(args[2],'within_predictions.csv'),row.names=FALSE,quote=FALSE)
write.csv(centers,file.path(args[2],'unit_effects.csv'),row.names=FALSE,quote=FALSE)
# Isolated Stata8 float32-ingestion audit, never the native float64 oracle.
float32 <- function(x) readBin(writeBin(as.double(x),raw(),size=4),double(),n=length(x),size=4)
a <- as.data.frame(matrix(float32(v),ncol=8,byrow=TRUE)); names(a) <- names(d)[2:9]
af <- plm::plm(lnhr~lnwg,data=a,index=c('id','year'),model='within',effect='individual')
ab <- unname(coef(af)); ase <- sqrt(vcov(af)[1,1]); arss <- sum(residuals(af)^2)
author <- c(slope=ab,standard_error=ase,common_intercept=mean(a$lnhr)-mean(a$lnwg)*ab,
  root_mse=sqrt(arss/df_fit),within_r_squared=1-arss/sum(plm::pmodel.response(af)^2),f_statistic=(ab/ase)^2)
write.csv(data.frame(quantity=names(author),value=as.numeric(author)),file.path(args[2],'author_float32.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,'; plm',as.character(packageVersion('plm')),'\n')
cat('Scalar references:',length(rows),'; original records:',n,'; unit effects:',G,'\n')
