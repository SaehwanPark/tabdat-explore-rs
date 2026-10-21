# R 4.6.1, quantreg 6.1. Independent ASCII sample, rq(br), summary.rq IID/HS.
# Native tails/intervals explicitly normal; summary.rq's finite-t tails are not used.
options(digits=17)
a <- commandArgs(TRUE); dir.create(a[2],recursive=TRUE,showWarnings=FALSE)
if(grepl('\\.asc$',a[1])) {
 d <- as.data.frame(scan(a[1],what=list(0,0,0,'','',0,0,0,0),na.strings='.',quote='"',quiet=TRUE),stringsAsFactors=FALSE)
 names(d) <- c('sex','age','educyr98','farm','urban98','hhsize','lhhexp1','lhhex12m','lnrlfood')
 stopifnot(nrow(d)==5999);d$source_row <- seq_len(nrow(d))
} else d <- read.csv(a[1])
d <- d[is.finite(d$lhhexp1)&is.finite(d$lhhex12m),]
if(grepl('\\.asc$',a[1])) stopifnot(nrow(d)==5006)
d$lntotal <- d$lhhexp1;d$lnmed <- d$lhhex12m
out <- d[,c('source_row','lntotal','lnmed')];rows <- list()
emit <- function(case,path,value) rows[[length(rows)+1]] <<- data.frame(case,path,value=unname(value))
for(tau in c(.1,.5,.9)) {
 case <- paste0('q',tau*100);fit <- quantreg::rq(lnmed~lntotal,d,tau=tau,method='br')
 s <- summary(fit,se='iid',covariance=TRUE,hs=TRUE);b <- unname(coef(fit));V <- s$cov
 for(j in 1:2) {
  se <- sqrt(V[j,j]);stat <- b[j]/se
  v <- c(value=b[j],standard_error=se,statistic=stat,p_value=2*pnorm(-abs(stat)),ci_lower=b[j]-qnorm(.975)*se,ci_upper=b[j]+qnorm(.975)*se)
  for(n in names(v))emit(case,paste0('/coefficients/',j-1,'/',n),v[[n]])
  for(k in 1:2)emit(case,paste0('/covariance_matrix/',j-1,'/',k-1),V[j,k])
 }
 loss <- function(r)sum(r*(tau-(r<0)))
 null <- loss(d$lnmed-quantile(d$lnmed,tau,type=1,names=FALSE))
 for(n in c('check_loss','null_check_loss','pseudo_r_squared','sparsity','observation_count','quantile')) {
   val <- switch(n,check_loss=loss(residuals(fit)),null_check_loss=null,pseudo_r_squared=1-loss(residuals(fit))/null,sparsity=1/s$scale,observation_count=nrow(d),quantile=tau)
   emit(case,paste0('/',n),val)
 }
 out[[case]] <- as.numeric(predict(fit,d))
 if(tau==.5) out$median_residual <- residuals(fit)
}
write.csv(do.call(rbind,rows),file.path(a[2],'reference.csv'),row.names=FALSE,quote=FALSE)
write.csv(out,file.path(a[2],'predictions.csv'),row.names=FALSE,quote=FALSE)
cat(R.version.string,'quantreg',as.character(packageVersion('quantreg')),';',length(rows),'quantities\n')
